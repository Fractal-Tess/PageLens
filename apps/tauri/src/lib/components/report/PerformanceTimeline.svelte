<script lang="ts">
  import type { PerformanceTiming } from '$lib/types'

  let { timing }: { timing: PerformanceTiming } = $props()

  function formatMs(value: number | null, base: number): string {
    if (value == null) return '—'
    const ms = value - base
    if (ms < 1000) return `${ms}ms`
    return `${(ms / 1000).toFixed(2)}s`
  }

  const navStart = $derived(timing.navigation_start)

  const metrics = $derived([
    {
      label: 'TTFB',
      description: 'Time to First Byte',
      value: timing.response_start,
      color: 'bg-blue-500'
    },
    {
      label: 'FP',
      description: 'First Paint',
      value: timing.first_paint,
      color: 'bg-green-500'
    },
    {
      label: 'FCP',
      description: 'First Contentful Paint',
      value: timing.first_contentful_paint,
      color: 'bg-teal-500'
    },
    {
      label: 'DOM Interactive',
      description: 'DOM Interactive',
      value: timing.dom_interactive,
      color: 'bg-amber-500'
    },
    {
      label: 'DCL',
      description: 'DOM Content Loaded',
      value: timing.dom_content_loaded,
      color: 'bg-orange-500'
    },
    {
      label: 'Load',
      description: 'Load Complete',
      value: timing.load_complete,
      color: 'bg-red-500'
    }
  ])

  const maxValue = $derived(
    Math.max(
      ...metrics
        .map(m => (m.value != null ? m.value - navStart : 0))
        .filter(v => v > 0),
      1
    )
  )
</script>

<div class="space-y-3">
  {#each metrics as metric}
    {@const ms = metric.value != null ? metric.value - navStart : null}
    {@const width = ms != null ? Math.max((ms / maxValue) * 100, 2) : 0}
    <div class="space-y-1">
      <div class="flex items-center justify-between text-sm">
        <span class="font-medium">{metric.label}</span>
        <span class="text-muted-foreground tabular-nums">
          {formatMs(metric.value, navStart)}
        </span>
      </div>
      {#if ms != null}
        <div class="h-2 w-full rounded-full bg-muted">
          <div
            class="h-full rounded-full transition-all duration-300 {metric.color}"
            style="width: {width}%"
          ></div>
        </div>
      {:else}
        <div class="h-2 w-full rounded-full bg-muted"></div>
      {/if}
    </div>
  {/each}
</div>
