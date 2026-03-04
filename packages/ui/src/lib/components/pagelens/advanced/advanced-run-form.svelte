<script lang="ts">
  import { Button } from '../../ui/button'
  import { Input } from '../../ui/input'
  import { Label } from '../../ui/label'
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle
  } from '../../ui/card'
  import { Checkbox } from '../../ui/checkbox'
  import { Separator } from '../../ui/separator'
  import { Badge } from '../../ui/badge'
  import {
    Scan,
    Globe,
    Layers,
    LoaderCircle,
    WandSparkles,
    RotateCcw
  } from '@lucide/svelte'

  type RunKind = 'single' | 'crawl' | 'site_files'

  interface ProfileSummary {
    id: string
    name: string
  }

  interface SingleOptions {
    includeHtml: boolean
    includeAccessibilityTree: boolean
    includePerformanceTiming: boolean
    includeComputedStyles: boolean
  }

  interface CrawlOptions {
    maxPages: number
    maxDepth: number
    maxConcurrency: number
    pageTimeoutMs: number
    delayMs: number
    followExternalLinks: boolean
    sameSubdomainOnly: boolean
  }

  interface Props {
    profiles: ProfileSummary[]
    selectedProfileId: string
    selectedProfileName: string | null
    url: string
    name: string
    runKind: RunKind
    isRunning: boolean
    singleOptions: SingleOptions
    crawlOptions: CrawlOptions
    onSelectProfile: (id: string) => void
    onResetToProfile: () => void
    onUrlChange: (value: string) => void
    onNameChange: (value: string) => void
    onRunKindChange: (value: RunKind) => void
    onSingleToggle: (key: keyof SingleOptions, checked: boolean) => void
    onCrawlNumberChange: (key: keyof CrawlOptions, value: string) => void
    onCrawlToggle: (
      key: 'followExternalLinks' | 'sameSubdomainOnly',
      checked: boolean
    ) => void
    onStartRun: () => void
  }

  let {
    profiles,
    selectedProfileId,
    selectedProfileName,
    url,
    name,
    runKind,
    isRunning,
    singleOptions,
    crawlOptions,
    onSelectProfile,
    onResetToProfile,
    onUrlChange,
    onNameChange,
    onRunKindChange,
    onSingleToggle,
    onCrawlNumberChange,
    onCrawlToggle,
    onStartRun
  }: Props = $props()
</script>

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
          onSelectProfile((event.currentTarget as HTMLSelectElement).value)}
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
        onclick={onResetToProfile}
        disabled={!selectedProfileId}
      >
        <RotateCcw class="mr-2 h-4 w-4" /> Reset to Profile
      </Button>
    </div>

    <div class="self-end flex items-center">
      {#if selectedProfileName}
        <Badge variant="secondary">Using: {selectedProfileName}</Badge>
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
          value={url}
          disabled={isRunning}
          oninput={event =>
            onUrlChange((event.currentTarget as HTMLInputElement).value)}
          placeholder="https://example.com"
        />
      </div>
      <div class="space-y-2">
        <Label for="advanced-name">Run name (optional)</Label>
        <Input
          id="advanced-name"
          value={name}
          disabled={isRunning}
          oninput={event =>
            onNameChange((event.currentTarget as HTMLInputElement).value)}
          placeholder="My advanced run"
        />
      </div>
    </div>

    <div class="space-y-2">
      <Label for="advanced-kind">Run type</Label>
      <select
        id="advanced-kind"
        class="w-full rounded-md border bg-background px-3 py-2 text-sm"
        value={runKind}
        onchange={event =>
          onRunKindChange(
            (event.currentTarget as HTMLSelectElement).value as RunKind
          )}
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
              checked={singleOptions.includeHtml}
              id="adv-html"
              disabled={isRunning}
              onCheckedChange={v => onSingleToggle('includeHtml', Boolean(v))}
            />
            <Label for="adv-html" class="text-sm font-normal"
              >Include HTML</Label
            >
          </div>
          <div class="flex items-center space-x-2">
            <Checkbox
              checked={singleOptions.includeAccessibilityTree}
              id="adv-a11y"
              disabled={isRunning}
              onCheckedChange={v =>
                onSingleToggle('includeAccessibilityTree', Boolean(v))}
            />
            <Label for="adv-a11y" class="text-sm font-normal"
              >Accessibility tree</Label
            >
          </div>
          <div class="flex items-center space-x-2">
            <Checkbox
              checked={singleOptions.includePerformanceTiming}
              id="adv-perf"
              disabled={isRunning}
              onCheckedChange={v =>
                onSingleToggle('includePerformanceTiming', Boolean(v))}
            />
            <Label for="adv-perf" class="text-sm font-normal"
              >Performance timing</Label
            >
          </div>
          <div class="flex items-center space-x-2">
            <Checkbox
              checked={singleOptions.includeComputedStyles}
              id="adv-styles"
              disabled={isRunning}
              onCheckedChange={v =>
                onSingleToggle('includeComputedStyles', Boolean(v))}
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
              value={crawlOptions.maxPages}
              disabled={isRunning}
              oninput={event =>
                onCrawlNumberChange(
                  'maxPages',
                  (event.currentTarget as HTMLInputElement).value
                )}
            />
          </div>
          <div class="space-y-2">
            <Label>Max depth</Label>
            <Input
              type="number"
              value={crawlOptions.maxDepth}
              disabled={isRunning}
              oninput={event =>
                onCrawlNumberChange(
                  'maxDepth',
                  (event.currentTarget as HTMLInputElement).value
                )}
            />
          </div>
          <div class="space-y-2">
            <Label>Concurrency</Label>
            <Input
              type="number"
              value={crawlOptions.maxConcurrency}
              disabled={isRunning}
              oninput={event =>
                onCrawlNumberChange(
                  'maxConcurrency',
                  (event.currentTarget as HTMLInputElement).value
                )}
            />
          </div>
          <div class="space-y-2">
            <Label>Timeout (ms)</Label>
            <Input
              type="number"
              value={crawlOptions.pageTimeoutMs}
              disabled={isRunning}
              oninput={event =>
                onCrawlNumberChange(
                  'pageTimeoutMs',
                  (event.currentTarget as HTMLInputElement).value
                )}
            />
          </div>
          <div class="space-y-2">
            <Label>Delay (ms)</Label>
            <Input
              type="number"
              value={crawlOptions.delayMs}
              disabled={isRunning}
              oninput={event =>
                onCrawlNumberChange(
                  'delayMs',
                  (event.currentTarget as HTMLInputElement).value
                )}
            />
          </div>
        </div>
        <div class="grid gap-3 sm:grid-cols-2">
          <div class="flex items-center space-x-2">
            <Checkbox
              checked={crawlOptions.followExternalLinks}
              id="adv-external"
              disabled={isRunning}
              onCheckedChange={v =>
                onCrawlToggle('followExternalLinks', Boolean(v))}
            />
            <Label for="adv-external" class="text-sm font-normal"
              >Follow external links</Label
            >
          </div>
          <div class="flex items-center space-x-2">
            <Checkbox
              checked={crawlOptions.sameSubdomainOnly}
              id="adv-subdomain"
              disabled={isRunning}
              onCheckedChange={v =>
                onCrawlToggle('sameSubdomainOnly', Boolean(v))}
            />
            <Label for="adv-subdomain" class="text-sm font-normal"
              >Same subdomain only</Label
            >
          </div>
        </div>
      </div>
    {/if}

    {#if runKind === 'site_files'}
      <div class="rounded border bg-muted/40 p-4 text-sm text-muted-foreground">
        Site-files mode checks robots.txt and sitemap files for this URL.
      </div>
    {/if}

    <Button
      class="w-full"
      onclick={onStartRun}
      disabled={isRunning || !url.trim()}
    >
      {#if isRunning}
        <LoaderCircle class="mr-2 h-4 w-4 animate-spin" /> Running advanced analysis...
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
