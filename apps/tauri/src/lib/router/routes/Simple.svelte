<script lang="ts">
  import { onMount } from 'svelte';
  import { push } from 'svelte-spa-router';
  import { toast } from 'svelte-sonner';
  import { Separator } from '@pagelens/ui/shadcn/separator';
  import {
    PageHeader,
    AnalysisForm,
    AnalysisProgress,
    analysisStore,
    createSingleOptionsStore,
    DEFAULT_SINGLE_OPTIONS,
  } from '@pagelens/ui';
  import { Scan } from '@lucide/svelte';
  import { commands } from '$lib/ipc';

  // Local state
  const singleOptions = createSingleOptionsStore(DEFAULT_SINGLE_OPTIONS);
  
  onMount(() => {
    const hash = window.location.hash;
    const queryIndex = hash.indexOf('?');
    if (queryIndex === -1) return;
    
    const params = new URLSearchParams(hash.slice(queryIndex + 1));
    const prefilledUrl = params.get('url');
    if (prefilledUrl) {
      analysisStore.setUrl(prefilledUrl);
    }
  });

  async function handleSubmit() {
    const { url, name } = $analysisStore;
    
    if (!url.trim()) {
      toast.error('Please enter a URL');
      return;
    }

    const runId = crypto.randomUUID();
    analysisStore.start(runId);
    push(`/run/${runId}`);

    try {
      const result = await commands.analyzeUrl({
        run_id: runId,
        url: url.trim(),
        name: name.trim() || null,
        options: {
          include_html: $singleOptions.includeHtml,
          include_accessibility_tree: $singleOptions.includeAccessibilityTree,
          include_performance_timing: $singleOptions.includePerformanceTiming,
          include_computed_styles: $singleOptions.includeComputedStyles,
        },
      });

      if (result.status === 'error') {
        throw new Error(result.error);
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      analysisStore.fail(message);
      toast.error(`Analysis failed: ${message}`);
    }
  }
</script>

<div class="h-full overflow-auto p-6 space-y-6">
  <PageHeader
    title="Simple Analysis"
    description="Quick single-page scan with minimal setup."
    icon={Scan}
  />

  <Separator />

  <div class="max-w-2xl">
    <AnalysisForm
      type="single"
      url={$analysisStore.url}
      name={$analysisStore.name}
      singleOptions={$singleOptions}
      crawlOptions={{
        maxPages: 50,
        maxDepth: 3,
        followExternalLinks: false,
        sameSubdomainOnly: true,
        pageTimeoutMs: 30000,
        delayMs: 100,
        maxConcurrency: 4,
      }}
      isSubmitting={$analysisStore.status === 'running'}
      onSubmit={handleSubmit}
      onUrlChange={analysisStore.setUrl}
      onNameChange={analysisStore.setName}
      onSingleOptionChange={(key, value) => singleOptions.update(s => ({ ...s, [key]: value }))}
      onCrawlOptionChange={() => {}}
    />
  </div>
</div>
