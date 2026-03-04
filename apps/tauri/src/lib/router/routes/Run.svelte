<script lang="ts">
  import { onDestroy, onMount } from 'svelte'
  import { push } from 'svelte-spa-router'
  import { toast } from 'svelte-sonner'
  import { Button } from '@pagelens/ui/shadcn/button'
  import { Card, CardContent, CardHeader, CardTitle } from '@pagelens/ui/shadcn/card'
  import { Badge } from '@pagelens/ui/shadcn/badge'
  import { Progress } from '@pagelens/ui/shadcn/progress'
  import { Separator } from '@pagelens/ui/shadcn/separator'
  import { ScrollArea } from '@pagelens/ui/shadcn/scroll-area'
  import {
    Loader2,
    AlertCircle,
    CheckCircle2,
    Clock,
    Gauge,
    Globe,
    FileText,
    RotateCw
  } from '@lucide/svelte'
  import { commands, events, type AnalysisRun } from '$lib/ipc'

  type RunStatus = 'pending' | 'running' | 'completed' | 'failed'

  type RunWithLive = AnalysisRun & {
    status?: RunStatus
    current_stage?: string | null
    current_message?: string | null
    progress?: number | null
  }

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

  type ProgressPayload = {
    run_id: string
    stage: string
    message: string
    progress: number | null
    page_count: number | null
    success_count: number | null
    failed_count: number | null
  }

  type PagePayload = {
    run_id: string
    page: PageRow
  }

  let { params = {} }: { params?: { id?: string } } = $props()

  let run = $state<RunWithLive | null>(null)
  let pages = $state<PageRow[]>([])
  let progress = $state(0)
  let stage = $state('pending')
  let message = $state('Waiting for analysis to start...')
  let error = $state<string | null>(null)
  let isLoading = $state(true)
  let isRerunning = $state(false)
  let refetchState = $state<
    Record<string, { runId: string; label: string; completed: boolean }>
  >({})
  const refetchUnsubs: Record<string, () => void> = {}

  let unlistenProgress: (() => void) | null = null
  let unlistenPage: (() => void) | null = null
  let pollId: ReturnType<typeof setInterval> | null = null

  const totalIssues = $derived(
    pages.reduce((sum, page) => sum + page.total_issues, 0)
  )
  const totalErrors = $derived(
    pages.reduce((sum, page) => sum + page.error_count, 0)
  )
  const avgScore = $derived(() => {
    const scored = pages.filter(page => page.seo_score != null)
    if (scored.length === 0) return null
    return (
      scored.reduce((sum, page) => sum + (page.seo_score ?? 0), 0) /
      scored.length
    )
  })

  const scoreBuckets = $derived(() => {
    let b0 = 0
    let b1 = 0
    let b2 = 0
    let b3 = 0
    let b4 = 0
    for (const page of pages) {
      if (page.seo_score == null) continue
      const score = Math.round(page.seo_score)
      if (score <= 20) b0 += 1
      else if (score <= 40) b1 += 1
      else if (score <= 60) b2 += 1
      else if (score <= 80) b3 += 1
      else b4 += 1
    }
    return [b0, b1, b2, b3, b4]
  })

  function runStatus(): RunStatus {
    return (
      run?.status ??
      (stage === 'failed'
        ? 'failed'
        : progress >= 100
          ? 'completed'
          : 'running')
    )
  }

  function runInProgress(): boolean {
    const status = runStatus()
    return status === 'running' || status === 'pending'
  }

  function mergePage(incoming: PageRow) {
    const index = pages.findIndex(page => page.id === incoming.id)
    if (index >= 0) {
      const next = [...pages]
      next[index] = incoming
      pages = next
      return
    }
    pages = [...pages, incoming]
  }

  async function refresh() {
    const runId = params.id
    if (!runId) {
      error = 'No run id provided'
      isLoading = false
      return
    }

    try {
      const runResult = await commands.getHistoryItem(runId)
      if (runResult.status === 'error') {
        throw new Error(runResult.error)
      }
      const liveRun = runResult.data as RunWithLive
      run = liveRun
      progress = (liveRun.progress ?? 0) * 100
      stage = liveRun.current_stage ?? stage
      message = liveRun.current_message ?? message

      const pagesResult = await commands.listAnalysisPageResults(runId)
      if (pagesResult.status === 'ok') {
        pages = pagesResult.data as PageRow[]
      }
      error = null
    } catch (err) {
      error = err instanceof Error ? err.message : String(err)
    } finally {
      isLoading = false
    }
  }

  onMount(async () => {
    await refresh()

    unlistenProgress = await events.analysisProgressEvent.listen(event => {
      if (event.payload.run_id !== params.id) return
      stage = event.payload.stage
      message = event.payload.message
      progress =
        event.payload.progress != null ? event.payload.progress * 100 : progress
    })

    unlistenPage = await events.analysisPageEvent.listen(event => {
      if (event.payload.run_id !== params.id) return
      mergePage(event.payload.page)
    })

    pollId = setInterval(async () => {
      await refresh()
      const status = runStatus()
      if (status === 'completed' || status === 'failed') {
        if (pollId) {
          clearInterval(pollId)
          pollId = null
        }
      }
    }, 1500)
  })

  onDestroy(() => {
    if (unlistenProgress) unlistenProgress()
    if (unlistenPage) unlistenPage()
    if (pollId) clearInterval(pollId)
    for (const unsubscribe of Object.values(refetchUnsubs)) {
      unsubscribe()
    }
  })

  function formatDate(iso: string) {
    return new Date(iso).toLocaleString()
  }

  function formatDuration(ms: number) {
    if (ms < 1000) return `${ms}ms`
    return `${(ms / 1000).toFixed(1)}s`
  }

  function normalizedHost(rawUrl: string): string | null {
    try {
      const parsed = new URL(rawUrl)
      return parsed.hostname.replace(/^www\./i, '').toLowerCase()
    } catch {
      return null
    }
  }

  function isTargetWebsite(pageUrl: string): boolean {
    if (!run?.url) return true
    const runHost = normalizedHost(run.url)
    const pageHost = normalizedHost(pageUrl)
    if (!runHost || !pageHost) return true
    return pageHost === runHost || pageHost.endsWith(`.${runHost}`)
  }

  function displayPath(rawUrl: string): string {
    try {
      const parsed = new URL(rawUrl)
      const pathname = parsed.pathname || '/'
      return `${pathname}${parsed.search}`
    } catch {
      return rawUrl
    }
  }

  function sourceHoverText(pageUrl: string): string {
    const runHost = run?.url ? normalizedHost(run.url) : null
    const pageHost = normalizedHost(pageUrl)
    const source = isTargetWebsite(pageUrl)
      ? 'Target website'
      : 'External website'
    return `${source}\nTarget host: ${runHost ?? 'unknown'}\nPage host: ${pageHost ?? 'unknown'}\nFull URL: ${pageUrl}`
  }

  function progressLabel(
    stageName: string,
    progressValue: number | null
  ): string {
    const stage = stageName.charAt(0).toUpperCase() + stageName.slice(1)
    const percent =
      progressValue != null ? ` ${Math.round(progressValue * 100)}%` : ''
    return `${stage}${percent}`
  }

  function buttonLabel(pageId: string): string {
    return refetchState[pageId]?.label ?? 'Refetch'
  }

  function buttonDisabled(pageId: string): boolean {
    const current = refetchState[pageId]
    return current != null && !current.completed
  }

  async function refetchPage(page: PageRow, event: MouseEvent) {
    event.stopPropagation()

    const existing = refetchState[page.id]
    if (existing?.completed) {
      push(`/run/${existing.runId}`)
      return
    }
    if (existing && !existing.completed) return

    const runId = crypto.randomUUID()
    refetchState = {
      ...refetchState,
      [page.id]: { runId, label: 'Launching 0%', completed: false }
    }

    const unlisten = await events.analysisProgressEvent.listen(
      progressEvent => {
        if (progressEvent.payload.run_id !== runId) return

        const stageName = progressEvent.payload.stage
        const inProgressLabel = progressLabel(
          stageName,
          progressEvent.payload.progress
        )
        const isDone = stageName === 'complete'
        const isFailed = stageName === 'failed'

        refetchState = {
          ...refetchState,
          [page.id]: {
            runId,
            label: isDone
              ? 'Open Result'
              : isFailed
                ? 'Failed'
                : inProgressLabel,
            completed: isDone
          }
        }

        if (isDone || isFailed) {
          refetchUnsubs[page.id]?.()
          delete refetchUnsubs[page.id]
        }
      }
    )
    refetchUnsubs[page.id] = unlisten

    try {
      const result = await commands.analyzeUrl({
        run_id: runId,
        url: page.url,
        name: `Refetch ${displayPath(page.url)}`,
        options: null
      })

      if (result.status === 'error') {
        throw new Error(result.error)
      }
      toast.success('Refetch complete')
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err)
      refetchState = {
        ...refetchState,
        [page.id]: { runId, label: 'Failed', completed: false }
      }
      toast.error(`Refetch failed: ${msg}`)
    } finally {
      const unsubscribe = refetchUnsubs[page.id]
      if (unsubscribe) unsubscribe()
      delete refetchUnsubs[page.id]
    }
  }

  async function rerunAnalysis() {
    if (!run || isRerunning) return

    isRerunning = true
    const runId = crypto.randomUUID()
    push(`/run/${runId}`)

    try {
      const parsedPayload = JSON.parse(run.payload_json || '{}') as Record<
        string,
        unknown
      >

      let result:
        | Awaited<ReturnType<typeof commands.analyzeUrl>>
        | Awaited<ReturnType<typeof commands.crawlUrl>>
        | Awaited<ReturnType<typeof commands.analyzeSiteFiles>>

      if (run.analysis_type === 'crawl') {
        result = await commands.crawlUrl({
          run_id: runId,
          url: run.url,
          name: run.name,
          options: null
        })
      } else if ('site_files_report' in parsedPayload) {
        result = await commands.analyzeSiteFiles({
          run_id: runId,
          url: run.url,
          crawled_urls: null
        })
      } else {
        result = await commands.analyzeUrl({
          run_id: runId,
          url: run.url,
          name: run.name,
          options: null
        })
      }

      if (result.status === 'error') {
        throw new Error(result.error)
      }
      toast.success('Re-run complete')
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err)
      toast.error(`Re-run failed: ${msg}`)
    } finally {
      isRerunning = false
    }
  }
</script>

<div class="h-full overflow-auto p-6 space-y-6">
  <div class="flex items-center justify-between gap-3">
    <div>
      <h1 class="text-3xl font-bold tracking-tight">Analysis Results</h1>
      <p class="text-sm text-muted-foreground">Run ID: {params.id}</p>
    </div>
    <div class="flex items-center gap-2">
      <Badge
        variant={runStatus() === 'completed'
          ? 'default'
          : runStatus() === 'failed'
            ? 'destructive'
            : 'secondary'}
      >
        {runStatus()}
      </Badge>
      <Button variant="outline" onclick={rerunAnalysis} disabled={isRerunning}
        >{#if isRerunning}<Loader2
            class="mr-1 h-4 w-4 animate-spin"
          />{:else}<RotateCw class="mr-1 h-4 w-4" />{/if}Re-run Analysis</Button
      >
      <Button variant="outline" onclick={() => push('/#history')}
        >History</Button
      >
    </div>
  </div>

  {#if runInProgress()}
    <Card>
      <CardHeader>
        <CardTitle class="flex items-center gap-2">
          {#if runStatus() === 'running'}
            <Loader2 class="h-5 w-5 animate-spin" />
          {:else if runStatus() === 'completed'}
            <CheckCircle2 class="h-5 w-5 text-green-500" />
          {:else if runStatus() === 'failed'}
            <AlertCircle class="h-5 w-5 text-red-500" />
          {:else}
            <Clock class="h-5 w-5" />
          {/if}
          Real-time progress
        </CardTitle>
      </CardHeader>
      <CardContent class="space-y-3">
        <div class="flex justify-between text-sm">
          <span class="font-medium capitalize">{stage}</span>
          <span class="text-muted-foreground">{Math.round(progress)}%</span>
        </div>
        <Progress value={progress} class="h-2" />
        <p class="text-sm text-muted-foreground">{message}</p>
      </CardContent>
    </Card>
  {/if}

  <div class="grid gap-4 md:grid-cols-4">
    <Card
      ><CardContent class="pt-5"
        ><div class="text-xs text-muted-foreground">Pages analyzed</div>
        <div class="text-2xl font-bold">{pages.length}</div></CardContent
      ></Card
    >
    <Card
      ><CardContent class="pt-5"
        ><div class="text-xs text-muted-foreground">Average SEO score</div>
        <div class="text-2xl font-bold">
          {avgScore() == null ? '-' : Math.round(avgScore()!)}
        </div></CardContent
      ></Card
    >
    <Card
      ><CardContent class="pt-5"
        ><div class="text-xs text-muted-foreground">Total issues</div>
        <div class="text-2xl font-bold">{totalIssues}</div></CardContent
      ></Card
    >
    <Card
      ><CardContent class="pt-5"
        ><div class="text-xs text-muted-foreground">Errors</div>
        <div class="text-2xl font-bold">{totalErrors}</div></CardContent
      ></Card
    >
  </div>

  <div class="grid gap-4 lg:grid-cols-2">
    <Card>
      <CardHeader
        ><CardTitle class="text-base">SEO Score distribution</CardTitle
        ></CardHeader
      >
      <CardContent class="space-y-2">
        {#each scoreBuckets() as count, i}
          <div class="space-y-1">
            <div class="flex justify-between text-xs text-muted-foreground">
              <span>{i * 20 + (i === 0 ? 0 : 1)}-{(i + 1) * 20}</span>
              <span>{count}</span>
            </div>
            <div class="h-2 rounded bg-muted overflow-hidden">
              <div
                class="h-full bg-indigo-500"
                style={`width: ${pages.length > 0 ? (count / pages.length) * 100 : 0}%`}
              ></div>
            </div>
          </div>
        {/each}
      </CardContent>
    </Card>

    <Card>
      <CardHeader
        ><CardTitle class="text-base">Run summary</CardTitle></CardHeader
      >
      <CardContent class="space-y-2 text-sm">
        <div class="flex justify-between">
          <span class="text-muted-foreground">URL</span><span
            class="truncate max-w-[60%] text-right">{run?.url ?? '-'}</span
          >
        </div>
        <div class="flex justify-between">
          <span class="text-muted-foreground">Type</span><span
            class="flex items-center gap-1"
            >{#if run?.analysis_type === 'single'}<FileText
                class="h-3.5 w-3.5"
              />Single{:else}<Globe class="h-3.5 w-3.5" />Crawl{/if}</span
          >
        </div>
        <div class="flex justify-between">
          <span class="text-muted-foreground">Created</span><span
            >{run ? formatDate(run.created_at) : '-'}</span
          >
        </div>
        <div class="flex justify-between">
          <span class="text-muted-foreground">Duration</span><span
            >{run ? formatDuration(run.summary.duration_ms) : '-'}</span
          >
        </div>
        <div class="flex justify-between">
          <span class="text-muted-foreground">Score</span><span
            class="flex items-center gap-1"
            ><Gauge class="h-3.5 w-3.5" />{run?.summary.seo_score == null
              ? '-'
              : Math.round(run.summary.seo_score)}</span
          >
        </div>
      </CardContent>
    </Card>
  </div>

  <Separator />

  <Card>
    <CardHeader>
      <CardTitle>Analyzed pages (streaming)</CardTitle>
    </CardHeader>
    <CardContent>
      {#if isLoading}
        <div class="py-8 text-center text-muted-foreground">
          Loading run data...
        </div>
      {:else if error}
        <div class="py-8 text-center text-red-500">{error}</div>
      {:else if pages.length === 0}
        <div class="py-8 text-center text-muted-foreground">
          No pages processed yet.
        </div>
      {:else}
        <ScrollArea class="h-[420px] rounded border">
          <table class="w-full text-sm">
            <thead class="sticky top-0 bg-background border-b">
              <tr class="text-left">
                <th class="px-3 py-2 w-[120px]">Source</th>
                <th class="px-3 py-2">Path</th>
                <th class="px-3 py-2">Depth</th>
                <th class="px-3 py-2">Status</th>
                <th class="px-3 py-2">Score</th>
                <th class="px-3 py-2">Issues</th>
                <th class="px-3 py-2">Analyzed at</th>
                <th class="px-3 py-2 w-[140px]">Actions</th>
              </tr>
            </thead>
            <tbody>
              {#each pages as page (page.id)}
                <tr
                  class="border-b last:border-b-0 cursor-pointer hover:bg-muted/40"
                  onclick={() => push(`/run/${params.id}/page/${page.id}`)}
                >
                  <td class="px-3 py-2" title={sourceHoverText(page.url)}>
                    <Badge
                      variant={isTargetWebsite(page.url)
                        ? 'default'
                        : 'secondary'}
                    >
                      {isTargetWebsite(page.url) ? 'T' : 'E'}
                    </Badge>
                  </td>
                  <td class="px-3 py-2 max-w-[420px] truncate" title={page.url}>
                    {displayPath(page.url)}
                  </td>
                  <td class="px-3 py-2">{page.depth}</td>
                  <td class="px-3 py-2">
                    <Badge variant={page.success ? 'default' : 'destructive'}
                      >{page.success ? 'ok' : 'failed'}</Badge
                    >
                  </td>
                  <td class="px-3 py-2"
                    >{page.seo_score == null
                      ? '-'
                      : Math.round(page.seo_score)}</td
                  >
                  <td class="px-3 py-2">{page.total_issues}</td>
                  <td class="px-3 py-2 text-muted-foreground"
                    >{formatDate(page.analyzed_at)}</td
                  >
                  <td class="px-3 py-2">
                    <Button
                      variant="outline"
                      size="sm"
                      onclick={event => refetchPage(page, event)}
                      disabled={buttonDisabled(page.id)}
                    >
                      {#if buttonDisabled(page.id)}
                        <Loader2 class="mr-1 h-3.5 w-3.5 animate-spin" />
                      {:else}
                        <RotateCw class="mr-1 h-3.5 w-3.5" />
                      {/if}
                      {buttonLabel(page.id)}
                    </Button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </ScrollArea>
      {/if}
    </CardContent>
  </Card>
</div>
