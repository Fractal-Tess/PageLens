<script lang="ts">
  import { Card, CardContent } from '../../ui/card';
  import { getScoreColor, formatDuration } from '../../../stores/report.svelte';
  import type { CrawlStats, AggregateMetrics } from './types';
  import { cn } from '../../../utils';
  
  interface Props {
    stats: CrawlStats;
    aggregate: AggregateMetrics;
    class?: string;
  }
  
  let { stats, aggregate, class: className = '' }: Props = $props();
  
  const maxBucket = $derived(Math.max(...aggregate.score_distribution, 1));
  
  const scoreCards = $derived([
    { label: 'Average', value: Math.round(aggregate.avg_seo_score) },
    { label: 'Min', value: Math.round(aggregate.min_seo_score) },
    { label: 'Max', value: Math.round(aggregate.max_seo_score) },
  ]);
  
  const statItems = $derived([
    { label: 'Pages Crawled', value: stats.crawled_pages },
    { label: 'Failed', value: stats.failed_pages },
    { label: 'External Links', value: stats.external_links },
    { label: 'Crawl Time', value: formatDuration(stats.crawl_time_ms) },
  ]);
</script>

<div class="space-y-6 {className}">
  <!-- Score cards -->
  <div class="grid grid-cols-3 gap-4">
    {#each scoreCards as card}
      <Card>
        <CardContent class="pt-4 text-center">
          <p class="text-xs text-muted-foreground">{card.label}</p>
          <p class={cn('text-2xl font-bold', getScoreColor(card.value))}>
            {card.value}
          </p>
        </CardContent>
      </Card>
    {/each}
  </div>
  
  <!-- Crawl stats -->
  <div class="grid grid-cols-2 gap-4 sm:grid-cols-4">
    {#each statItems as stat}
      <div class="rounded-lg bg-muted p-3 text-center">
        <p class="text-xs text-muted-foreground">{stat.label}</p>
        <p class="text-lg font-bold">{stat.value}</p>
      </div>
    {/each}
  </div>
  
  <!-- Score distribution -->
  {#if aggregate.score_distribution.length > 0}
    <div class="space-y-2">
      <h4 class="text-sm font-medium">Score Distribution</h4>
      <div class="flex items-end gap-1 h-24">
        {#each aggregate.score_distribution as count, i}
          {@const height = (count / maxBucket) * 100}
          {@const midScore = i * 10 + 5}
          <div class="flex-1 flex flex-col items-center gap-1">
            <div
              class={cn(
                'w-full rounded-t transition-all',
                midScore >= 80 ? 'bg-green-500' : midScore >= 50 ? 'bg-amber-500' : 'bg-red-500'
              )}
              style="height: {Math.max(height, 2)}%"
              title="{i * 10}-{(i + 1) * 10}: {count} pages"
            ></div>
          </div>
        {/each}
      </div>
      <div class="flex gap-1 text-[10px] text-muted-foreground">
        {#each aggregate.score_distribution as _, i}
          <div class="flex-1 text-center">{i * 10}</div>
        {/each}
      </div>
    </div>
  {/if}
</div>
