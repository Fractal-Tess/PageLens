<script lang="ts">
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from '../../ui/card';
  import { Check, X } from '@lucide/svelte';
  import type { OpenGraphInfo, TwitterCardInfo } from './types';
  
  interface Props {
    og: OpenGraphInfo;
    twitter: TwitterCardInfo;
    class?: string;
  }
  
  let { og, twitter, class: className = '' }: Props = $props();
  
  interface FieldRow {
    label: string;
    value: string | null;
  }
  
  const ogFields: FieldRow[] = $derived([
    { label: 'Title', value: og.title },
    { label: 'Description', value: og.description },
    { label: 'Type', value: og.og_type },
    { label: 'URL', value: og.url },
    { label: 'Image', value: og.image },
  ]);
  
  const twitterFields: FieldRow[] = $derived([
    { label: 'Card type', value: twitter.card },
    { label: 'Title', value: twitter.title },
    { label: 'Description', value: twitter.description },
    { label: 'Image', value: twitter.image },
  ]);
</script>

<div class="grid gap-4 md:grid-cols-2 {className}">
  <!-- Open Graph -->
  <Card>
    <CardHeader class="pb-3">
      <CardTitle class="text-sm">Open Graph</CardTitle>
    </CardHeader>
    <CardContent class="space-y-2">
      {#each ogFields as { label, value }}
        <div class="flex items-start gap-2 text-sm">
          {#if value}
            <Check class="mt-0.5 h-3.5 w-3.5 shrink-0 text-green-500" />
          {:else}
            <X class="mt-0.5 h-3.5 w-3.5 shrink-0 text-muted-foreground" />
          {/if}
          <div class="min-w-0">
            <span class="font-medium">{label}: </span>
            <span class="break-all text-muted-foreground">{value ?? 'Not set'}</span>
          </div>
        </div>
      {/each}
    </CardContent>
  </Card>
  
  <!-- Twitter Card -->
  <Card>
    <CardHeader class="pb-3">
      <CardTitle class="text-sm">Twitter Card</CardTitle>
    </CardHeader>
    <CardContent class="space-y-2">
      {#each twitterFields as { label, value }}
        <div class="flex items-start gap-2 text-sm">
          {#if value}
            <Check class="mt-0.5 h-3.5 w-3.5 shrink-0 text-green-500" />
          {:else}
            <X class="mt-0.5 h-3.5 w-3.5 shrink-0 text-muted-foreground" />
          {/if}
          <div class="min-w-0">
            <span class="font-medium">{label}: </span>
            <span class="break-all text-muted-foreground">{value ?? 'Not set'}</span>
          </div>
        </div>
      {/each}
    </CardContent>
  </Card>
</div>
