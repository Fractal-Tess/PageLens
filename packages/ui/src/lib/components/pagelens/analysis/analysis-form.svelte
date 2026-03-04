<script lang="ts">
  import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '../../ui/card';
  import { Button } from '../../ui/button';
  import { Input } from '../../ui/input';
  import { Label } from '../../ui/label';
  import { Checkbox } from '../../ui/checkbox';
  import { Loader2, Scan, Globe, Layers } from '@lucide/svelte';
  import type { AnalysisType, SingleAnalysisOptions, CrawlAnalysisOptions } from './types';
  
  interface Props {
    type: AnalysisType;
    url: string;
    name: string;
    singleOptions: SingleAnalysisOptions;
    crawlOptions: CrawlAnalysisOptions;
    isSubmitting: boolean;
    onSubmit: () => void;
    onUrlChange: (url: string) => void;
    onNameChange: (name: string) => void;
    onSingleOptionChange: <K extends keyof SingleAnalysisOptions>(key: K, value: SingleAnalysisOptions[K]) => void;
    onCrawlOptionChange: <K extends keyof CrawlAnalysisOptions>(key: K, value: CrawlAnalysisOptions[K]) => void;
    class?: string;
  }
  
  let {
    type,
    url,
    name,
    singleOptions,
    crawlOptions,
    isSubmitting,
    onSubmit,
    onUrlChange,
    onNameChange,
    onSingleOptionChange,
    onCrawlOptionChange,
    class: className = ''
  }: Props = $props();
  
  const isValid = $derived(url.trim().length > 0);
  
  const typeConfig = $derived({
    single: { icon: Scan, label: 'Single Page Analysis', description: 'Analyze SEO, accessibility, and performance for a single URL' },
    crawl: { icon: Globe, label: 'Site Crawl', description: 'Crawl and analyze multiple pages starting from a URL' },
    site_files: { icon: Layers, label: 'Site Files Analysis', description: 'Check robots.txt, sitemaps, and other site files' },
  }[type]);
</script>

<Card class={className}>
  <CardHeader>
    <CardTitle class="flex items-center gap-2">
      <svelte:component this={typeConfig.icon} class="h-5 w-5 text-indigo-500" />
      {typeConfig.label}
    </CardTitle>
    <CardDescription>{typeConfig.description}</CardDescription>
  </CardHeader>
  <CardContent class="space-y-4">
    <!-- URL Input -->
    <div class="space-y-2">
      <Label for="analysis-url">URL</Label>
      <Input
        id="analysis-url"
        type="url"
        placeholder="https://example.com"
        value={url}
        oninput={(e) => onUrlChange(e.currentTarget.value)}
        disabled={isSubmitting}
      />
    </div>
    
    <!-- Name Input -->
    <div class="space-y-2">
      <Label for="analysis-name">Name (optional)</Label>
      <Input
        id="analysis-name"
        placeholder="My Analysis"
        value={name}
        oninput={(e) => onNameChange(e.currentTarget.value)}
        disabled={isSubmitting}
      />
    </div>
    
    <!-- Type-specific options -->
    {#if type === 'single'}
      <div class="space-y-3 pt-2">
        <Label class="text-sm font-medium">Snapshot Options</Label>
        <div class="grid gap-3 sm:grid-cols-2">
          <div class="flex items-center space-x-2">
            <Checkbox
              id="opt-html"
              checked={singleOptions.includeHtml}
              onCheckedChange={(v) => onSingleOptionChange('includeHtml', Boolean(v))}
              disabled={isSubmitting}
            />
            <Label for="opt-html" class="text-sm font-normal">Include HTML content</Label>
          </div>
          <div class="flex items-center space-x-2">
            <Checkbox
              id="opt-a11y"
              checked={singleOptions.includeAccessibilityTree}
              onCheckedChange={(v) => onSingleOptionChange('includeAccessibilityTree', Boolean(v))}
              disabled={isSubmitting}
            />
            <Label for="opt-a11y" class="text-sm font-normal">Include accessibility tree</Label>
          </div>
          <div class="flex items-center space-x-2">
            <Checkbox
              id="opt-perf"
              checked={singleOptions.includePerformanceTiming}
              onCheckedChange={(v) => onSingleOptionChange('includePerformanceTiming', Boolean(v))}
              disabled={isSubmitting}
            />
            <Label for="opt-perf" class="text-sm font-normal">Include performance timing</Label>
          </div>
          <div class="flex items-center space-x-2">
            <Checkbox
              id="opt-styles"
              checked={singleOptions.includeComputedStyles}
              onCheckedChange={(v) => onSingleOptionChange('includeComputedStyles', Boolean(v))}
              disabled={isSubmitting}
            />
            <Label for="opt-styles" class="text-sm font-normal">Include computed styles</Label>
          </div>
        </div>
      </div>
    {:else if type === 'crawl'}
      <div class="space-y-4 pt-2">
        <Label class="text-sm font-medium">Crawl Options</Label>
        <div class="grid gap-4 sm:grid-cols-3">
          <div class="space-y-2">
            <Label for="opt-max-pages">Max Pages</Label>
            <Input
              id="opt-max-pages"
              type="number"
              value={crawlOptions.maxPages}
              oninput={(e) => onCrawlOptionChange('maxPages', parseInt(e.currentTarget.value) || 50)}
              disabled={isSubmitting}
            />
          </div>
          <div class="space-y-2">
            <Label for="opt-max-depth">Max Depth</Label>
            <Input
              id="opt-max-depth"
              type="number"
              value={crawlOptions.maxDepth}
              oninput={(e) => onCrawlOptionChange('maxDepth', parseInt(e.currentTarget.value) || 3)}
              disabled={isSubmitting}
            />
          </div>
          <div class="space-y-2">
            <Label for="opt-concurrency">Concurrency</Label>
            <Input
              id="opt-concurrency"
              type="number"
              value={crawlOptions.maxConcurrency}
              oninput={(e) => onCrawlOptionChange('maxConcurrency', parseInt(e.currentTarget.value) || 4)}
              disabled={isSubmitting}
            />
          </div>
        </div>
        <div class="grid gap-3 sm:grid-cols-2">
          <div class="flex items-center space-x-2">
            <Checkbox
              id="opt-external"
              checked={crawlOptions.followExternalLinks}
              onCheckedChange={(v) => onCrawlOptionChange('followExternalLinks', Boolean(v))}
              disabled={isSubmitting}
            />
            <Label for="opt-external" class="text-sm font-normal">Follow external links</Label>
          </div>
          <div class="flex items-center space-x-2">
            <Checkbox
              id="opt-subdomain"
              checked={crawlOptions.sameSubdomainOnly}
              onCheckedChange={(v) => onCrawlOptionChange('sameSubdomainOnly', Boolean(v))}
              disabled={isSubmitting}
            />
            <Label for="opt-subdomain" class="text-sm font-normal">Same subdomain only</Label>
          </div>
        </div>
      </div>
    {/if}
    
    <!-- Submit Button -->
    <Button class="w-full" onclick={onSubmit} disabled={isSubmitting || !isValid}>
      {#if isSubmitting}
        <Loader2 class="mr-2 h-4 w-4 animate-spin" />
        Analyzing...
      {:else}
        <svelte:component this={typeConfig.icon} class="mr-2 h-4 w-4" />
        Start {type === 'single' ? 'Analysis' : type === 'crawl' ? 'Crawl' : 'Site Files Check'}
      {/if}
    </Button>
  </CardContent>
</Card>
