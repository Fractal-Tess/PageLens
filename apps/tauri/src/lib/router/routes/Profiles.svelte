<script lang="ts">
  import { onMount } from 'svelte';
  import { toast } from 'svelte-sonner';
  import { ProfilesManager } from '@pagelens/ui';
  import {
    createBlankProfile,
    loadDefaultProfileId,
    loadProfiles,
    saveDefaultProfileId,
    saveProfiles,
    type AnalysisProfile,
    type RunKind,
  } from '$lib/analysis-profiles';

  let profiles = $state<AnalysisProfile[]>([]);
  let selectedId = $state<string | null>(null);
  let defaultProfileId = $state<string | null>(null);

  const selectedProfile = $derived(profiles.find(profile => profile.id === selectedId) ?? null);

  onMount(() => {
    profiles = loadProfiles();
    defaultProfileId = loadDefaultProfileId();
    selectedId = profiles[0]?.id ?? null;
  });

  function persistProfiles(nextProfiles: AnalysisProfile[]) {
    profiles = nextProfiles;
    saveProfiles(nextProfiles);
  }

  function updateSelected(mutator: (profile: AnalysisProfile) => AnalysisProfile) {
    if (!selectedId) return;
    const next = profiles.map(profile => {
      if (profile.id !== selectedId) return profile;
      return {
        ...mutator(profile),
        updatedAt: new Date().toISOString(),
      };
    });
    persistProfiles(next);
  }

  function createProfile() {
    const profile = createBlankProfile();
    profile.name = `Profile ${profiles.length + 1}`;
    persistProfiles([profile, ...profiles]);
    selectedId = profile.id;
    toast.success('Profile created');
  }

  function duplicateProfile() {
    if (!selectedProfile) return;
    const source = selectedProfile;
    const clone: AnalysisProfile = {
      ...source,
      id: crypto.randomUUID(),
      name: `${source.name} Copy`,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
    persistProfiles([clone, ...profiles]);
    selectedId = clone.id;
    toast.success('Profile duplicated');
  }

  function deleteProfile() {
    if (!selectedId) return;
    const next = profiles.filter(profile => profile.id !== selectedId);
    persistProfiles(next);

    if (defaultProfileId === selectedId) {
      defaultProfileId = null;
      saveDefaultProfileId(null);
    }

    selectedId = next[0]?.id ?? null;
    toast.success('Profile deleted');
  }

  function setDefault() {
    if (!selectedId) return;
    defaultProfileId = selectedId;
    saveDefaultProfileId(selectedId);
    toast.success('Default profile updated');
  }

  function parseNumber(value: string, fallback: number): number {
    const parsed = Number.parseInt(value, 10);
    return Number.isFinite(parsed) ? parsed : fallback;
  }

  function setRunKind(value: string) {
    const runKind: RunKind = value === 'single' || value === 'crawl' || value === 'site_files' ? value : 'crawl';
    updateSelected(profile => ({ ...profile, runKind }));
  }

  function updateSingleOption(
    key: keyof AnalysisProfile['singleOptions'],
    checked: boolean,
  ) {
    updateSelected(profile => ({
      ...profile,
      singleOptions: {
        ...profile.singleOptions,
        [key]: checked,
      },
    }));
  }

  function updateCrawlNumber(
    key: keyof AnalysisProfile['crawlOptions'],
    value: string,
  ) {
    updateSelected(profile => ({
      ...profile,
      crawlOptions: {
        ...profile.crawlOptions,
        [key]: parseNumber(value, Number(profile.crawlOptions[key] ?? 0)),
      },
    }));
  }

  function updateCrawlToggle(
    key: 'followExternalLinks' | 'sameSubdomainOnly',
    checked: boolean,
  ) {
    updateSelected(profile => ({
      ...profile,
      crawlOptions: {
        ...profile.crawlOptions,
        [key]: checked,
      },
    }));
  }
</script>

<div class="h-full overflow-auto p-6 space-y-6">
  <div>
    <h1 class="text-3xl font-bold tracking-tight">Profiles</h1>
    <p class="text-sm text-muted-foreground">
      Save reusable analysis settings and prefill Advanced runs.
    </p>
  </div>

  <ProfilesManager
    {profiles}
    {selectedId}
    {defaultProfileId}
    {selectedProfile}
    onCreate={createProfile}
    onSelect={id => (selectedId = id)}
    onNameChange={value =>
      updateSelected(profile => ({
        ...profile,
        name: value,
      }))}
    onDescriptionChange={value =>
      updateSelected(profile => ({
        ...profile,
        description: value,
      }))}
    onRunKindChange={setRunKind}
    onSingleToggle={updateSingleOption}
    onCrawlNumberChange={updateCrawlNumber}
    onCrawlToggle={updateCrawlToggle}
    onDuplicate={duplicateProfile}
    onSetDefault={setDefault}
    onDelete={deleteProfile}
  />
</div>
