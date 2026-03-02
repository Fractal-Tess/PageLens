<script lang="ts">
  import { cn } from '$lib/utils'

  let { score }: { score: number } = $props()

  const rounded = $derived(Math.round(score))
  const circumference = 2 * Math.PI * 54
  const offset = $derived(circumference - (score / 100) * circumference)

  const color = $derived(
    score >= 80
      ? 'text-green-500'
      : score >= 50
        ? 'text-amber-500'
        : 'text-red-500'
  )

  const trackColor = $derived(
    score >= 80
      ? 'stroke-green-500/20'
      : score >= 50
        ? 'stroke-amber-500/20'
        : 'stroke-red-500/20'
  )
</script>

<div class="relative inline-flex items-center justify-center">
  <svg class="h-32 w-32 -rotate-90" viewBox="0 0 120 120">
    <circle
      cx="60"
      cy="60"
      r="54"
      fill="none"
      class={trackColor}
      stroke-width="8"
    />
    <circle
      cx="60"
      cy="60"
      r="54"
      fill="none"
      stroke="currentColor"
      stroke-width="8"
      stroke-linecap="round"
      stroke-dasharray={circumference}
      stroke-dashoffset={offset}
      class={cn(color, 'transition-all duration-500 ease-out')}
    />
  </svg>
  <div class="absolute flex flex-col items-center">
    <span class={cn('text-3xl font-bold', color)}>{rounded}</span>
    <span class="text-xs text-muted-foreground">/ 100</span>
  </div>
</div>
