<script lang="ts">
  import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '../../ui/card';
  import { Badge } from '../../ui/badge';
  import { Separator } from '../../ui/separator';
  import { CheckCircle2, Gauge, Layout, AlertCircle, Clock } from '@lucide/svelte';
  import { formatDuration, getScoreColor } from '../../../stores/report.svelte';
  import { Button } from '../../ui/button';
  import { Layers } from '@lucide/svelte';
  
  interface Summary {
    seo_score: number | null;
    page_count: number;
    error_count: number;
    warning_count: number;
    total_issues: number;
    duration_ms: number;
  }
  
  interface Props {
    name: string | null;
    createdAt: string;
    analysisType: 'single' | 'crawl';
    summary: Summary;
    onViewDetails?: () => void;
    class?: string;
  }
  
  let { name, createdAt, analysisType, summary, onViewDetails, class: className = '' }: Props = $props();
  
  function formatDate(iso: string): string {
    return new Date(iso).toLocaleString();
  }
  
  const stats = $derived([
    { icon: Gauge, label: 'SEO Score', value: summary.seo_score ? Math.round(summary.seo_score) : '-', iconClass: 'text-indigo-500' },
    { icon: Layout, label: 'Pages', value: summary.page_count, iconClass: 'text-violet-500' },
    { icon: AlertCircle, label: 'Errors', value: summary.error_count, iconClass: 'text-red-500' },
    { icon: Clock, label: 'Duration', value: formatDuration(summary.duration_ms), iconClass: 'text-amber-500' },
  ]);
</script>

<Card class={className}>
  <CardHeader>
    <CardTitle class="flex items-center gap-2">
      <CheckCircle2 class="h-5 w-5 text-green-500" />
      Analysis Complete
    </CardTitle>
    <CardDescription>
      {name || 'Unnamed analysis'} • {formatDate(createdAt)}
    </CardDescription>
  </CardHeader>
  <CardContent class="space-y-4">
    <!-- Summary Stats -->
    <div class="grid grid-cols-2 gap-4 sm:grid-cols-4">
      {#each stats as stat}
        <div class="flex flex-col items-center justify-center rounded-lg bg-muted p-3">
          <stat.icon class="mb-1 h-5 w-5 {stat.iconClass}" />
          <span class="text-2xl font-bold">{stat.value}</span>
          <span class="text-xs text-muted-foreground">{stat.label}</span>
        </div>
      {/each}
    </div>
    
    <Separator />
    
    <!-- Issues Breakdown -->
    <div class="space-y-2">
      <div class="flex items-center justify-between">
        <span class="text-sm font-medium">Total Issues</span>
        <Badge variant="secondary">{summary.total_issues}</Badge>
      </div>
      <div class="flex items-center justify-between">
        <span class="text-sm font-medium">Warnings</span>
        <Badge variant="outline" class="border-amber-500 text-amber-600">
          {summary.warning_count}
        </Badge>
      </div>
    </div>
    
    <!-- Analysis Type Badge -->
    <div class="flex items-center gap-2">
      <span class="text-sm text-muted-foreground">Type:</span>
      <Badge variant={analysisType === 'single' ? 'default' : 'secondary'}>
        {analysisType === 'single' ? 'Single Page' : 'Crawl'}
      </Badge>
    </div>
    
    <!-- View Details -->
    {#if onViewDetails}
      <Button variant="outline" class="w-full" onclick={onViewDetails}>
        <Layers class="mr-2 h-4 w-4" />
        View Detailed Report
      </Button>
    {/if}
  </CardContent>
</Card>
