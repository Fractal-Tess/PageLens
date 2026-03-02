<script lang="ts">
  import { onMount } from 'svelte'
  import { push } from 'svelte-spa-router'
  import { Button } from '$components/ui/button'
  import { Card, CardContent, CardHeader, CardTitle } from '$components/ui/card'
  import { ScrollArea } from '$components/ui/scroll-area'
  import { Separator } from '$components/ui/separator'
  import * as Chart from '$components/ui/chart'
  import { LineChart, PieChart, Text } from 'layerchart'
  import { curveLinearClosed } from 'd3-shape'
  import { scaleBand } from 'd3-scale'
  import {
    AlertCircle,
    Image,
    Heading,
    Activity,
    FileText
  } from '@lucide/svelte'
  import { commands, type AnalysisRun } from '$lib/ipc'
  import type {
    CrawlResult,
    SeoReport,
    SinglePagePayload,
    Snapshot
  } from '$lib/types'
  import IssuesTable from '$lib/components/report/IssuesTable.svelte'
  import MetaDetails from '$lib/components/report/MetaDetails.svelte'
  import SocialMeta from '$lib/components/report/SocialMeta.svelte'
  import HeadingStructure from '$lib/components/report/HeadingStructure.svelte'
  import PerformanceTimeline from '$lib/components/report/PerformanceTimeline.svelte'
  import DomTree from '$lib/components/report/DomTree.svelte'

  type PageRow = {
    id: string
    run_id: string
    url: string
    depth: number
    success: boolean
    seo_score: number | null
    total_issues: number
    error_count: number
    warning_count: number
    links_found_count: number
    error_message: string | null
    analyzed_at: string
  }

  let { params = {} }: { params?: { id?: string; pageId?: string } } = $props()

  let run = $state<AnalysisRun | null>(null)
  let pageRow = $state<PageRow | null>(null)
  let singlePayload = $state<SinglePagePayload | null>(null)
  let seo = $state<SeoReport | null>(null)
  let snapshot = $state<Snapshot | null>(null)
  let isLoading = $state(true)
  let error = $state<string | null>(null)
  let contrastIssues = $state<
    Array<{
      id: string
      selector: string
      text: string
      ratio: number
      required: number
      fg: string
      bg: string
    }>
  >([])
  let selectedContrastId = $state<string | null>(null)
  let domPreviewFrame = $state<HTMLIFrameElement | null>(null)

  const ogImage = $derived(
    seo?.open_graph.image ?? seo?.twitter_card.image ?? null
  )
  function escapeHtmlAttribute(value: string): string {
    return value
      .replaceAll('&', '&amp;')
      .replaceAll('"', '&quot;')
      .replaceAll('<', '&lt;')
      .replaceAll('>', '&gt;')
  }

  function buildPreviewHtml(currentSnapshot: Snapshot | null): string {
    if (!currentSnapshot?.html) return ''

    const sanitized = currentSnapshot.html
      .replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '')
      .replace(/\son\w+="[^"]*"/gi, '')
      .replace(/\son\w+='[^']*'/gi, '')

    const baseTag = currentSnapshot.url
      ? `<base href="${escapeHtmlAttribute(currentSnapshot.url)}">`
      : ''

    const stylesheetLinks = (
      currentSnapshot.referenced_assets?.stylesheets ?? []
    )
      .filter(Boolean)
      .map(
        href => `<link rel="stylesheet" href="${escapeHtmlAttribute(href)}">`
      )
      .join('')

    const injectedHead = `${baseTag}${stylesheetLinks}`

    if (/<head\b[^>]*>/i.test(sanitized)) {
      return sanitized.replace(/<head([^>]*)>/i, `<head$1>${injectedHead}`)
    }

    return `<head>${injectedHead}</head>${sanitized}`
  }

  const previewHtml = $derived(buildPreviewHtml(snapshot))
  function clampScore(value: number): number {
    return Math.max(0, Math.min(100, Math.round(value)))
  }

  function average(values: number[]): number | null {
    if (values.length === 0) return null
    return values.reduce((sum, value) => sum + value, 0) / values.length
  }

  function severityWeight(severity: 'Error' | 'Warning' | 'Info'): number {
    switch (severity) {
      case 'Error':
        return 12
      case 'Warning':
        return 6
      default:
        return 2
    }
  }

  function scoreByIssues(
    currentSeo: SeoReport | null,
    categories: string[],
    baseline = 100
  ): number | null {
    if (!currentSeo) return null
    const penalties = currentSeo.issues
      .filter(issue => categories.includes(issue.category))
      .reduce((sum, issue) => sum + severityWeight(issue.severity), 0)
    return clampScore(baseline - penalties)
  }

  function scoreFromThresholdMs(
    value: number | null,
    good: number,
    bad: number
  ): number | null {
    if (value == null || Number.isNaN(value)) return null
    if (value <= good) return 100
    if (value >= bad) return 0
    return ((bad - value) / (bad - good)) * 100
  }

  const performanceScore = $derived.by(() => {
    const timing = snapshot?.performance_timing
    if (!timing) return null

    const measurements = [
      scoreFromThresholdMs(timing.first_contentful_paint, 1800, 4500),
      scoreFromThresholdMs(timing.dom_interactive, 2000, 5000),
      scoreFromThresholdMs(timing.dom_content_loaded, 2500, 6000),
      scoreFromThresholdMs(timing.load_complete, 3500, 8000)
    ].filter(value => value != null)

    const value = average(measurements)
    return value == null ? 50 : clampScore(value)
  })

  const accessibilityScore = $derived.by(() => {
    let score = 100
    const treeSize = snapshot?.accessibility_tree?.length ?? 0

    if (treeSize === 0) score -= 55
    else if (treeSize < 10) score -= 25

    const imagePenalty = seo?.issues
      .filter(issue => issue.category === 'images')
      .reduce((sum, issue) => sum + severityWeight(issue.severity), 0)

    return clampScore(score - (imagePenalty ?? 0))
  })

  const semanticHtmlScore = $derived.by(() => {
    if (!seo) return null

    let score = 100
    const h1Count = seo.headings.h1_count

    if (h1Count === 0) score -= 40
    else if (h1Count > 1) score -= 20
    if (seo.headings.structure.length === 0) score -= 20

    const headingIssuePenalty = seo.issues
      .filter(issue => issue.category === 'headings')
      .reduce((sum, issue) => sum + severityWeight(issue.severity), 0)

    return clampScore(score - headingIssuePenalty)
  })

  const contrastScore = $derived.by(() => {
    if (!snapshot) return null
    if (contrastIssues.length === 0) return 100

    const averageDeficit =
      contrastIssues.reduce((sum, issue) => {
        const ratioDeficit = Math.max(0, issue.required - issue.ratio)
        return sum + ratioDeficit / issue.required
      }, 0) / contrastIssues.length

    const penalty = contrastIssues.length * 6 + averageDeficit * 40
    return clampScore(100 - penalty)
  })

  const seoMetaScore = $derived.by(() => {
    if (!seo) return null

    let score = scoreByIssues(seo, ['meta', 'canonical'], 100) ?? 100
    if (!seo.meta.title) score -= 8
    if (!seo.meta.description) score -= 8
    if (!seo.canonical_url) score -= 4

    return clampScore(score)
  })

  const socialMetadataScore = $derived.by(() => {
    if (!seo) return null

    const checks = [
      seo.open_graph.title,
      seo.open_graph.description,
      seo.open_graph.image,
      seo.twitter_card.card,
      seo.twitter_card.title,
      seo.twitter_card.description
    ]

    const present = checks.filter(Boolean).length
    return clampScore((present / checks.length) * 100)
  })

  const overallScore = $derived.by(() => {
    const categoryScores = [
      performanceScore,
      accessibilityScore,
      semanticHtmlScore,
      contrastScore,
      seoMetaScore,
      socialMetadataScore
    ].filter(value => value != null)

    const value = average(categoryScores)
    return value == null ? null : clampScore(value)
  })

  const overallChartData = $derived([
    {
      segment: 'score',
      value: overallScore ?? 0,
      color: 'var(--chart-1)'
    },
    {
      segment: 'remaining',
      value: overallScore == null ? 100 : Math.max(0, 100 - overallScore),
      color: 'var(--muted)'
    }
  ])
  const overallChartConfig = {
    score: { label: 'Score', color: 'var(--chart-1)' },
    remaining: { label: 'Remaining', color: 'var(--muted)' }
  } satisfies Chart.ChartConfig
  const categoryRadarConfig = {
    score: { label: 'Score', color: 'var(--chart-2)' }
  } satisfies Chart.ChartConfig
  const categoryScoreData = $derived.by(() => {
    return [
      { category: 'Performance', score: performanceScore },
      { category: 'Accessibility', score: accessibilityScore },
      { category: 'Semantic HTML', score: semanticHtmlScore },
      { category: 'Contrast', score: contrastScore },
      { category: 'SEO (Meta)', score: seoMetaScore },
      { category: 'Social metadata', score: socialMetadataScore }
    ].map(category => ({
      category: category.category,
      score: category.score ?? 0
    }))
  })

  onMount(async () => {
    const runId = params.id
    const pageId = params.pageId
    if (!runId || !pageId) {
      error = 'Missing run or page id'
      isLoading = false
      return
    }

    try {
      const runResult = await commands.getHistoryItem(runId)
      if (runResult.status === 'error') throw new Error(runResult.error)
      run = runResult.data

      const pagesResult = await commands.listAnalysisPageResults(runId)
      if (pagesResult.status === 'error') throw new Error(pagesResult.error)

      const selected = pagesResult.data.find(page => page.id === pageId) ?? null
      pageRow = selected
      if (!selected) throw new Error('Page result not found')

      const parsed = JSON.parse(run.payload_json)
      if (run.analysis_type === 'single') {
        singlePayload = parsed as SinglePagePayload
        seo = singlePayload.seo_report
        snapshot = singlePayload.snapshot
      } else {
        const crawl = parsed as CrawlResult
        const match =
          crawl.pages.find(
            page => page.url === selected.url && page.depth === selected.depth
          ) ??
          crawl.pages.find(page => page.url === selected.url) ??
          null
        seo = match?.seo_report ?? null
        snapshot = match?.snapshot ?? null
      }

      error = null
    } catch (err) {
      error = err instanceof Error ? err.message : String(err)
    } finally {
      isLoading = false
    }
  })

  function parseRgbaColor(color: string | null): {
    rgb: [number, number, number]
    alpha: number
  } | null {
    if (!color) return null
    const match = color.match(
      /rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([\d.]+))?\)/i
    )
    if (!match) return null
    const alpha = match[4] == null ? 1 : Number(match[4])
    return {
      rgb: [Number(match[1]), Number(match[2]), Number(match[3])],
      alpha
    }
  }

  function relativeLuminance([r, g, b]: [number, number, number]): number {
    const convert = (value: number) => {
      const channel = value / 255
      return channel <= 0.03928
        ? channel / 12.92
        : ((channel + 0.055) / 1.055) ** 2.4
    }
    return 0.2126 * convert(r) + 0.7152 * convert(g) + 0.0722 * convert(b)
  }

  function contrastRatio(
    foreground: [number, number, number],
    background: [number, number, number]
  ): number {
    const l1 = relativeLuminance(foreground)
    const l2 = relativeLuminance(background)
    const lighter = Math.max(l1, l2)
    const darker = Math.min(l1, l2)
    return (lighter + 0.05) / (darker + 0.05)
  }

  function cssPath(element: Element): string {
    const segments: string[] = []
    let current: Element | null = element
    while (current && current.tagName.toLowerCase() !== 'html') {
      const tag = current.tagName.toLowerCase()
      if (current.id) {
        segments.unshift(`#${current.id}`)
        break
      }
      const parent: Element | null = current.parentElement
      if (!parent) {
        segments.unshift(tag)
        break
      }
      const siblings = Array.from(
        parent.children as HTMLCollectionOf<Element>
      ).filter(item => item.tagName.toLowerCase() === tag)
      const index = siblings.indexOf(current) + 1
      segments.unshift(`${tag}:nth-of-type(${index})`)
      current = parent
    }
    return segments.join(' > ')
  }

  function clearHighlights() {
    selectedContrastId = null
    const doc = domPreviewFrame?.contentDocument
    if (!doc) return
    doc.querySelectorAll('[data-contrast-highlight="1"]').forEach(node => {
      const el = node as HTMLElement
      el.style.outline = ''
      el.style.outlineOffset = ''
      el.style.backgroundColor = ''
      el.removeAttribute('data-contrast-highlight')
    })
  }

  function highlightContrastIssue(issueId: string) {
    selectedContrastId = issueId
    clearHighlights()

    const doc = domPreviewFrame?.contentDocument
    if (!doc) return
    const issue = contrastIssues.find(item => item.id === issueId)
    if (!issue) return
    const target = doc.querySelector(issue.selector) as HTMLElement | null
    if (!target) return

    target.setAttribute('data-contrast-highlight', '1')
    target.style.outline = '3px solid #ef4444'
    target.style.outlineOffset = '2px'
    target.style.backgroundColor = 'rgba(239,68,68,0.08)'
    target.scrollIntoView({ block: 'center', behavior: 'smooth' })
  }

  function scanContrastIssues() {
    const doc = domPreviewFrame?.contentDocument
    if (!doc) return

    const found: Array<{
      id: string
      selector: string
      text: string
      ratio: number
      required: number
      fg: string
      bg: string
    }> = []

    const elements = Array.from(doc.body.querySelectorAll('*'))
    for (const element of elements) {
      if (!(element instanceof HTMLElement)) continue
      const text = (element.innerText || '').trim().replace(/\s+/g, ' ')
      if (!text) continue
      if (text.length > 160) continue

      const style = doc.defaultView?.getComputedStyle(element)
      if (!style) continue
      if (style.display === 'none' || style.visibility === 'hidden') continue

      const fgColor = parseRgbaColor(style.color)
      const bgColor = parseRgbaColor(style.backgroundColor)
      if (!fgColor) continue

      const background =
        bgColor && bgColor.alpha > 0
          ? bgColor.rgb
          : ([255, 255, 255] as [number, number, number])
      const ratio = contrastRatio(fgColor.rgb, background)

      const fontSize = Number.parseFloat(style.fontSize || '16')
      const fontWeight = Number.parseFloat(style.fontWeight || '400')
      const isLarge = fontSize >= 24 || (fontSize >= 18.66 && fontWeight >= 700)
      const required = isLarge ? 3 : 4.5

      if (ratio < required) {
        found.push({
          id: `${found.length + 1}`,
          selector: cssPath(element),
          text,
          ratio,
          required,
          fg: style.color,
          bg: style.backgroundColor
        })
      }
      if (found.length >= 80) break
    }

    contrastIssues = found
  }

  function onPreviewLoad() {
    scanContrastIssues()
    setTimeout(scanContrastIssues, 250)
    setTimeout(scanContrastIssues, 1000)
  }
</script>

<div class="h-full overflow-auto p-6 space-y-6">
  {#if isLoading}
    <div class="flex items-center justify-center py-12 text-muted-foreground">
      Loading page details...
    </div>
  {:else if error}
    <Card>
      <CardContent class="py-8 text-center space-y-3">
        <AlertCircle class="mx-auto h-10 w-10 text-destructive" />
        <p class="font-medium">Could not load page details</p>
        <p class="text-sm text-muted-foreground">{error}</p>
        <Button variant="outline" onclick={() => push(`/run/${params.id}`)}
          >Back to run</Button
        >
      </CardContent>
    </Card>
  {:else if run && pageRow}
    <div class="grid gap-4 md:grid-cols-4">
      <Card
        ><CardContent class="pt-5"
          ><div class="mb-3 text-xs text-muted-foreground">Overall score</div>
          <Chart.Container
            config={overallChartConfig}
            class="mx-auto aspect-square max-h-[180px]"
          >
            <PieChart
              data={overallChartData}
              key="segment"
              value="value"
              c="color"
              innerRadius={62}
              padding={20}
              range={[-90, 90]}
              props={{ pie: { sort: null } }}
              cornerRadius={4}
            >
              {#snippet aboveMarks()}
                <Text
                  value={overallScore == null ? '-' : String(overallScore)}
                  textAnchor="middle"
                  verticalAnchor="middle"
                  class="fill-foreground text-2xl! font-bold"
                  dy={-14}
                />
                <Text
                  value="Overall"
                  textAnchor="middle"
                  verticalAnchor="middle"
                  class="fill-muted-foreground! text-muted-foreground"
                  dy={4}
                />
              {/snippet}
              {#snippet tooltip()}
                <Chart.Tooltip hideLabel />
              {/snippet}
            </PieChart>
          </Chart.Container></CardContent
        ></Card
      >
      <Card
        class="md:col-span-2 border-border/60 bg-gradient-to-br from-card via-card to-muted/20"
        ><CardContent class="pt-4 pb-4"
          ><div class="mb-3 flex items-start justify-between gap-3">
            <div>
              <p class="text-xs uppercase tracking-wide text-muted-foreground">
                Issue categories
              </p>
              <p class="text-sm font-medium text-foreground/90">
                Radar view across quality dimensions
              </p>
            </div>
            <span
              class="rounded-full border border-border/70 bg-background/60 px-2 py-1 text-xs text-muted-foreground"
            >
              {categoryScoreData.length} axes
            </span>
          </div>

          {#if categoryScoreData.length > 0}
            <div
              class="grid gap-3 md:grid-cols-[minmax(0,1fr)_200px] md:items-center"
            >
              <Chart.Container
                config={categoryRadarConfig}
                class="mx-auto aspect-square h-[188px] w-full max-h-[188px]"
              >
                <LineChart
                  data={categoryScoreData}
                  series={[
                    {
                      key: 'score',
                      label: 'Score',
                      color: categoryRadarConfig.score.color
                    }
                  ]}
                  radial
                  x="category"
                  xScale={scaleBand()}
                  padding={22}
                  props={{
                    spline: {
                      curve: curveLinearClosed,
                      fill: 'var(--color-score)',
                      fillOpacity: 0.3,
                      stroke: 'var(--color-score)',
                      strokeWidth: 2,
                      motion: 'tween'
                    },
                    xAxis: {
                      tickLength: 0
                    },
                    yAxis: {
                      format: () => ''
                    },
                    grid: {
                      radialY: 'linear'
                    },
                    tooltip: {
                      context: {
                        mode: 'voronoi'
                      }
                    },
                    highlight: {
                      lines: false
                    }
                  }}
                >
                  {#snippet tooltip()}
                    <Chart.Tooltip />
                  {/snippet}
                </LineChart>
              </Chart.Container>

              <div class="space-y-1.5">
                {#each categoryScoreData as item}
                  <div
                    class="flex items-center justify-between rounded-md border border-border/60 bg-background/40 px-2.5 py-1.5"
                  >
                    <span class="truncate text-xs text-muted-foreground"
                      >{item.category}</span
                    >
                    <span
                      class="rounded-full bg-primary/10 px-2 py-0.5 text-xs font-semibold text-primary"
                    >
                      {Math.round(item.score)}
                    </span>
                  </div>
                {/each}
              </div>
            </div>
          {:else}
            <div
              class="h-[210px] rounded-md border border-dashed border-border/70"
            ></div>
          {/if}</CardContent
        ></Card
      >
      <Card><CardContent class="pt-5 min-h-[180px]"></CardContent></Card>
    </div>

    <Card>
      <CardHeader
        ><CardTitle class="text-base flex items-center gap-2"
          ><Activity class="h-4 w-4" /> Performance</CardTitle
        ></CardHeader
      >
      <CardContent>
        {#if snapshot}
          <PerformanceTimeline timing={snapshot.performance_timing} />
        {:else}
          <p class="text-sm text-muted-foreground">
            No performance timing captured.
          </p>
        {/if}
      </CardContent>
    </Card>

    <Card>
      <CardHeader
        ><CardTitle class="text-base flex items-center gap-2"
          ><Heading class="h-4 w-4" /> DOM preview</CardTitle
        ></CardHeader
      >
      <CardContent>
        {#if previewHtml}
          <iframe
            bind:this={domPreviewFrame}
            title="DOM preview"
            class="h-[460px] w-full rounded border bg-white"
            srcdoc={previewHtml}
            onload={onPreviewLoad}
          ></iframe>
        {:else}
          <p class="text-sm text-muted-foreground">
            No HTML snapshot available.
          </p>
        {/if}
      </CardContent>
    </Card>

    <div class="grid gap-4 lg:grid-cols-[320px_minmax(0,1fr)_320px]">
      <Card>
        <CardHeader
          ><CardTitle class="text-base flex items-center gap-2"
            ><Image class="h-4 w-4" /> OG preview</CardTitle
          ></CardHeader
        >
        <CardContent>
          {#if ogImage}
            <img
              src={ogImage}
              alt="Open Graph preview"
              class="w-full rounded border object-cover"
            />
          {:else}
            <div
              class="rounded border bg-muted p-6 text-center text-sm text-muted-foreground"
            >
              No OG image
            </div>
          {/if}
          {#if seo?.meta.title}
            <p class="mt-3 text-sm font-medium">{seo.meta.title}</p>
          {/if}
          {#if seo?.meta.description}
            <p class="mt-1 text-xs text-muted-foreground">
              {seo.meta.description}
            </p>
          {/if}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle class="text-base flex items-center gap-2"
            ><Heading class="h-4 w-4" /> Accessibility tree</CardTitle
          >
        </CardHeader>
        <CardContent>
          {#if snapshot && snapshot.accessibility_tree.length > 0}
            <ScrollArea class="h-[420px] pr-2">
              <DomTree nodes={snapshot.accessibility_tree} />
            </ScrollArea>
          {:else}
            <div
              class="rounded border bg-muted p-8 text-center text-sm text-muted-foreground"
            >
              No DOM/accessibility tree captured.
            </div>
          {/if}
        </CardContent>
      </Card>

      <Card>
        <CardHeader
          ><CardTitle class="text-base">Quick stats</CardTitle></CardHeader
        >
        <CardContent class="space-y-3">
          <Separator />
          <div class="text-sm space-y-1">
            <div class="flex justify-between">
              <span class="text-muted-foreground">Errors</span><span
                >{pageRow.error_count}</span
              >
            </div>
            <div class="flex justify-between">
              <span class="text-muted-foreground">Warnings</span><span
                >{pageRow.warning_count}</span
              >
            </div>
            <div class="flex justify-between">
              <span class="text-muted-foreground">Depth</span><span
                >{pageRow.depth}</span
              >
            </div>
            <div class="flex justify-between">
              <span class="text-muted-foreground">Page type</span><span
                >{run.analysis_type}</span
              >
            </div>
          </div>
        </CardContent>
      </Card>
    </div>

    <div class="grid gap-4 lg:grid-cols-2">
      <Card class="lg:col-span-2">
        <CardHeader
          ><CardTitle class="text-base flex items-center gap-2"
            ><AlertCircle class="h-4 w-4" /> Contrast issues</CardTitle
          ></CardHeader
        >
        <CardContent>
          {#if contrastIssues.length === 0}
            <p class="text-sm text-muted-foreground">
              No contrast issues detected from the captured page preview.
            </p>
          {:else}
            <div class="space-y-2">
              {#each contrastIssues as issue}
                <button
                  type="button"
                  class={`w-full rounded border p-3 text-left text-sm hover:bg-muted ${selectedContrastId === issue.id ? 'border-red-400 bg-red-50/50' : ''}`}
                  onmouseenter={() => highlightContrastIssue(issue.id)}
                  onmouseleave={clearHighlights}
                >
                  <div class="flex items-center justify-between gap-2">
                    <span class="font-medium truncate">{issue.text}</span>
                    <span class="text-xs text-red-600"
                      >{issue.ratio.toFixed(2)} / {issue.required.toFixed(
                        1
                      )}</span
                    >
                  </div>
                  <div class="mt-1 text-xs text-muted-foreground truncate">
                    {issue.selector}
                  </div>
                </button>
              {/each}
            </div>
          {/if}
        </CardContent>
      </Card>

      <Card class="lg:col-span-2">
        <CardHeader
          ><CardTitle class="text-base flex items-center gap-2"
            ><AlertCircle class="h-4 w-4" /> Issues</CardTitle
          ></CardHeader
        >
        <CardContent>
          {#if seo}
            <IssuesTable issues={seo.issues} />
          {:else}
            <p class="text-sm text-muted-foreground">
              No issue details available for this page.
            </p>
          {/if}
        </CardContent>
      </Card>

      <Card>
        <CardHeader
          ><CardTitle class="text-base flex items-center gap-2"
            ><FileText class="h-4 w-4" /> Meta</CardTitle
          ></CardHeader
        >
        <CardContent>
          {#if seo}
            <MetaDetails meta={seo.meta} canonical={seo.canonical_url} />
          {:else}
            <p class="text-sm text-muted-foreground">
              No meta details available.
            </p>
          {/if}
        </CardContent>
      </Card>

      <Card>
        <CardHeader
          ><CardTitle class="text-base flex items-center gap-2"
            ><Heading class="h-4 w-4" /> Headings</CardTitle
          ></CardHeader
        >
        <CardContent>
          {#if seo}
            <HeadingStructure headings={seo.headings} />
          {:else}
            <p class="text-sm text-muted-foreground">
              No heading details available.
            </p>
          {/if}
        </CardContent>
      </Card>

      <Card class="lg:col-span-2">
        <CardHeader
          ><CardTitle class="text-base flex items-center gap-2"
            ><Image class="h-4 w-4" /> Social</CardTitle
          ></CardHeader
        >
        <CardContent>
          {#if seo}
            <SocialMeta og={seo.open_graph} twitter={seo.twitter_card} />
          {:else}
            <p class="text-sm text-muted-foreground">
              No social metadata available.
            </p>
          {/if}
        </CardContent>
      </Card>
    </div>
  {/if}
</div>
