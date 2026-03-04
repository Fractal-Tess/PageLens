<script lang="ts">
  import { Card, CardContent } from '../../ui/card';
  import type { Component, Snippet } from 'svelte';
  
  interface Props {
    title: string;
    description?: string;
    icon?: typeof import('@lucide/svelte').LayoutDashboard;
    action?: Snippet;
    class?: string;
  }
  
  let {
    title,
    description,
    icon: Icon,
    action,
    class: className = ''
  }: Props = $props();
</script>

<Card class="border-dashed {className}">
  <CardContent class="flex flex-col items-center justify-center py-12 text-center">
    {#if Icon}
      <div class="flex h-16 w-16 items-center justify-center rounded-full bg-muted mb-4">
        <Icon class="h-8 w-8 text-muted-foreground" />
      </div>
    {/if}
    <h3 class="text-lg font-semibold">{title}</h3>
    {#if description}
      <p class="text-sm text-muted-foreground mt-1">{description}</p>
    {/if}
    {#if action}
      <div class="mt-4">
        {@render action()}
      </div>
    {/if}
  </CardContent>
</Card>
