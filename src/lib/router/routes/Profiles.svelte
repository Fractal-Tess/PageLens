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
  import { Badge } from '$components/ui/badge'
  import { Separator } from '$components/ui/separator'
  import { toast } from 'svelte-sonner'
  import { Settings2, Copy, Trash2, Star, Plus } from '@lucide/svelte'
  import {
    createBlankProfile,
    loadDefaultProfileId,
    loadProfiles,
    saveDefaultProfileId,
    saveProfiles,
    type AnalysisProfile,
    type RunKind
  } from '$lib/analysis-profiles'

  let profiles = $state<AnalysisProfile[]>([])
  let selectedId = $state<string | null>(null)
  let defaultProfileId = $state<string | null>(null)

  const selectedProfile = $derived(
    profiles.find(profile => profile.id === selectedId) ?? null
  )

  onMount(() => {
    profiles = loadProfiles()
    defaultProfileId = loadDefaultProfileId()
    selectedId = profiles[0]?.id ?? null
  })

  function persistProfiles(nextProfiles: AnalysisProfile[]) {
    profiles = nextProfiles
    saveProfiles(nextProfiles)
  }

  function updateSelected(
    mutator: (profile: AnalysisProfile) => AnalysisProfile
  ) {
    if (!selectedId) return
    const next = profiles.map(profile => {
      if (profile.id !== selectedId) return profile
      return {
        ...mutator(profile),
        updatedAt: new Date().toISOString()
      }
    })
    persistProfiles(next)
  }

  function createProfile() {
    const profile = createBlankProfile()
    profile.name = `Profile ${profiles.length + 1}`
    persistProfiles([profile, ...profiles])
    selectedId = profile.id
    toast.success('Profile created')
  }

  function duplicateProfile() {
    if (!selectedProfile) return
    const source = selectedProfile
    const clone: AnalysisProfile = {
      ...source,
      id: crypto.randomUUID(),
      name: `${source.name} Copy`,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString()
    }
    persistProfiles([clone, ...profiles])
    selectedId = clone.id
    toast.success('Profile duplicated')
  }

  function deleteProfile() {
    if (!selectedId) return
    const next = profiles.filter(profile => profile.id !== selectedId)
    persistProfiles(next)

    if (defaultProfileId === selectedId) {
      defaultProfileId = null
      saveDefaultProfileId(null)
    }

    selectedId = next[0]?.id ?? null
    toast.success('Profile deleted')
  }

  function setDefault() {
    if (!selectedId) return
    defaultProfileId = selectedId
    saveDefaultProfileId(selectedId)
    toast.success('Default profile updated')
  }

  function parseNumber(value: string, fallback: number): number {
    const parsed = Number.parseInt(value, 10)
    return Number.isFinite(parsed) ? parsed : fallback
  }

  function setRunKind(value: string) {
    const runKind: RunKind =
      value === 'single' || value === 'crawl' || value === 'site_files'
        ? value
        : 'crawl'
    updateSelected(profile => ({ ...profile, runKind }))
  }
</script>

<div class="h-full overflow-auto p-6 space-y-6">
  <div>
    <h1 class="text-3xl font-bold tracking-tight">Profiles</h1>
    <p class="text-sm text-muted-foreground">
      Save reusable analysis settings and prefill Advanced runs.
    </p>
  </div>

  <div class="grid gap-6 lg:grid-cols-[320px_minmax(0,1fr)]">
    <Card>
      <CardHeader>
        <CardTitle class="flex items-center gap-2">
          <Settings2 class="h-5 w-5 text-indigo-500" /> Saved profiles
        </CardTitle>
      </CardHeader>
      <CardContent class="space-y-3">
        <Button class="w-full" onclick={createProfile}>
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
                onclick={() => (selectedId = profile.id)}
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
                  updateSelected(profile => ({
                    ...profile,
                    name: (event.currentTarget as HTMLInputElement).value
                  }))}
              />
            </div>

            <div class="space-y-2 sm:col-span-2">
              <Label>Description</Label>
              <Input
                value={selectedProfile.description}
                oninput={event =>
                  updateSelected(profile => ({
                    ...profile,
                    description: (event.currentTarget as HTMLInputElement).value
                  }))}
              />
            </div>

            <div class="space-y-2 sm:col-span-2">
              <Label>Default run mode</Label>
              <select
                class="w-full rounded-md border bg-background px-3 py-2 text-sm"
                value={selectedProfile.runKind}
                onchange={event =>
                  setRunKind((event.currentTarget as HTMLSelectElement).value)}
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
                    updateSelected(profile => ({
                      ...profile,
                      singleOptions: {
                        ...profile.singleOptions,
                        includeHtml: Boolean(checked)
                      }
                    }))}
                />
                <Label class="text-sm font-normal">Include HTML</Label>
              </div>
              <div class="flex items-center space-x-2">
                <Checkbox
                  checked={selectedProfile.singleOptions
                    .includeAccessibilityTree}
                  onCheckedChange={checked =>
                    updateSelected(profile => ({
                      ...profile,
                      singleOptions: {
                        ...profile.singleOptions,
                        includeAccessibilityTree: Boolean(checked)
                      }
                    }))}
                />
                <Label class="text-sm font-normal">Accessibility tree</Label>
              </div>
              <div class="flex items-center space-x-2">
                <Checkbox
                  checked={selectedProfile.singleOptions
                    .includePerformanceTiming}
                  onCheckedChange={checked =>
                    updateSelected(profile => ({
                      ...profile,
                      singleOptions: {
                        ...profile.singleOptions,
                        includePerformanceTiming: Boolean(checked)
                      }
                    }))}
                />
                <Label class="text-sm font-normal">Performance timing</Label>
              </div>
              <div class="flex items-center space-x-2">
                <Checkbox
                  checked={selectedProfile.singleOptions.includeComputedStyles}
                  onCheckedChange={checked =>
                    updateSelected(profile => ({
                      ...profile,
                      singleOptions: {
                        ...profile.singleOptions,
                        includeComputedStyles: Boolean(checked)
                      }
                    }))}
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
                    updateSelected(profile => ({
                      ...profile,
                      crawlOptions: {
                        ...profile.crawlOptions,
                        maxPages: parseNumber(
                          (event.currentTarget as HTMLInputElement).value,
                          profile.crawlOptions.maxPages
                        )
                      }
                    }))}
                />
              </div>
              <div class="space-y-2">
                <Label>Max depth</Label>
                <Input
                  type="number"
                  value={selectedProfile.crawlOptions.maxDepth}
                  oninput={event =>
                    updateSelected(profile => ({
                      ...profile,
                      crawlOptions: {
                        ...profile.crawlOptions,
                        maxDepth: parseNumber(
                          (event.currentTarget as HTMLInputElement).value,
                          profile.crawlOptions.maxDepth
                        )
                      }
                    }))}
                />
              </div>
              <div class="space-y-2">
                <Label>Concurrency</Label>
                <Input
                  type="number"
                  value={selectedProfile.crawlOptions.maxConcurrency}
                  oninput={event =>
                    updateSelected(profile => ({
                      ...profile,
                      crawlOptions: {
                        ...profile.crawlOptions,
                        maxConcurrency: parseNumber(
                          (event.currentTarget as HTMLInputElement).value,
                          profile.crawlOptions.maxConcurrency
                        )
                      }
                    }))}
                />
              </div>
              <div class="space-y-2">
                <Label>Timeout (ms)</Label>
                <Input
                  type="number"
                  value={selectedProfile.crawlOptions.pageTimeoutMs}
                  oninput={event =>
                    updateSelected(profile => ({
                      ...profile,
                      crawlOptions: {
                        ...profile.crawlOptions,
                        pageTimeoutMs: parseNumber(
                          (event.currentTarget as HTMLInputElement).value,
                          profile.crawlOptions.pageTimeoutMs
                        )
                      }
                    }))}
                />
              </div>
              <div class="space-y-2">
                <Label>Delay (ms)</Label>
                <Input
                  type="number"
                  value={selectedProfile.crawlOptions.delayMs}
                  oninput={event =>
                    updateSelected(profile => ({
                      ...profile,
                      crawlOptions: {
                        ...profile.crawlOptions,
                        delayMs: parseNumber(
                          (event.currentTarget as HTMLInputElement).value,
                          profile.crawlOptions.delayMs
                        )
                      }
                    }))}
                />
              </div>
            </div>

            <div class="grid gap-3 sm:grid-cols-2">
              <div class="flex items-center space-x-2">
                <Checkbox
                  checked={selectedProfile.crawlOptions.followExternalLinks}
                  onCheckedChange={checked =>
                    updateSelected(profile => ({
                      ...profile,
                      crawlOptions: {
                        ...profile.crawlOptions,
                        followExternalLinks: Boolean(checked)
                      }
                    }))}
                />
                <Label class="text-sm font-normal">Follow external links</Label>
              </div>
              <div class="flex items-center space-x-2">
                <Checkbox
                  checked={selectedProfile.crawlOptions.sameSubdomainOnly}
                  onCheckedChange={checked =>
                    updateSelected(profile => ({
                      ...profile,
                      crawlOptions: {
                        ...profile.crawlOptions,
                        sameSubdomainOnly: Boolean(checked)
                      }
                    }))}
                />
                <Label class="text-sm font-normal">Same subdomain only</Label>
              </div>
            </div>
          </div>

          <Separator />

          <div class="flex flex-wrap gap-2">
            <Button variant="secondary" onclick={duplicateProfile}>
              <Copy class="mr-2 h-4 w-4" /> Duplicate
            </Button>
            <Button variant="secondary" onclick={setDefault}>
              <Star class="mr-2 h-4 w-4" /> Set default
            </Button>
            <Button variant="destructive" onclick={deleteProfile}>
              <Trash2 class="mr-2 h-4 w-4" /> Delete
            </Button>
          </div>
        </CardContent>
      </Card>
    {/if}
  </div>
</div>
