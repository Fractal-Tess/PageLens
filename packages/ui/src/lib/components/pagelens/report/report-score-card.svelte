<script lang="ts">
  import { Card, CardContent } from '../../ui/card';
  import { Gauge, AlertCircle, AlertTriangle, Clock } from '@lucide/svelte';
  import { getScoreColor, formatDuration } from '../../../stores/report.svelte';
  import type { Issue } from './types';
  
  interface Props {
    score: number;
    issues: Issue[];
    durationMs: number;
    class?: string;
  }
  
  let { score, issues, durationMs, class: className = '' }: Props = $props();
  
  const errorCount = $derived(issues.filter(i => i.severity === 'Error').length);
  const warningCount = $derived(issues.filter(i => i.severity === 'Warning').length);
</script>

<div class="grid flex-1 grid-cols-2 gap-3 sm:grid-cols-4 {className}">
  <Card>
    <CardContent class="pt-4 text-center">
      <Gauge class="mx-auto mb-1 h-5 w-5 text-indigo-500" />
      <p class="text-2xl font-bold">{Math.round(score)}</p>
      <p class="text-xs text-muted-foreground">Score</p>
    </CardContent>
  </Card>
  
  <Card>
    <CardContent class="pt-4 text-center">
      <AlertCircle class="mx-auto mb-1 h-5 w-5 text-red-500" />
      <p class="text-2xl font-bold">{errorCount}</p>
      <p class="text-xs text-muted-foreground">Errors</p>
    </CardContent>
  </Card>
  
  <Card>
    <CardContent class="pt-4 text-center">
      <AlertTriangle class="mx-auto mb-1 h-5 w-5 text-amber-500" />
      <p class="text-2xl font-bold">{warningCount}</p>
      <p class="text-xs text-muted-foreground">Warnings</p>
    </CardContent>
  </Card>
  
  <Card>
    <CardContent class="pt-4 text-center">
      <Clock class="mx-auto mb-1 h-5 w-5 text-muted-foreground" />
      <p class="text-2xl font-bold">{formatDuration(durationMs)}</p>
      <p class="text-xs text-muted-foreground">Duration</p>
    </CardContent>
  </Card>
</div>
