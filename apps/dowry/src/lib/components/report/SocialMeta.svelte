<script lang="ts">
  import type { OpenGraphInfo, TwitterCardInfo } from '$lib/types'
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle
  } from '@pagelens/ui/shadcn/card'
  import { Check, X } from '@lucide/svelte'

  let {
    og,
    twitter
  }: { og: OpenGraphInfo; twitter: TwitterCardInfo } = $props()

  function fieldRow(label: string, value: string | null) {
    return { label, value }
  }
</script>

<div class="grid gap-4 md:grid-cols-2">
  <!-- Open Graph -->
  <Card>
    <CardHeader class="pb-3">
      <CardTitle class="text-sm">Open Graph</CardTitle>
    </CardHeader>
    <CardContent class="space-y-2">
      {#each [
        fieldRow('Title', og.title),
        fieldRow('Description', og.description),
        fieldRow('Type', og.og_type),
        fieldRow('URL', og.url),
        fieldRow('Image', og.image)
      ] as { label, value }}
        <div class="flex items-start gap-2 text-sm">
          {#if value}
            <Check class="mt-0.5 h-3.5 w-3.5 shrink-0 text-green-500" />
          {:else}
            <X class="mt-0.5 h-3.5 w-3.5 shrink-0 text-muted-foreground" />
          {/if}
          <div class="min-w-0">
            <span class="font-medium">{label}: </span>
            <span class="break-all text-muted-foreground"
              >{value ?? 'Not set'}</span
            >
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
      {#each [
        fieldRow('Card type', twitter.card),
        fieldRow('Title', twitter.title),
        fieldRow('Description', twitter.description),
        fieldRow('Image', twitter.image)
      ] as { label, value }}
        <div class="flex items-start gap-2 text-sm">
          {#if value}
            <Check class="mt-0.5 h-3.5 w-3.5 shrink-0 text-green-500" />
          {:else}
            <X class="mt-0.5 h-3.5 w-3.5 shrink-0 text-muted-foreground" />
          {/if}
          <div class="min-w-0">
            <span class="font-medium">{label}: </span>
            <span class="break-all text-muted-foreground"
              >{value ?? 'Not set'}</span
            >
          </div>
        </div>
      {/each}
    </CardContent>
  </Card>
</div>
