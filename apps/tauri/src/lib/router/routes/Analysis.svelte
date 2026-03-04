<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { push } from 'svelte-spa-router'
  import { toast } from 'svelte-sonner'
  import {
    Tabs,
    TabsContent,
    TabsList,
    TabsTrigger
  } from '@pagelens/ui/shadcn/tabs'
  import { Separator } from '@pagelens/ui/shadcn/separator'
  import { FileText, Globe, Layers } from '@lucide/svelte'
  import { Scan } from '@lucide/svelte'
  import {
    PageHeader,
    AnalysisForm,
    AnalysisProgress,
    ResultsPreview,
    SiteFilesReportCard,
    EmptyState,
    analysisStore,
    createSingleOptionsStore,
    createCrawlOptionsStore,
    DEFAULT_SINGLE_OPTIONS,
    DEFAULT_CRAWL_OPTIONS
  } from '@pagelens/ui'
  import { commands, events, type AnalysisRun } from '$lib/ipc'
  import type { SiteFilesReport as SiteFilesReportType } from '$lib/types'

  // Local state
  const singleOptions = createSingleOptionsStore(DEFAULT_SINGLE_OPTIONS)
  const crawlOptions = createCrawlOptionsStore(DEFAULT_CRAWL_OPTIONS)

  let activeTab = $state('single')
  let lastResult = $state<AnalysisRun | null>(null)
  let siteFilesReport = $state<SiteFilesReportType | null>(null)
  let siteFilesError = $state<string | null>(null)
  let unlistenProgress: (() => void) | null = null

  onMount(async () => {
    unlistenProgress = await events.analysisProgressEvent.listen(event => {
      if (event.payload.run_id !== $analysisStore.runId) return

      analysisStore.updateProgress({
        stage: event.payload.stage,
        message: event.payload.message,
        progress: event.payload.progress ? event.payload.progress * 100 : 0
      })

      if (event.payload.stage === 'complete') {
        analysisStore.complete()
      } else if (event.payload.stage === 'failed') {
        analysisStore.fail(event.payload.message)
      }
    })
  })

  onDestroy(() => {
    if (unlistenProgress) unlistenProgress()
  })

  async function handleSingleSubmit() {
    await startAnalysis('single', () =>
      commands.analyzeUrl({
        run_id: $analysisStore.runId!,
        url: $analysisStore.url.trim(),
        name: $analysisStore.name.trim() || null,
        options: {
          include_html: $singleOptions.includeHtml,
          include_accessibility_tree: $singleOptions.includeAccessibilityTree,
          include_performance_timing: $singleOptions.includePerformanceTiming,
          include_computed_styles: $singleOptions.includeComputedStyles
        }
      })
    )
  }

  async function handleCrawlSubmit() {
    await startAnalysis('crawl', () =>
      commands.crawlUrl({
        run_id: $analysisStore.runId!,
        url: $analysisStore.url.trim(),
        name: $analysisStore.name.trim() || null,
        options: {
          max_pages: $crawlOptions.maxPages,
          max_depth: $crawlOptions.maxDepth,
          follow_external_links: $crawlOptions.followExternalLinks,
          same_subdomain_only: $crawlOptions.sameSubdomainOnly,
          page_timeout_ms: $crawlOptions.pageTimeoutMs,
          delay_ms: $crawlOptions.delayMs,
          max_concurrency: $crawlOptions.maxConcurrency
        }
      })
    )
  }

  async function handleSiteFilesSubmit() {
    siteFilesError = null
    siteFilesReport = null

    const runId = crypto.randomUUID()
    analysisStore.start(runId)
    push(`/run/${runId}`)

    try {
      const result = await commands.analyzeSiteFiles({
        run_id: runId,
        url: $analysisStore.url.trim(),
        crawled_urls: null
      })

      if (result.status === 'error') {
        throw new Error(result.error)
      }

      analysisStore.complete()
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err)
      siteFilesError = message
      analysisStore.fail(message)
      toast.error(`Site files analysis failed: ${message}`)
    }
  }

  async function startAnalysis(
    type: 'single' | 'crawl',
    command: () => Promise<{
      status: 'ok' | 'error'
      data?: unknown
      error?: string
    }>
  ) {
    const runId = crypto.randomUUID()
    analysisStore.start(runId)
    push(`/run/${runId}`)

    try {
      const result = await command()

      if (result.status === 'error') {
        throw new Error(result.error)
      }

      if (
        result.data &&
        typeof result.data === 'object' &&
        'id' in result.data
      ) {
        lastResult = result.data as AnalysisRun
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err)
      analysisStore.fail(message)
      toast.error(
        `${type === 'single' ? 'Analysis' : 'Crawl'} failed: ${message}`
      )
    }
  }

  function formatDuration(ms: number): string {
    if (ms < 1000) return `${ms}ms`
    return `${(ms / 1000).toFixed(1)}s`
  }
</script>

<div class="h-full flex flex-col p-6 gap-6 overflow-auto">
  <PageHeader
    title="Analysis"
    description="Analyze single pages or crawl entire sites"
    icon={Scan}
  />

  <Separator />

  <div class="grid gap-6 lg:grid-cols-2">
    <!-- Input Section -->
    <div class="space-y-6">
      <Tabs bind:value={activeTab} class="w-full">
        <TabsList class="grid w-full grid-cols-3">
          <TabsTrigger value="single">
            <FileText class="mr-2 h-4 w-4" />
            Single Page
          </TabsTrigger>
          <TabsTrigger value="crawl">
            <Globe class="mr-2 h-4 w-4" />
            Crawl
          </TabsTrigger>
          <TabsTrigger value="sitefiles">
            <Layers class="mr-2 h-4 w-4" />
            Site Files
          </TabsTrigger>
        </TabsList>

        <TabsContent value="single" class="space-y-4">
          <AnalysisForm
            type="single"
            url={$analysisStore.url}
            name={$analysisStore.name}
            singleOptions={$singleOptions}
            crawlOptions={$crawlOptions}
            isSubmitting={$analysisStore.status === 'running'}
            onSubmit={handleSingleSubmit}
            onUrlChange={analysisStore.setUrl}
            onNameChange={analysisStore.setName}
            onSingleOptionChange={(key, value) =>
              singleOptions.update(s => ({ ...s, [key]: value }))}
            onCrawlOptionChange={() => {}}
          />
        </TabsContent>

        <TabsContent value="crawl" class="space-y-4">
          <AnalysisForm
            type="crawl"
            url={$analysisStore.url}
            name={$analysisStore.name}
            singleOptions={$singleOptions}
            crawlOptions={$crawlOptions}
            isSubmitting={$analysisStore.status === 'running'}
            onSubmit={handleCrawlSubmit}
            onUrlChange={analysisStore.setUrl}
            onNameChange={analysisStore.setName}
            onSingleOptionChange={() => {}}
            onCrawlOptionChange={(key, value) =>
              crawlOptions.update(s => ({ ...s, [key]: value }))}
          />
        </TabsContent>

        <TabsContent value="sitefiles" class="space-y-4">
          <AnalysisForm
            type="site_files"
            url={$analysisStore.url}
            name={$analysisStore.name}
            singleOptions={$singleOptions}
            crawlOptions={$crawlOptions}
            isSubmitting={$analysisStore.status === 'running'}
            onSubmit={handleSiteFilesSubmit}
            onUrlChange={analysisStore.setUrl}
            onNameChange={analysisStore.setName}
            onSingleOptionChange={() => {}}
            onCrawlOptionChange={() => {}}
          />

          {#if siteFilesError}
            <p class="text-sm text-red-500">{siteFilesError}</p>
          {/if}

          {#if siteFilesReport}
            <Separator />
            <SiteFilesReportCard report={siteFilesReport} />
          {/if}
        </TabsContent>
      </Tabs>
    </div>

    <!-- Results Section -->
    <div class="space-y-6">
      <AnalysisProgress
        status={$analysisStore.status}
        stage={$analysisStore.progress.stage}
        message={$analysisStore.progress.message}
        progress={$analysisStore.progress.progress}
      />

      {#if lastResult}
        <ResultsPreview
          name={lastResult.name}
          createdAt={lastResult.created_at}
          analysisType={lastResult.analysis_type}
          summary={lastResult.summary}
          onViewDetails={() => push(`/run/${lastResult!.id}`)}
        />
      {:else if $analysisStore.status === 'idle'}
        <EmptyState
          title="No Analysis Yet"
          description="Enter a URL and click Analyze to start. Results will appear here."
          icon={Scan}
        />
      {/if}
    </div>
  </div>
</div>
