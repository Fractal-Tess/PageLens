<script lang="ts">
  import type { Snippet } from 'svelte';
  
  interface Props {
    title: string;
    description?: string;
    icon?: typeof import('@lucide/svelte').LayoutDashboard;
    iconClass?: string;
    actions?: Snippet;
    class?: string;
  }
  
  let {
    title,
    description,
    icon: Icon,
    iconClass = 'text-white',
    actions,
    class: className = ''
  }: Props = $props();
</script>

<div class="flex items-center gap-4 {className}">
  {#if Icon}
    <div class="flex h-12 w-12 items-center justify-center rounded-xl bg-gradient-to-br from-indigo-500 to-violet-600 shadow-lg">
      <Icon class="h-6 w-6 {iconClass}" />
    </div>
  {/if}
  <div class="flex-1">
    <h1 class="text-3xl font-bold tracking-tight">{title}</h1>
    {#if description}
      <p class="text-muted-foreground">{description}</p>
    {/if}
  </div>
  {#if actions}
    <div class="flex items-center gap-2">
      {@render actions()}
    </div>
  {/if}
</div>
