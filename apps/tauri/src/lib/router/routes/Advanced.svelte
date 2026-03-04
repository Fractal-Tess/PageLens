<script lang="ts">
  import { onMount } from 'svelte';
  import { toast } from 'svelte-sonner';
  import { push } from 'svelte-spa-router';
  import { commands } from '$lib/ipc';
  import { AdvancedRunForm } from '@pagelens/ui';
  import {
    DEFAULT_CRAWL_OPTIONS,
    DEFAULT_SINGLE_OPTIONS,
    loadDefaultProfileId,
    loadProfiles,
    type AnalysisProfile,
    type RunKind,
  } from '$lib/analysis-profiles';

  let profiles = $state<AnalysisProfile[]>([]);
  let selectedProfileId = $state('');

  let url = $state('');
  let name = $state('');
  let runKind = $state<RunKind>('crawl');
  let isRunning = $state(false);

  let singleOptions = $state({ ...DEFAULT_SINGLE_OPTIONS });
  let crawlOptions = $state({ ...DEFAULT_CRAWL_OPTIONS });

  const selectedProfile = $derived(profiles.find(profile => profile.id === selectedProfileId) ?? null);

  onMount(() => {
    profiles = loadProfiles();
    const defaultId = loadDefaultProfileId();
    if (defaultId && profiles.some(profile => profile.id === defaultId)) {
      selectedProfileId = defaultId;
      applyProfile(defaultId);
    }
  });

  function applyProfile(profileId: string) {
    const profile = profiles.find(item => item.id === profileId);
    if (!profile) return;
    selectedProfileId = profile.id;
    runKind = profile.runKind;
    singleOptions = { ...profile.singleOptions };
    crawlOptions = { ...profile.crawlOptions };
  }

  function resetToProfile() {
    if (!selectedProfileId) return;
    applyProfile(selectedProfileId);
    toast.success('Reset to profile defaults');
  }

  function updateSingleOption(
    key: keyof typeof singleOptions,
    checked: boolean,
  ) {
    singleOptions = { ...singleOptions, [key]: checked };
  }

  function updateCrawlNumber(
    key: keyof typeof crawlOptions,
    value: string,
  ) {
    const numeric = Number.parseInt(value, 10);
    if (!Number.isFinite(numeric)) return;
    crawlOptions = { ...crawlOptions, [key]: numeric };
  }

  function updateCrawlToggle(
    key: 'followExternalLinks' | 'sameSubdomainOnly',
    checked: boolean,
  ) {
    crawlOptions = { ...crawlOptions, [key]: checked };
  }

  async function startRun() {
    if (!url.trim()) {
      toast.error('Please enter a URL');
      return;
    }

    isRunning = true;
    const runId = crypto.randomUUID();
    push(`/run/${runId}`);

    try {
      let result:
        | Awaited<ReturnType<typeof commands.analyzeUrl>>
        | Awaited<ReturnType<typeof commands.crawlUrl>>
        | Awaited<ReturnType<typeof commands.analyzeSiteFiles>>;

      if (runKind === 'single') {
        result = await commands.analyzeUrl({
          run_id: runId,
          url: url.trim(),
          name: name.trim() || null,
          options: {
            include_html: singleOptions.includeHtml,
            include_accessibility_tree: singleOptions.includeAccessibilityTree,
            include_performance_timing: singleOptions.includePerformanceTiming,
            include_computed_styles: singleOptions.includeComputedStyles,
          },
        });
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
            max_concurrency: crawlOptions.maxConcurrency,
          },
        });
      } else {
        result = await commands.analyzeSiteFiles({
          run_id: runId,
          url: url.trim(),
          crawled_urls: null,
        });
      }

      if (result.status === 'error') {
        throw new Error(result.error);
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      toast.error(`Advanced run failed: ${message}`);
    } finally {
      isRunning = false;
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

  <AdvancedRunForm
    profiles={profiles.map(profile => ({ id: profile.id, name: profile.name }))}
    {selectedProfileId}
    selectedProfileName={selectedProfile?.name ?? null}
    {url}
    {name}
    {runKind}
    {isRunning}
    {singleOptions}
    {crawlOptions}
    onSelectProfile={applyProfile}
    onResetToProfile={resetToProfile}
    onUrlChange={value => (url = value)}
    onNameChange={value => (name = value)}
    onRunKindChange={value => (runKind = value)}
    onSingleToggle={updateSingleOption}
    onCrawlNumberChange={updateCrawlNumber}
    onCrawlToggle={updateCrawlToggle}
    onStartRun={startRun}
  />
</div>
