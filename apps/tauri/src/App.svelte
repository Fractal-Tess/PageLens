<script lang="ts">
  import Router from '$router/Router.svelte';
  import { ModeWatcher, mode } from 'mode-watcher';
  import { Toaster } from 'svelte-sonner';
  import { AppShell } from '@pagelens/ui';
  import AppSidebar from '$lib/components/AppSidebar.svelte';
  import AppHeader from '$lib/components/AppHeader.svelte';
</script>

<ModeWatcher defaultMode="dark" />
<Toaster theme={mode.current} />

<AppShell>
  {#snippet sidebar()}
    <AppSidebar />
  {/snippet}
  {#snippet header()}
    <AppHeader />
  {/snippet}
  {#snippet children()}
    <Router />
  {/snippet}
</AppShell>
