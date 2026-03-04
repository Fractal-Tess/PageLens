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
  import { Badge } from '../../ui/badge'
  import { Separator } from '../../ui/separator'
  import { Settings2, Copy, Trash2, Star, Plus } from '@lucide/svelte'

  type RunKind = 'single' | 'crawl' | 'site_files'

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

  interface ProfileModel {
    id: string
    name: string
    description: string
    runKind: RunKind
    singleOptions: SingleOptions
    crawlOptions: CrawlOptions
  }

  interface Props {
    profiles: ProfileModel[]
    selectedId: string | null
    defaultProfileId: string | null
    selectedProfile: ProfileModel | null
    onCreate: () => void
    onSelect: (id: string) => void
    onNameChange: (value: string) => void
    onDescriptionChange: (value: string) => void
    onRunKindChange: (value: string) => void
    onSingleToggle: (key: keyof SingleOptions, checked: boolean) => void
    onCrawlNumberChange: (key: keyof CrawlOptions, value: string) => void
    onCrawlToggle: (
      key: 'followExternalLinks' | 'sameSubdomainOnly',
      checked: boolean
    ) => void
    onDuplicate: () => void
    onSetDefault: () => void
    onDelete: () => void
  }

  let {
    profiles,
    selectedId,
    defaultProfileId,
    selectedProfile,
    onCreate,
    onSelect,
    onNameChange,
    onDescriptionChange,
    onRunKindChange,
    onSingleToggle,
    onCrawlNumberChange,
    onCrawlToggle,
    onDuplicate,
    onSetDefault,
    onDelete
  }: Props = $props()
</script>

<div class="grid gap-6 lg:grid-cols-[320px_minmax(0,1fr)]">
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2">
        <Settings2 class="h-5 w-5 text-indigo-500" /> Saved profiles
      </CardTitle>
    </CardHeader>
    <CardContent class="space-y-3">
      <Button class="w-full" onclick={onCreate}>
        <Plus class="mr-2 h-4 w-4" /> New profile
      </Button>

      <Separator />

      <div class="space-y-2">
        {#if profiles.length === 0}
          <p class="text-sm text-muted-foreground">No profiles yet.</p>
        {:else}
          {#each profiles as profile (profile.id)}
            <button
              type="button"
              class={`w-full rounded border px-3 py-2 text-left text-sm hover:bg-muted ${selectedId === profile.id ? 'border-indigo-500 bg-indigo-50/40 dark:bg-indigo-950/20' : ''}`}
              onclick={() => onSelect(profile.id)}
            >
              <div class="flex items-center justify-between gap-2">
                <span class="font-medium truncate">{profile.name}</span>
                {#if defaultProfileId === profile.id}
                  <Badge variant="secondary">Default</Badge>
                {/if}
              </div>
              <div class="mt-1 text-xs text-muted-foreground uppercase">
                {profile.runKind}
              </div>
            </button>
          {/each}
        {/if}
      </div>
    </CardContent>
  </Card>

  {#if selectedProfile}
    <Card>
      <CardHeader>
        <CardTitle>Profile editor</CardTitle>
        <CardDescription>Changes are saved automatically.</CardDescription>
      </CardHeader>
      <CardContent class="space-y-6">
        <div class="grid gap-4 sm:grid-cols-2">
          <div class="space-y-2 sm:col-span-2">
            <Label>Name</Label>
            <Input
              value={selectedProfile.name}
              oninput={event =>
                onNameChange((event.currentTarget as HTMLInputElement).value)}
            />
          </div>

          <div class="space-y-2 sm:col-span-2">
            <Label>Description</Label>
            <Input
              value={selectedProfile.description}
              oninput={event =>
                onDescriptionChange(
                  (event.currentTarget as HTMLInputElement).value
                )}
            />
          </div>

          <div class="space-y-2 sm:col-span-2">
            <Label>Default run mode</Label>
            <select
              class="w-full rounded-md border bg-background px-3 py-2 text-sm"
              value={selectedProfile.runKind}
              onchange={event =>
                onRunKindChange(
                  (event.currentTarget as HTMLSelectElement).value
                )}
            >
              <option value="single">Single</option>
              <option value="crawl">Crawl</option>
              <option value="site_files">Site files</option>
            </select>
          </div>
        </div>

        <Separator />

        <div class="space-y-3">
          <Label class="text-sm font-medium">Single-page options</Label>
          <div class="grid gap-3 sm:grid-cols-2">
            <div class="flex items-center space-x-2">
              <Checkbox
                checked={selectedProfile.singleOptions.includeHtml}
                onCheckedChange={checked =>
                  onSingleToggle('includeHtml', Boolean(checked))}
              />
              <Label class="text-sm font-normal">Include HTML</Label>
            </div>
            <div class="flex items-center space-x-2">
              <Checkbox
                checked={selectedProfile.singleOptions.includeAccessibilityTree}
                onCheckedChange={checked =>
                  onSingleToggle('includeAccessibilityTree', Boolean(checked))}
              />
              <Label class="text-sm font-normal">Accessibility tree</Label>
            </div>
            <div class="flex items-center space-x-2">
              <Checkbox
                checked={selectedProfile.singleOptions.includePerformanceTiming}
                onCheckedChange={checked =>
                  onSingleToggle('includePerformanceTiming', Boolean(checked))}
              />
              <Label class="text-sm font-normal">Performance timing</Label>
            </div>
            <div class="flex items-center space-x-2">
              <Checkbox
                checked={selectedProfile.singleOptions.includeComputedStyles}
                onCheckedChange={checked =>
                  onSingleToggle('includeComputedStyles', Boolean(checked))}
              />
              <Label class="text-sm font-normal">Computed styles</Label>
            </div>
          </div>
        </div>

        <Separator />

        <div class="space-y-3">
          <Label class="text-sm font-medium">Crawl options</Label>
          <div class="grid gap-4 sm:grid-cols-3">
            <div class="space-y-2">
              <Label>Max pages</Label>
              <Input
                type="number"
                value={selectedProfile.crawlOptions.maxPages}
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
                value={selectedProfile.crawlOptions.maxDepth}
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
                value={selectedProfile.crawlOptions.maxConcurrency}
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
                value={selectedProfile.crawlOptions.pageTimeoutMs}
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
                value={selectedProfile.crawlOptions.delayMs}
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
                checked={selectedProfile.crawlOptions.followExternalLinks}
                onCheckedChange={checked =>
                  onCrawlToggle('followExternalLinks', Boolean(checked))}
              />
              <Label class="text-sm font-normal">Follow external links</Label>
            </div>
            <div class="flex items-center space-x-2">
              <Checkbox
                checked={selectedProfile.crawlOptions.sameSubdomainOnly}
                onCheckedChange={checked =>
                  onCrawlToggle('sameSubdomainOnly', Boolean(checked))}
              />
              <Label class="text-sm font-normal">Same subdomain only</Label>
            </div>
          </div>
        </div>

        <Separator />

        <div class="flex flex-wrap gap-2">
          <Button variant="secondary" onclick={onDuplicate}>
            <Copy class="mr-2 h-4 w-4" /> Duplicate
          </Button>
          <Button variant="secondary" onclick={onSetDefault}>
            <Star class="mr-2 h-4 w-4" /> Set default
          </Button>
          <Button variant="destructive" onclick={onDelete}>
            <Trash2 class="mr-2 h-4 w-4" /> Delete
          </Button>
        </div>
      </CardContent>
    </Card>
  {/if}
</div>
