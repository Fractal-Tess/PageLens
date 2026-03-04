<script lang="ts">
  import { SidebarProvider, SidebarInset } from '../../ui/sidebar';
  import type { Snippet } from 'svelte';
  
  interface Props {
    sidebar: Snippet;
    header: Snippet;
    children: Snippet;
  }
  
  let { sidebar, header, children }: Props = $props();
</script>

<SidebarProvider>
  {@render sidebar()}
  <SidebarInset>
    {@render header()}
    <main class="flex-1 overflow-auto">
      {@render children()}
    </main>
  </SidebarInset>
</SidebarProvider>
