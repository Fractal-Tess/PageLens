<script lang="ts">
  import Router from '$router/Router.svelte'
  import { ModeWatcher, mode } from 'mode-watcher'
  import { Toaster } from 'svelte-sonner'
  import { SidebarProvider, SidebarInset } from '@pagelens/ui/shadcn/sidebar'
  import AppSidebar from '$lib/components/AppSidebar.svelte'
  import AppHeader from '$lib/components/AppHeader.svelte'
</script>

<ModeWatcher defaultMode="dark" />
<Toaster theme={mode.current} />

<SidebarProvider>
  <AppSidebar />
  <SidebarInset>
    <AppHeader />
    <main class="flex-1 overflow-auto">
      <Router />
    </main>
  </SidebarInset>
</SidebarProvider>
