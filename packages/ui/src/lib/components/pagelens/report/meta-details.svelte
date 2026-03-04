<script lang="ts">
  import { Check, X } from '@lucide/svelte';
  import type { MetaInfo } from './types';
  
  interface Props {
    meta: MetaInfo;
    canonical: string | null;
    class?: string;
  }
  
  let { meta, canonical, class: className = '' }: Props = $props();
  
  const titleLength = $derived(meta.title?.length ?? 0);
  const descLength = $derived(meta.description?.length ?? 0);
  
  const booleanFlags = $derived([
    { label: 'Charset', value: meta.charset },
    { label: 'Viewport', value: meta.viewport },
    { label: 'Language', value: !!meta.language },
    { label: 'Canonical', value: !!canonical },
  ]);
</script>

<div class="grid gap-4 {className}">
  <!-- Title -->
  <div class="space-y-1">
    <div class="flex items-center justify-between">
      <span class="text-sm font-medium">Title</span>
      <span class="text-xs text-muted-foreground">{titleLength} chars</span>
    </div>
    {#if meta.title}
      <p class="rounded-md bg-muted p-2 text-sm">{meta.title}</p>
      {#if titleLength < 30 || titleLength > 60}
        <p class="text-xs text-amber-500">Recommended length: 30-60 characters</p>
      {/if}
    {:else}
      <p class="text-sm text-red-500">Missing title tag</p>
    {/if}
  </div>
  
  <!-- Description -->
  <div class="space-y-1">
    <div class="flex items-center justify-between">
      <span class="text-sm font-medium">Description</span>
      <span class="text-xs text-muted-foreground">{descLength} chars</span>
    </div>
    {#if meta.description}
      <p class="rounded-md bg-muted p-2 text-sm">{meta.description}</p>
      {#if descLength < 50 || descLength > 160}
        <p class="text-xs text-amber-500">Recommended length: 50-160 characters</p>
      {/if}
    {:else}
      <p class="text-sm text-red-500">Missing meta description</p>
    {/if}
  </div>
  
  <!-- Boolean flags -->
  <div class="grid grid-cols-2 gap-3">
    {#each booleanFlags as item}
      <div class="flex items-center gap-2 text-sm">
        {#if item.value}
          <Check class="h-4 w-4 text-green-500" />
        {:else}
          <X class="h-4 w-4 text-red-500" />
        {/if}
        <span>{item.label}</span>
      </div>
    {/each}
  </div>
  
  {#if meta.language}
    <div class="text-sm">
      <span class="font-medium">Language:</span>
      <span class="text-muted-foreground">{meta.language}</span>
    </div>
  {/if}
  
  {#if meta.robots}
    <div class="text-sm">
      <span class="font-medium">Robots:</span>
      <span class="text-muted-foreground">{meta.robots}</span>
    </div>
  {/if}
  
  {#if canonical}
    <div class="text-sm">
      <span class="font-medium">Canonical URL:</span>
      <span class="text-muted-foreground break-all">{canonical}</span>
    </div>
  {/if}
</div>
