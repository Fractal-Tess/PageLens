<script lang="ts">
  import { onMount } from 'svelte'
  import { push } from 'svelte-spa-router'
  import { Button } from '@pagelens/ui/shadcn/button'
  import { Tabs, TabsList, TabsTrigger } from '@pagelens/ui/shadcn/tabs'
  import { Separator } from '@pagelens/ui/shadcn/separator'
  import {
    ArrowLeft,
    AlertCircle,
    FileText,
    Globe,
    AlertCircle as IssuesIcon,
    FileText as MetaIcon,
    Share2,
    Heading,
    Image,
    Timer,
    FileJson
  } from '@lucide/svelte'
  import {
    ReportHeader,
    ReportScoreCard,
    ReportTabPanel,
    ScoreGauge,
    IssuesTable,
    MetaDetails,
    SocialMeta,
    HeadingStructure,
    ImagesTable,
    PerformanceTimeline,
    CrawlOverview,
    CrawlPagesTable,
    LoadingState,
    ErrorState,
    createReportViewStore
  } from '@pagelens/ui'
  import { commands, type AnalysisRun } from '$lib/ipc'
  import type {
    SinglePagePayload,
    CrawlResult,
    SeoReport,
    PerformanceTiming
  } from '$lib/types'

  interface Props {
    params?: { id?: string }
  }

  let { params = {} }: Props = $props()
  let run = $state<AnalysisRun | null>(null)
  let isLoading = $state(true)
  let error = $state<string | null>(null)
  let singlePayload = $state<SinglePagePayload | null>(null)
  let crawlPayload = $state<CrawlResult | null>(null)

  const viewStore = createReportViewStore('issues')

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

  const seoReport = $derived(singlePayload?.seo_report ?? null)
  const perfTiming = $derived(
    singlePayload?.snapshot?.performance_timing ?? null
  )

  // Computed tab availability
  const hasStructuredData = $derived(
    seoReport?.structured_data?.length ?? 0 > 0
  )
  const hasPerformance = $derived(perfTiming != null)
</script>

<div class="flex flex-col h-full overflow-auto p-6 space-y-6">
  {#if isLoading}
    <LoadingState message="Loading report..." />
  {:else if error}
    <ErrorState
      title="Error loading report"
      message={error}
      onRetry={() => push('/')}
    />
  {:else if run}
    <ReportHeader
      title={run.name || 'Analysis Report'}
      url={run.url}
      createdAt={run.created_at}
      analysisType={run.analysis_type}
      onBack={() => history.back()}
    />

    {#if run.analysis_type === 'single' && seoReport}
      <!-- Single Page Report -->
      <div class="flex flex-col items-center gap-4 sm:flex-row sm:items-start">
        <ScoreGauge score={seoReport.score} />
        <ReportScoreCard
          score={seoReport.score}
          issues={seoReport.issues}
          durationMs={run.summary.duration_ms}
        />
      </div>

      <Separator />

      <Tabs bind:value={$viewStore.activeTab}>
        <TabsList>
          <TabsTrigger value="issues">
            <IssuesIcon class="mr-1.5 h-3.5 w-3.5" />
            Issues ({seoReport.issues.length})
          </TabsTrigger>
          <TabsTrigger value="meta">
            <MetaIcon class="mr-1.5 h-3.5 w-3.5" />
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
          {#if hasStructuredData}
            <TabsTrigger value="structured">Schema</TabsTrigger>
          {/if}
          {#if hasPerformance}
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

        <ReportTabPanel value="issues" activeTab={$viewStore.activeTab}>
          <IssuesTable issues={seoReport.issues} />
        </ReportTabPanel>

        <ReportTabPanel value="meta" activeTab={$viewStore.activeTab}>
          <MetaDetails
            meta={seoReport.meta}
            canonical={seoReport.canonical_url}
          />
        </ReportTabPanel>

        <ReportTabPanel value="social" activeTab={$viewStore.activeTab}>
          <SocialMeta
            og={seoReport.open_graph}
            twitter={seoReport.twitter_card}
          />
        </ReportTabPanel>

        <ReportTabPanel value="headings" activeTab={$viewStore.activeTab}>
          <HeadingStructure headings={seoReport.headings} />
        </ReportTabPanel>

        <ReportTabPanel value="images" activeTab={$viewStore.activeTab}>
          <ImagesTable images={seoReport.images} />
        </ReportTabPanel>

        {#if hasStructuredData}
          <ReportTabPanel value="structured" activeTab={$viewStore.activeTab}>
            <div class="space-y-4">
              {#each seoReport.structured_data as sd}
                <div class="space-y-1">
                  <span class="rounded border px-2 py-0.5 text-xs"
                    >{sd.schema_type}</span
                  >
                  <pre
                    class="mt-1 rounded-md bg-muted p-3 text-xs overflow-auto max-h-48"><code
                      >{(() => {
                        try {
                          return JSON.stringify(JSON.parse(sd.raw), null, 2)
                        } catch {
                          return sd.raw
                        }
                      })()}</code
                    ></pre>
                </div>
              {/each}
            </div>
          </ReportTabPanel>
        {/if}

        {#if hasPerformance}
          <ReportTabPanel value="performance" activeTab={$viewStore.activeTab}>
            <PerformanceTimeline timing={perfTiming!} />
          </ReportTabPanel>
        {/if}

        <ReportTabPanel value="raw" activeTab={$viewStore.activeTab}>
          <pre
            class="rounded-md bg-muted p-4 text-xs overflow-auto max-h-[60vh]"><code
              >{JSON.stringify(JSON.parse(run.payload_json), null, 2)}</code
            ></pre>
        </ReportTabPanel>
      </Tabs>
    {:else if run.analysis_type === 'crawl' && crawlPayload}
      <!-- Crawl Report -->
      <CrawlOverview
        stats={crawlPayload.stats}
        aggregate={crawlPayload.aggregate}
      />

      <Separator />

      <Tabs bind:value={$viewStore.activeTab}>
        <TabsList>
          <TabsTrigger value="pages">
            <Globe class="mr-1.5 h-3.5 w-3.5" />
            Pages ({crawlPayload.pages.length})
          </TabsTrigger>
          <TabsTrigger value="issues">
            <IssuesIcon class="mr-1.5 h-3.5 w-3.5" />
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

        <ReportTabPanel value="pages" activeTab={$viewStore.activeTab}>
          <CrawlPagesTable pages={crawlPayload.pages} />
        </ReportTabPanel>

        <ReportTabPanel value="issues" activeTab={$viewStore.activeTab}>
          {@const allIssues = crawlPayload.pages.flatMap(p =>
            p.seo_report.issues.map(i => ({ ...i, url: p.url }))
          )}
          <IssuesTable issues={allIssues} />
        </ReportTabPanel>

        {#if crawlPayload.skipped_urls.length > 0}
          <ReportTabPanel value="skipped" activeTab={$viewStore.activeTab}>
            <div class="space-y-1 max-h-96 overflow-auto">
              {#each crawlPayload.skipped_urls as url}
                <p class="text-xs font-mono text-muted-foreground truncate">
                  {url}
                </p>
              {/each}
            </div>
          </ReportTabPanel>
        {/if}

        <ReportTabPanel value="raw" activeTab={$viewStore.activeTab}>
          <pre
            class="rounded-md bg-muted p-4 text-xs overflow-auto max-h-[60vh]"><code
              >{JSON.stringify(JSON.parse(run.payload_json), null, 2)}</code
            ></pre>
        </ReportTabPanel>
      </Tabs>
    {:else}
      <div class="rounded-lg border border-dashed p-12 text-center">
        <AlertCircle class="mx-auto h-12 w-12 text-muted-foreground mb-4" />
        <p class="text-lg font-semibold">Unable to parse report data</p>
        <Button variant="outline" class="mt-4" onclick={() => push('/')}
          >Back to Dashboard</Button
        >
      </div>
    {/if}
  {/if}
</div>
