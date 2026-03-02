<script lang="ts">
  import { onMount } from 'svelte'
  import { push } from 'svelte-spa-router'
  import { Button } from '@pagelens/ui/shadcn/button'
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle
  } from '@pagelens/ui/shadcn/card'
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '@pagelens/ui/shadcn/tabs'
  import { Badge } from '@pagelens/ui/shadcn/badge'
  import { Separator } from '@pagelens/ui/shadcn/separator'
  import { ScrollArea } from '@pagelens/ui/shadcn/scroll-area'
  import { toast } from 'svelte-sonner'
  import {
    ArrowLeft,
    AlertCircle,
    Clock,
    Globe,
    FileText,
    Gauge,
    AlertTriangle,
    Image,
    Code,
    Heading,
    Share2,
    Timer,
    FileJson,
    Loader2,
    ExternalLink
  } from '@lucide/svelte'
  import { commands, type AnalysisRun } from '$lib/ipc'
  import type {
    SinglePagePayload,
    CrawlResult,
    SeoReport,
    PerformanceTiming
  } from '$lib/types'
  import ScoreGauge from '$lib/components/report/ScoreGauge.svelte'
  import IssuesTable from '$lib/components/report/IssuesTable.svelte'
  import MetaDetails from '$lib/components/report/MetaDetails.svelte'
  import SocialMeta from '$lib/components/report/SocialMeta.svelte'
  import HeadingStructure from '$lib/components/report/HeadingStructure.svelte'
  import ImagesTable from '$lib/components/report/ImagesTable.svelte'
  import PerformanceTimeline from '$lib/components/report/PerformanceTimeline.svelte'
  import CrawlOverview from '$lib/components/report/CrawlOverview.svelte'
  import CrawlPagesTable from '$lib/components/report/CrawlPagesTable.svelte'

  let { params = {} }: { params?: { id?: string } } = $props()

  let run = $state<AnalysisRun | null>(null)
  let isLoading = $state(true)
  let error = $state<string | null>(null)

  // Parsed payload
  let singlePayload = $state<SinglePagePayload | null>(null)
  let crawlPayload = $state<CrawlResult | null>(null)
  let showRawJson = $state(false)

  onMount(async () => {
    const id = params.id
    if (!id) {
      error = 'No report ID provided'
      isLoading = false
      return
    }

    try {
      const result = await commands.getHistoryItem(id)
      if (result.status === 'error') {
        error = result.error
      } else {
        run = result.data
        parsePayload()
      }
    } catch (err) {
      error = err instanceof Error ? err.message : String(err)
    } finally {
      isLoading = false
    }
  })

  function parsePayload() {
    if (!run) return
    try {
      const data = JSON.parse(run.payload_json)
      if (run.analysis_type === 'single') {
        singlePayload = data as SinglePagePayload
      } else {
        crawlPayload = data as CrawlResult
      }
    } catch {
      error = 'Failed to parse analysis payload'
    }
  }

  function formatDate(iso: string): string {
    return new Date(iso).toLocaleString()
  }

  function formatDuration(ms: number): string {
    if (ms < 1000) return `${ms}ms`
    return `${(ms / 1000).toFixed(1)}s`
  }

  const seoReport = $derived(
    singlePayload?.seo_report ?? null
  )

  const perfTiming = $derived(
    singlePayload?.snapshot?.performance_timing ?? null
  )
</script>

<div class="flex flex-col h-full overflow-auto">
  {#if isLoading}
    <div class="flex flex-1 items-center justify-center">
      <Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
    </div>
  {:else if error}
    <div class="flex flex-1 flex-col items-center justify-center gap-4 p-6">
      <AlertCircle class="h-12 w-12 text-destructive" />
      <p class="text-lg font-medium">Error loading report</p>
      <p class="text-sm text-muted-foreground">{error}</p>
      <Button variant="outline" onclick={() => push('/')}>
        <ArrowLeft class="mr-2 h-4 w-4" />
        Back to Dashboard
      </Button>
    </div>
  {:else if run}
    <div class="p-6 space-y-6">
      <!-- Header -->
      <div class="flex items-start justify-between gap-4">
        <div class="flex items-center gap-4">
          <Button variant="ghost" size="icon" onclick={() => history.back()}>
            <ArrowLeft class="h-4 w-4" />
          </Button>
          <div>
            <h1 class="text-2xl font-bold tracking-tight">
              {run.name || 'Analysis Report'}
            </h1>
            <div class="flex items-center gap-3 mt-1 text-sm text-muted-foreground">
              <a
                href={run.url}
                target="_blank"
                rel="noreferrer noopener"
                class="flex items-center gap-1 hover:text-foreground truncate max-w-md"
              >
                {run.url}
                <ExternalLink class="h-3 w-3 shrink-0" />
              </a>
              <span class="flex items-center gap-1">
                <Clock class="h-3 w-3" />
                {formatDate(run.created_at)}
              </span>
            </div>
          </div>
        </div>
        <Badge variant={run.analysis_type === 'single' ? 'default' : 'secondary'}>
          {#if run.analysis_type === 'single'}
            <FileText class="mr-1 h-3 w-3" />
            Single Page
          {:else}
            <Globe class="mr-1 h-3 w-3" />
            Crawl
          {/if}
        </Badge>
      </div>

      <!-- ============================================================ -->
      <!-- Single Page Report -->
      <!-- ============================================================ -->
      {#if run.analysis_type === 'single' && seoReport}
        <!-- Score + Summary Cards -->
        <div class="flex flex-col items-center gap-4 sm:flex-row sm:items-start">
          <ScoreGauge score={seoReport.score} />
          <div class="grid flex-1 grid-cols-2 gap-3 sm:grid-cols-4">
            <Card>
              <CardContent class="pt-4 text-center">
                <Gauge class="mx-auto mb-1 h-5 w-5 text-indigo-500" />
                <p class="text-2xl font-bold">
                  {Math.round(seoReport.score)}
                </p>
                <p class="text-xs text-muted-foreground">Score</p>
              </CardContent>
            </Card>
            <Card>
              <CardContent class="pt-4 text-center">
                <AlertCircle class="mx-auto mb-1 h-5 w-5 text-red-500" />
                <p class="text-2xl font-bold">
                  {seoReport.issues.filter(i => i.severity === 'Error').length}
                </p>
                <p class="text-xs text-muted-foreground">Errors</p>
              </CardContent>
            </Card>
            <Card>
              <CardContent class="pt-4 text-center">
                <AlertTriangle class="mx-auto mb-1 h-5 w-5 text-amber-500" />
                <p class="text-2xl font-bold">
                  {seoReport.issues.filter(i => i.severity === 'Warning').length}
                </p>
                <p class="text-xs text-muted-foreground">Warnings</p>
              </CardContent>
            </Card>
            <Card>
              <CardContent class="pt-4 text-center">
                <Clock class="mx-auto mb-1 h-5 w-5 text-muted-foreground" />
                <p class="text-2xl font-bold">
                  {formatDuration(run.summary.duration_ms)}
                </p>
                <p class="text-xs text-muted-foreground">Duration</p>
              </CardContent>
            </Card>
          </div>
        </div>

        <Separator />

        <!-- Tabs -->
        <Tabs value="issues">
          <TabsList>
            <TabsTrigger value="issues">
              <AlertCircle class="mr-1.5 h-3.5 w-3.5" />
              Issues ({seoReport.issues.length})
            </TabsTrigger>
            <TabsTrigger value="meta">
              <FileText class="mr-1.5 h-3.5 w-3.5" />
              Meta
            </TabsTrigger>
            <TabsTrigger value="social">
              <Share2 class="mr-1.5 h-3.5 w-3.5" />
              Social
            </TabsTrigger>
            <TabsTrigger value="headings">
              <Heading class="mr-1.5 h-3.5 w-3.5" />
              Headings
            </TabsTrigger>
            <TabsTrigger value="images">
              <Image class="mr-1.5 h-3.5 w-3.5" />
              Images ({seoReport.images.length})
            </TabsTrigger>
            {#if seoReport.structured_data.length > 0}
              <TabsTrigger value="structured">
                <Code class="mr-1.5 h-3.5 w-3.5" />
                Schema
              </TabsTrigger>
            {/if}
            {#if perfTiming}
              <TabsTrigger value="performance">
                <Timer class="mr-1.5 h-3.5 w-3.5" />
                Performance
              </TabsTrigger>
            {/if}
            <TabsTrigger value="raw">
              <FileJson class="mr-1.5 h-3.5 w-3.5" />
              Raw
            </TabsTrigger>
          </TabsList>

          <TabsContent value="issues" class="mt-4">
            <Card>
              <CardContent class="pt-4">
                <IssuesTable issues={seoReport.issues} />
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="meta" class="mt-4">
            <Card>
              <CardContent class="pt-4">
                <MetaDetails
                  meta={seoReport.meta}
                  canonical={seoReport.canonical_url}
                />
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="social" class="mt-4">
            <SocialMeta
              og={seoReport.open_graph}
              twitter={seoReport.twitter_card}
            />
          </TabsContent>

          <TabsContent value="headings" class="mt-4">
            <Card>
              <CardContent class="pt-4">
                <HeadingStructure headings={seoReport.headings} />
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="images" class="mt-4">
            <Card>
              <CardContent class="pt-4">
                <ImagesTable images={seoReport.images} />
              </CardContent>
            </Card>
          </TabsContent>

          {#if seoReport.structured_data.length > 0}
            <TabsContent value="structured" class="mt-4">
              <Card>
                <CardContent class="pt-4 space-y-4">
                  {#each seoReport.structured_data as sd}
                    <div class="space-y-1">
                      <Badge variant="outline">{sd.schema_type}</Badge>
                      <pre class="mt-1 rounded-md bg-muted p-3 text-xs overflow-auto max-h-48"><code>{(() => {
                          try { return JSON.stringify(JSON.parse(sd.raw), null, 2) }
                          catch { return sd.raw }
                        })()}</code></pre>
                    </div>
                  {/each}
                </CardContent>
              </Card>
            </TabsContent>
          {/if}

          {#if perfTiming}
            <TabsContent value="performance" class="mt-4">
              <Card>
                <CardContent class="pt-4">
                  <PerformanceTimeline timing={perfTiming} />
                </CardContent>
              </Card>
            </TabsContent>
          {/if}

          <TabsContent value="raw" class="mt-4">
            <Card>
              <CardContent class="pt-4">
                <pre class="rounded-md bg-muted p-4 text-xs overflow-auto max-h-[60vh]"><code>{JSON.stringify(
                    JSON.parse(run.payload_json),
                    null,
                    2
                  )}</code></pre>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>

      <!-- ============================================================ -->
      <!-- Crawl Report -->
      <!-- ============================================================ -->
      {:else if run.analysis_type === 'crawl' && crawlPayload}
        <CrawlOverview
          stats={crawlPayload.stats}
          aggregate={crawlPayload.aggregate}
        />

        <Separator />

        <Tabs value="pages">
          <TabsList>
            <TabsTrigger value="pages">
              <Globe class="mr-1.5 h-3.5 w-3.5" />
              Pages ({crawlPayload.pages.length})
            </TabsTrigger>
            <TabsTrigger value="issues">
              <AlertCircle class="mr-1.5 h-3.5 w-3.5" />
              All Issues
            </TabsTrigger>
            {#if crawlPayload.skipped_urls.length > 0}
              <TabsTrigger value="skipped">
                Skipped ({crawlPayload.skipped_urls.length})
              </TabsTrigger>
            {/if}
            <TabsTrigger value="raw">
              <FileJson class="mr-1.5 h-3.5 w-3.5" />
              Raw
            </TabsTrigger>
          </TabsList>

          <TabsContent value="pages" class="mt-4">
            <Card>
              <CardContent class="pt-4">
                <CrawlPagesTable pages={crawlPayload.pages} />
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="issues" class="mt-4">
            <Card>
              <CardContent class="pt-4">
                {@const allIssues = crawlPayload.pages.flatMap(p =>
                  p.seo_report.issues.map(i => ({
                    ...i,
                    url: p.url
                  }))
                )}
                <IssuesTable issues={allIssues} />
              </CardContent>
            </Card>
          </TabsContent>

          {#if crawlPayload.skipped_urls.length > 0}
            <TabsContent value="skipped" class="mt-4">
              <Card>
                <CardContent class="pt-4">
                  <div class="space-y-1 max-h-96 overflow-auto">
                    {#each crawlPayload.skipped_urls as url}
                      <p class="text-xs font-mono text-muted-foreground truncate">
                        {url}
                      </p>
                    {/each}
                  </div>
                </CardContent>
              </Card>
            </TabsContent>
          {/if}

          <TabsContent value="raw" class="mt-4">
            <Card>
              <CardContent class="pt-4">
                <pre class="rounded-md bg-muted p-4 text-xs overflow-auto max-h-[60vh]"><code>{JSON.stringify(
                    JSON.parse(run.payload_json),
                    null,
                    2
                  )}</code></pre>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      {:else}
        <Card class="border-dashed">
          <CardContent class="flex flex-col items-center py-12 text-center">
            <AlertCircle class="h-12 w-12 text-muted-foreground mb-4" />
            <p class="text-lg font-semibold">Unable to parse report data</p>
          </CardContent>
        </Card>
      {/if}
    </div>
  {/if}
</div>
