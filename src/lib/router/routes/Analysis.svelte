<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { Button } from '$components/ui/button'
  import { Input } from '$components/ui/input'
  import { Label } from '$components/ui/label'
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$components/ui/tabs'
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle
  } from '$components/ui/card'
  import { Progress } from '$components/ui/progress'
  import { Badge } from '$components/ui/badge'
  import { Separator } from '$components/ui/separator'
  import { ScrollArea } from '$components/ui/scroll-area'
  import { Checkbox } from '$components/ui/checkbox'
  import { toast } from 'svelte-sonner'
  import {
    Scan,
    Globe,
    Loader2,
    CheckCircle2,
    AlertCircle,
    Clock,
    FileText,
    Layout,
    Gauge,
    Layers
  } from '@lucide/svelte'
  import { push } from 'svelte-spa-router'
  import { commands, events, type AnalysisRun } from '$lib/ipc'
  import type { SiteFilesReport as SiteFilesReportType } from '$lib/types'
  import SiteFilesReportComponent from '$lib/components/report/SiteFilesReport.svelte'

  // ============================================================================
  // State
  // ============================================================================

  // URL input
  let url = $state('')
  let name = $state('')

  // Loading and progress states
  let isAnalyzing = $state(false)
  let progress = $state(0)
  let progressStage = $state('')
  let progressMessage = $state('')

  // Result
  let lastResult = $state<AnalysisRun | null>(null)
  let error = $state<string | null>(null)

  // Options for single analysis
  let singleOptions = $state({
    includeHtml: true,
    includeAccessibilityTree: true,
    includePerformanceTiming: true,
    includeComputedStyles: false
  })

  // Options for crawl analysis
  let crawlOptions = $state({
    maxPages: 50,
    maxDepth: 3,
    followExternalLinks: false,
    sameSubdomainOnly: true,
    pageTimeoutMs: 30000,
    delayMs: 100,
    maxConcurrency: 4
  })

  // Event listener
  let unlistenProgress: (() => void) | null = $state(null)

  // ============================================================================
  // Lifecycle
  // ============================================================================

  onMount(async () => {
    unlistenProgress = await events.analysisProgressEvent.listen(event => {
      progressStage = event.payload.stage
      progressMessage = event.payload.message
      progress = event.payload.progress ? event.payload.progress * 100 : 0
    })
  })

  onDestroy(() => {
    if (unlistenProgress) {
      unlistenProgress()
    }
  })

  // ============================================================================
  // Handlers
  // ============================================================================

  async function handleSingleAnalysis() {
    if (!url.trim()) {
      toast.error('Please enter a URL')
      return
    }

    const runId = crypto.randomUUID()
    push(`/run/${runId}`)

    void commands
      .analyzeUrl({
        run_id: runId,
        url: url.trim(),
        name: name.trim() || null,
        options: {
          include_html: singleOptions.includeHtml,
          include_accessibility_tree: singleOptions.includeAccessibilityTree,
          include_performance_timing: singleOptions.includePerformanceTiming,
          include_computed_styles: singleOptions.includeComputedStyles
        }
      })
      .catch(err => {
        const message = err instanceof Error ? err.message : String(err)
        toast.error(`Analysis failed: ${message}`)
      })
  }

  async function handleCrawlAnalysis() {
    if (!url.trim()) {
      toast.error('Please enter a URL')
      return
    }

    const runId = crypto.randomUUID()
    push(`/run/${runId}`)

    void commands
      .crawlUrl({
        run_id: runId,
        url: url.trim(),
        name: name.trim() || null,
        options: {
          max_pages: crawlOptions.maxPages,
          max_depth: crawlOptions.maxDepth,
          follow_external_links: crawlOptions.followExternalLinks,
          same_subdomain_only: crawlOptions.sameSubdomainOnly,
          page_timeout_ms: crawlOptions.pageTimeoutMs,
          delay_ms: crawlOptions.delayMs,
          max_concurrency: crawlOptions.maxConcurrency
        }
      })
      .catch(err => {
        const message = err instanceof Error ? err.message : String(err)
        toast.error(`Crawl failed: ${message}`)
      })
  }

  // Site files state
  let siteFilesUrl = $state('')
  let isAnalyzingSiteFiles = $state(false)
  let siteFilesReport = $state<SiteFilesReportType | null>(null)
  let siteFilesError = $state<string | null>(null)

  async function handleSiteFilesAnalysis() {
    if (!siteFilesUrl.trim()) {
      toast.error('Please enter a URL')
      return
    }

    const runId = crypto.randomUUID()
    push(`/run/${runId}`)

    void commands
      .analyzeSiteFiles({
        run_id: runId,
        url: siteFilesUrl.trim(),
        crawled_urls: null
      })
      .catch(err => {
        const message = err instanceof Error ? err.message : String(err)
        toast.error(`Site files analysis failed: ${message}`)
      })
  }

  function formatDuration(ms: number): string {
    if (ms < 1000) return `${ms}ms`
    return `${(ms / 1000).toFixed(1)}s`
  }

  function formatDate(isoString: string): string {
    return new Date(isoString).toLocaleString()
  }
</script>

<div class="h-full flex flex-col p-6 gap-6 overflow-auto">
  <!-- Header -->
  <div class="flex items-center gap-4">
    <div
      class="flex h-12 w-12 items-center justify-center rounded-xl bg-gradient-to-br from-indigo-500 to-violet-600 shadow-lg"
    >
      <Scan class="h-6 w-6 text-white" />
    </div>
    <div>
      <h1 class="text-3xl font-bold tracking-tight">Analysis</h1>
      <p class="text-muted-foreground">
        Analyze single pages or crawl entire sites
      </p>
    </div>
  </div>

  <Separator />

  <div class="grid gap-6 lg:grid-cols-2">
    <!-- Input Section -->
    <div class="space-y-6">
      <Tabs value="single" class="w-full">
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

        <!-- Single Page Analysis -->
        <TabsContent value="single" class="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle class="flex items-center gap-2">
                <FileText class="h-5 w-5 text-indigo-500" />
                Single Page Analysis
              </CardTitle>
              <CardDescription>
                Analyze SEO, accessibility, and performance for a single URL
              </CardDescription>
            </CardHeader>
            <CardContent class="space-y-4">
              <div class="space-y-2">
                <Label for="url-single">URL</Label>
                <Input
                  id="url-single"
                  type="url"
                  placeholder="https://example.com"
                  bind:value={url}
                  disabled={isAnalyzing}
                />
              </div>

              <div class="space-y-2">
                <Label for="name-single">Name (optional)</Label>
                <Input
                  id="name-single"
                  placeholder="My Analysis"
                  bind:value={name}
                  disabled={isAnalyzing}
                />
              </div>

              <Separator />

              <div class="space-y-3">
                <Label class="text-sm font-medium">Snapshot Options</Label>
                <div class="grid gap-3">
                  <div class="flex items-center space-x-2">
                    <Checkbox
                      id="html"
                      bind:checked={singleOptions.includeHtml}
                      disabled={isAnalyzing}
                    />
                    <Label for="html" class="text-sm font-normal"
                      >Include HTML content</Label
                    >
                  </div>
                  <div class="flex items-center space-x-2">
                    <Checkbox
                      id="a11y"
                      bind:checked={singleOptions.includeAccessibilityTree}
                      disabled={isAnalyzing}
                    />
                    <Label for="a11y" class="text-sm font-normal"
                      >Include accessibility tree</Label
                    >
                  </div>
                  <div class="flex items-center space-x-2">
                    <Checkbox
                      id="perf"
                      bind:checked={singleOptions.includePerformanceTiming}
                      disabled={isAnalyzing}
                    />
                    <Label for="perf" class="text-sm font-normal"
                      >Include performance timing</Label
                    >
                  </div>
                  <div class="flex items-center space-x-2">
                    <Checkbox
                      id="styles"
                      bind:checked={singleOptions.includeComputedStyles}
                      disabled={isAnalyzing}
                    />
                    <Label for="styles" class="text-sm font-normal"
                      >Include computed styles</Label
                    >
                  </div>
                </div>
              </div>

              <Button
                class="w-full"
                onclick={handleSingleAnalysis}
                disabled={isAnalyzing || !url.trim()}
              >
                {#if isAnalyzing}
                  <Loader2 class="mr-2 h-4 w-4 animate-spin" />
                  Analyzing...
                {:else}
                  <Scan class="mr-2 h-4 w-4" />
                  Analyze URL
                {/if}
              </Button>
            </CardContent>
          </Card>
        </TabsContent>

        <!-- Crawl Analysis -->
        <TabsContent value="crawl" class="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle class="flex items-center gap-2">
                <Globe class="h-5 w-5 text-violet-500" />
                Site Crawl
              </CardTitle>
              <CardDescription>
                Crawl and analyze multiple pages starting from a URL
              </CardDescription>
            </CardHeader>
            <CardContent class="space-y-4">
              <div class="space-y-2">
                <Label for="url-crawl">Starting URL</Label>
                <Input
                  id="url-crawl"
                  type="url"
                  placeholder="https://example.com"
                  bind:value={url}
                  disabled={isAnalyzing}
                />
              </div>

              <div class="space-y-2">
                <Label for="name-crawl">Name (optional)</Label>
                <Input
                  id="name-crawl"
                  placeholder="My Crawl"
                  bind:value={name}
                  disabled={isAnalyzing}
                />
              </div>

              <Separator />

              <div class="grid gap-4 sm:grid-cols-2">
                <div class="space-y-2">
                  <Label for="max-pages">Max Pages</Label>
                  <Input
                    id="max-pages"
                    type="number"
                    bind:value={crawlOptions.maxPages}
                    disabled={isAnalyzing}
                  />
                </div>
                <div class="space-y-2">
                  <Label for="max-depth">Max Depth</Label>
                  <Input
                    id="max-depth"
                    type="number"
                    bind:value={crawlOptions.maxDepth}
                    disabled={isAnalyzing}
                  />
                </div>
              </div>

              <div class="space-y-3">
                <Label class="text-sm font-medium">Crawl Options</Label>
                <div class="grid gap-3">
                  <div class="flex items-center space-x-2">
                    <Checkbox
                      id="external"
                      bind:checked={crawlOptions.followExternalLinks}
                      disabled={isAnalyzing}
                    />
                    <Label for="external" class="text-sm font-normal"
                      >Follow external links</Label
                    >
                  </div>
                  <div class="flex items-center space-x-2">
                    <Checkbox
                      id="subdomain"
                      bind:checked={crawlOptions.sameSubdomainOnly}
                      disabled={isAnalyzing}
                    />
                    <Label for="subdomain" class="text-sm font-normal"
                      >Same subdomain only</Label
                    >
                  </div>
                </div>
              </div>

              <Button
                class="w-full"
                onclick={handleCrawlAnalysis}
                disabled={isAnalyzing || !url.trim()}
              >
                {#if isAnalyzing}
                  <Loader2 class="mr-2 h-4 w-4 animate-spin" />
                  Crawling...
                {:else}
                  <Globe class="mr-2 h-4 w-4" />
                  Start Crawl
                {/if}
              </Button>
            </CardContent>
          </Card>
        </TabsContent>

        <!-- Site Files Analysis -->
        <TabsContent value="sitefiles" class="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle class="flex items-center gap-2">
                <Layers class="h-5 w-5 text-teal-500" />
                Site Files Analysis
              </CardTitle>
              <CardDescription>
                Check robots.txt, sitemaps, and other site files
              </CardDescription>
            </CardHeader>
            <CardContent class="space-y-4">
              <div class="space-y-2">
                <Label for="url-sitefiles">URL</Label>
                <Input
                  id="url-sitefiles"
                  type="url"
                  placeholder="https://example.com"
                  bind:value={siteFilesUrl}
                  disabled={isAnalyzingSiteFiles}
                />
              </div>

              <Button
                class="w-full"
                onclick={handleSiteFilesAnalysis}
                disabled={isAnalyzingSiteFiles || !siteFilesUrl.trim()}
              >
                {#if isAnalyzingSiteFiles}
                  <Loader2 class="mr-2 h-4 w-4 animate-spin" />
                  Analyzing...
                {:else}
                  <Layers class="mr-2 h-4 w-4" />
                  Analyze Site Files
                {/if}
              </Button>

              {#if siteFilesError}
                <p class="text-sm text-red-500">{siteFilesError}</p>
              {/if}

              {#if siteFilesReport}
                <Separator />
                <SiteFilesReportComponent report={siteFilesReport} />
              {/if}
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>

    <!-- Results Section -->
    <div class="space-y-6">
      <!-- Progress Card -->
      {#if isAnalyzing}
        <Card
          class="border-indigo-200 bg-indigo-50/50 dark:border-indigo-900 dark:bg-indigo-950/20"
        >
          <CardHeader>
            <CardTitle
              class="flex items-center gap-2 text-indigo-700 dark:text-indigo-300"
            >
              <Loader2 class="h-5 w-5 animate-spin" />
              Analysis in Progress
            </CardTitle>
          </CardHeader>
          <CardContent class="space-y-4">
            <div class="space-y-2">
              <div class="flex justify-between text-sm">
                <span class="font-medium capitalize">{progressStage}</span>
                <span class="text-muted-foreground"
                  >{Math.round(progress)}%</span
                >
              </div>
              <Progress value={progress} class="h-2" />
            </div>
            <p class="text-sm text-muted-foreground">{progressMessage}</p>
          </CardContent>
        </Card>
      {/if}

      <!-- Error Display -->
      {#if error}
        <Card
          class="border-red-200 bg-red-50/50 dark:border-red-900 dark:bg-red-950/20"
        >
          <CardHeader>
            <CardTitle
              class="flex items-center gap-2 text-red-700 dark:text-red-300"
            >
              <AlertCircle class="h-5 w-5" />
              Analysis Failed
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p class="text-sm text-red-600 dark:text-red-400">{error}</p>
          </CardContent>
        </Card>
      {/if}

      <!-- Last Result -->
      {#if lastResult}
        <Card>
          <CardHeader>
            <CardTitle class="flex items-center gap-2">
              <CheckCircle2 class="h-5 w-5 text-green-500" />
              Analysis Complete
            </CardTitle>
            <CardDescription>
              {lastResult.name || 'Unnamed analysis'} • {formatDate(
                lastResult.created_at
              )}
            </CardDescription>
          </CardHeader>
          <CardContent class="space-y-4">
            <!-- Summary Stats -->
            <div class="grid grid-cols-2 gap-4 sm:grid-cols-4">
              <div
                class="flex flex-col items-center justify-center rounded-lg bg-muted p-3"
              >
                <Gauge class="mb-1 h-5 w-5 text-indigo-500" />
                <span class="text-2xl font-bold">
                  {lastResult.summary.seo_score
                    ? Math.round(lastResult.summary.seo_score)
                    : '-'}
                </span>
                <span class="text-xs text-muted-foreground">SEO Score</span>
              </div>

              <div
                class="flex flex-col items-center justify-center rounded-lg bg-muted p-3"
              >
                <Layout class="mb-1 h-5 w-5 text-violet-500" />
                <span class="text-2xl font-bold"
                  >{lastResult.summary.page_count}</span
                >
                <span class="text-xs text-muted-foreground">Pages</span>
              </div>

              <div
                class="flex flex-col items-center justify-center rounded-lg bg-muted p-3"
              >
                <AlertCircle class="mb-1 h-5 w-5 text-red-500" />
                <span class="text-2xl font-bold"
                  >{lastResult.summary.error_count}</span
                >
                <span class="text-xs text-muted-foreground">Errors</span>
              </div>

              <div
                class="flex flex-col items-center justify-center rounded-lg bg-muted p-3"
              >
                <Clock class="mb-1 h-5 w-5 text-amber-500" />
                <span class="text-2xl font-bold"
                  >{formatDuration(lastResult.summary.duration_ms)}</span
                >
                <span class="text-xs text-muted-foreground">Duration</span>
              </div>
            </div>

            <Separator />

            <!-- Issues Breakdown -->
            <div class="space-y-2">
              <div class="flex items-center justify-between">
                <span class="text-sm font-medium">Total Issues</span>
                <Badge variant="secondary"
                  >{lastResult.summary.total_issues}</Badge
                >
              </div>
              <div class="flex items-center justify-between">
                <span class="text-sm font-medium">Warnings</span>
                <Badge
                  variant="outline"
                  class="border-amber-500 text-amber-600"
                >
                  {lastResult.summary.warning_count}
                </Badge>
              </div>
            </div>

            <!-- Analysis Type Badge -->
            <div class="flex items-center gap-2">
              <span class="text-sm text-muted-foreground">Type:</span>
              <Badge
                variant={lastResult.analysis_type === 'single'
                  ? 'default'
                  : 'secondary'}
              >
                {lastResult.analysis_type === 'single'
                  ? 'Single Page'
                  : 'Crawl'}
              </Badge>
            </div>

            <!-- View Detailed Report -->
            <Button
              variant="outline"
              class="w-full"
              onclick={() => push(`/run/${lastResult!.id}`)}
            >
              <Layers class="mr-2 h-4 w-4" />
              View Detailed Report
            </Button>
          </CardContent>
        </Card>
      {:else if !isAnalyzing}
        <!-- Empty State -->
        <Card class="border-dashed">
          <CardContent
            class="flex flex-col items-center justify-center py-12 text-center"
          >
            <div
              class="flex h-16 w-16 items-center justify-center rounded-full bg-muted"
            >
              <Scan class="h-8 w-8 text-muted-foreground" />
            </div>
            <h3 class="mt-4 text-lg font-semibold">No Analysis Yet</h3>
            <p class="mt-2 max-w-xs text-sm text-muted-foreground">
              Enter a URL and click Analyze to start. Results will appear here.
            </p>
          </CardContent>
        </Card>
      {/if}
    </div>
  </div>
</div>
