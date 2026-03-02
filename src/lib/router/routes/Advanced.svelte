<script lang="ts">
  import { onMount } from 'svelte'
  import { Button } from '$components/ui/button'
  import { Input } from '$components/ui/input'
  import { Label } from '$components/ui/label'
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle
  } from '$components/ui/card'
  import { Checkbox } from '$components/ui/checkbox'
  import { Separator } from '$components/ui/separator'
  import { Badge } from '$components/ui/badge'
  import { toast } from 'svelte-sonner'
  import {
    Scan,
    Globe,
    Layers,
    Loader2,
    WandSparkles,
    RotateCcw
  } from '@lucide/svelte'
  import { push } from 'svelte-spa-router'
  import { commands } from '$lib/ipc'
  import {
    DEFAULT_CRAWL_OPTIONS,
    DEFAULT_SINGLE_OPTIONS,
    loadDefaultProfileId,
    loadProfiles,
    type AnalysisProfile,
    type RunKind
  } from '$lib/analysis-profiles'

  let profiles = $state<AnalysisProfile[]>([])
  let selectedProfileId = $state<string>('')

  let url = $state('')
  let name = $state('')
  let runKind = $state<RunKind>('crawl')
  let isRunning = $state(false)

  let singleOptions = $state({ ...DEFAULT_SINGLE_OPTIONS })
  let crawlOptions = $state({ ...DEFAULT_CRAWL_OPTIONS })

  const selectedProfile = $derived(
    profiles.find(profile => profile.id === selectedProfileId) ?? null
  )

  onMount(() => {
    profiles = loadProfiles()
    const defaultId = loadDefaultProfileId()
    if (defaultId && profiles.some(profile => profile.id === defaultId)) {
      selectedProfileId = defaultId
      applyProfile(defaultId)
    }
  })

  function applyProfile(profileId: string) {
    const profile = profiles.find(item => item.id === profileId)
    if (!profile) return
    selectedProfileId = profile.id
    runKind = profile.runKind
    singleOptions = { ...profile.singleOptions }
    crawlOptions = { ...profile.crawlOptions }
  }

  function resetToProfile() {
    if (!selectedProfileId) return
    applyProfile(selectedProfileId)
    toast.success('Reset to profile defaults')
  }

  async function startRun() {
    if (!url.trim()) {
      toast.error('Please enter a URL')
      return
    }

    isRunning = true
    const runId = crypto.randomUUID()
    push(`/run/${runId}`)

    try {
      let result:
        | Awaited<ReturnType<typeof commands.analyzeUrl>>
        | Awaited<ReturnType<typeof commands.crawlUrl>>
        | Awaited<ReturnType<typeof commands.analyzeSiteFiles>>

      if (runKind === 'single') {
        result = await commands.analyzeUrl({
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
      } else if (runKind === 'crawl') {
        result = await commands.crawlUrl({
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
      } else {
        result = await commands.analyzeSiteFiles({
          run_id: runId,
          url: url.trim(),
          crawled_urls: null
        })
      }

      if (result.status === 'error') {
        throw new Error(result.error)
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err)
      toast.error(`Advanced run failed: ${message}`)
    } finally {
      isRunning = false
    }
  }
</script>

<div class="h-full overflow-auto p-6 space-y-6">
  <div>
    <h1 class="text-3xl font-bold tracking-tight">Advanced Analysis</h1>
    <p class="text-sm text-muted-foreground">
      Select a profile to prefill options, then edit anything before running.
    </p>
  </div>

  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2">
        <WandSparkles class="h-5 w-5 text-indigo-500" /> Profile prefill
      </CardTitle>
      <CardDescription>
        Applying a profile updates the options below; edits remain fully manual.
      </CardDescription>
    </CardHeader>
    <CardContent class="grid gap-4 sm:grid-cols-[1fr_auto_auto]">
      <div class="space-y-2">
        <Label for="advanced-profile">Profile</Label>
        <select
          id="advanced-profile"
          class="w-full rounded-md border bg-background px-3 py-2 text-sm"
          value={selectedProfileId}
          onchange={event =>
            applyProfile((event.currentTarget as HTMLSelectElement).value)}
        >
          <option value="">No profile</option>
          {#each profiles as profile (profile.id)}
            <option value={profile.id}>{profile.name}</option>
          {/each}
        </select>
      </div>

      <div class="self-end">
        <Button
          variant="secondary"
          onclick={resetToProfile}
          disabled={!selectedProfileId}
        >
          <RotateCcw class="mr-2 h-4 w-4" /> Reset to Profile
        </Button>
      </div>

      <div class="self-end flex items-center">
        {#if selectedProfile}
          <Badge variant="secondary">Using: {selectedProfile.name}</Badge>
        {/if}
      </div>
    </CardContent>
  </Card>

  <Card>
    <CardHeader>
      <CardTitle>Run Configuration</CardTitle>
    </CardHeader>
    <CardContent class="space-y-5">
      <div class="grid gap-4 sm:grid-cols-2">
        <div class="space-y-2">
          <Label for="advanced-url">URL</Label>
          <Input
            id="advanced-url"
            type="url"
            bind:value={url}
            disabled={isRunning}
            placeholder="https://example.com"
          />
        </div>
        <div class="space-y-2">
          <Label for="advanced-name">Run name (optional)</Label>
          <Input
            id="advanced-name"
            bind:value={name}
            disabled={isRunning}
            placeholder="My advanced run"
          />
        </div>
      </div>

      <div class="space-y-2">
        <Label for="advanced-kind">Run type</Label>
        <select
          id="advanced-kind"
          class="w-full rounded-md border bg-background px-3 py-2 text-sm"
          bind:value={runKind}
          disabled={isRunning}
        >
          <option value="single">Single page</option>
          <option value="crawl">Crawl</option>
          <option value="site_files">Site files</option>
        </select>
      </div>

      <Separator />

      {#if runKind === 'single'}
        <div class="space-y-3">
          <Label class="text-sm font-medium">Single page options</Label>
          <div class="grid gap-3 sm:grid-cols-2">
            <div class="flex items-center space-x-2">
              <Checkbox
                bind:checked={singleOptions.includeHtml}
                id="adv-html"
                disabled={isRunning}
              />
              <Label for="adv-html" class="text-sm font-normal"
                >Include HTML</Label
              >
            </div>
            <div class="flex items-center space-x-2">
              <Checkbox
                bind:checked={singleOptions.includeAccessibilityTree}
                id="adv-a11y"
                disabled={isRunning}
              />
              <Label for="adv-a11y" class="text-sm font-normal"
                >Accessibility tree</Label
              >
            </div>
            <div class="flex items-center space-x-2">
              <Checkbox
                bind:checked={singleOptions.includePerformanceTiming}
                id="adv-perf"
                disabled={isRunning}
              />
              <Label for="adv-perf" class="text-sm font-normal"
                >Performance timing</Label
              >
            </div>
            <div class="flex items-center space-x-2">
              <Checkbox
                bind:checked={singleOptions.includeComputedStyles}
                id="adv-styles"
                disabled={isRunning}
              />
              <Label for="adv-styles" class="text-sm font-normal"
                >Computed styles</Label
              >
            </div>
          </div>
        </div>
      {/if}

      {#if runKind === 'crawl'}
        <div class="space-y-4">
          <Label class="text-sm font-medium">Crawl options</Label>
          <div class="grid gap-4 sm:grid-cols-3">
            <div class="space-y-2">
              <Label>Max pages</Label>
              <Input
                type="number"
                bind:value={crawlOptions.maxPages}
                disabled={isRunning}
              />
            </div>
            <div class="space-y-2">
              <Label>Max depth</Label>
              <Input
                type="number"
                bind:value={crawlOptions.maxDepth}
                disabled={isRunning}
              />
            </div>
            <div class="space-y-2">
              <Label>Concurrency</Label>
              <Input
                type="number"
                bind:value={crawlOptions.maxConcurrency}
                disabled={isRunning}
              />
            </div>
            <div class="space-y-2">
              <Label>Timeout (ms)</Label>
              <Input
                type="number"
                bind:value={crawlOptions.pageTimeoutMs}
                disabled={isRunning}
              />
            </div>
            <div class="space-y-2">
              <Label>Delay (ms)</Label>
              <Input
                type="number"
                bind:value={crawlOptions.delayMs}
                disabled={isRunning}
              />
            </div>
          </div>
          <div class="grid gap-3 sm:grid-cols-2">
            <div class="flex items-center space-x-2">
              <Checkbox
                bind:checked={crawlOptions.followExternalLinks}
                id="adv-external"
                disabled={isRunning}
              />
              <Label for="adv-external" class="text-sm font-normal"
                >Follow external links</Label
              >
            </div>
            <div class="flex items-center space-x-2">
              <Checkbox
                bind:checked={crawlOptions.sameSubdomainOnly}
                id="adv-subdomain"
                disabled={isRunning}
              />
              <Label for="adv-subdomain" class="text-sm font-normal"
                >Same subdomain only</Label
              >
            </div>
          </div>
        </div>
      {/if}

      {#if runKind === 'site_files'}
        <div
          class="rounded border bg-muted/40 p-4 text-sm text-muted-foreground"
        >
          Site-files mode checks robots.txt and sitemap files for this URL.
        </div>
      {/if}

      <Button
        class="w-full"
        onclick={startRun}
        disabled={isRunning || !url.trim()}
      >
        {#if isRunning}
          <Loader2 class="mr-2 h-4 w-4 animate-spin" /> Running advanced analysis...
        {:else if runKind === 'single'}
          <Scan class="mr-2 h-4 w-4" /> Run Single Analysis
        {:else if runKind === 'crawl'}
          <Globe class="mr-2 h-4 w-4" /> Start Crawl Analysis
        {:else}
          <Layers class="mr-2 h-4 w-4" /> Analyze Site Files
        {/if}
      </Button>
    </CardContent>
  </Card>
</div>
