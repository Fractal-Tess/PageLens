<script lang="ts">
  import { Card, CardContent, CardHeader, CardTitle } from '../../ui/card';
  import { Progress } from '../../ui/progress';
  import { Loader2, CheckCircle2, AlertCircle, Clock } from '@lucide/svelte';
  import type { AnalysisStatus } from '../../../stores/analysis.svelte';
  
  interface Props {
    status: AnalysisStatus;
    stage: string;
    message: string;
    progress: number;
    class?: string;
  }
  
  let { status, stage, message, progress, class: className = '' }: Props = $props();
  
  const statusConfig = $derived({
    idle: { icon: Clock, colorClass: '' },
    pending: { icon: Clock, colorClass: 'border-indigo-200 bg-indigo-50/50 dark:border-indigo-900 dark:bg-indigo-950/20' },
    running: { icon: Loader2, colorClass: 'border-indigo-200 bg-indigo-50/50 dark:border-indigo-900 dark:bg-indigo-950/20' },
    completed: { icon: CheckCircle2, colorClass: 'border-green-200 bg-green-50/50 dark:border-green-900 dark:bg-green-950/20' },
    failed: { icon: AlertCircle, colorClass: 'border-red-200 bg-red-50/50 dark:border-red-900 dark:bg-red-950/20' },
  }[status]);
  
  const titleColor = $derived(
    status === 'failed' ? 'text-red-700 dark:text-red-300' :
    status === 'completed' ? 'text-green-700 dark:text-green-300' :
    'text-indigo-700 dark:text-indigo-300'
  );
</script>

{#if status !== 'idle'}
  <Card class="{statusConfig.colorClass} {className}">
    <CardHeader>
      <CardTitle class="flex items-center gap-2 {titleColor}">
        <svelte:component this={statusConfig.icon} class="h-5 w-5 {status === 'running' ? 'animate-spin' : ''}" />
        {#if status === 'running'}
          Analysis in Progress
        {:else if status === 'completed'}
          Analysis Complete
        {:else if status === 'failed'}
          Analysis Failed
        {:else}
          Analysis Pending
        {/if}
      </CardTitle>
    </CardHeader>
    <CardContent class="space-y-4">
      <div class="space-y-2">
        <div class="flex justify-between text-sm">
          <span class="font-medium capitalize">{stage}</span>
          <span class="text-muted-foreground">{Math.round(progress)}%</span>
        </div>
        <Progress value={progress} class="h-2" />
      </div>
      <p class="text-sm text-muted-foreground">{message}</p>
    </CardContent>
  </Card>
{/if}
