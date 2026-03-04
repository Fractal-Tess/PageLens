<script lang="ts">
  import { onMount } from 'svelte';
  import { push } from 'svelte-spa-router';
  import { toast } from 'svelte-sonner';
  import { Button } from '@pagelens/ui/shadcn/button';
  import { Input } from '@pagelens/ui/shadcn/input';
  import { Separator } from '@pagelens/ui/shadcn/separator';
  import { ChevronRight, Search, History, Gauge, AlertCircle, Scan } from '@lucide/svelte';
  import {
    PageHeader,
    StatCard,
    EmptyState,
    LoadingState,
    historyStore,
    historyStats,
    analysisStore,
  } from '@pagelens/ui';
  import { commands, type HistoryListItem } from '$lib/ipc';

  let recentItems = $state<HistoryListItem[]>([]);
  let isLoading = $state(true);

  onMount(async () => {
    try {
      const [recentResult, allResult] = await Promise.all([
        commands.listHistory(5, 0),
        commands.listHistory(null, null),
      ]);

      if (recentResult.status === 'ok') {
        recentItems = recentResult.data;
        historyStore.setItems(recentResult.data);
      }

      if (allResult.status === 'ok') {
        historyStore.setItems(allResult.data);
      }
    } catch {
      // silently fail on dashboard
    } finally {
      isLoading = false;
    }
  });

  function handleQuickAnalyze() {
    const { url } = $analysisStore;
    if (!url.trim()) {
      toast.error('Please enter a URL');
      return;
    }
    push(`/#simple?url=${encodeURIComponent(url.trim())}`);
  }

  function formatDate(iso: string): string {
    return new Date(iso).toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  function getScoreColor(score: number | null): string {
    if (score == null) return 'bg-muted text-muted-foreground';
    if (score >= 80) return 'bg-green-500 text-white';
    if (score >= 50) return 'bg-amber-500 text-white';
    return 'bg-red-500 text-white';
  }

  const stats = $derived([
    { icon: History, label: 'Total Analyses', value: $historyStats.total },
    { icon: Gauge, label: 'Avg Score (recent)', value: $historyStats.avgScore ? Math.round($historyStats.avgScore) : '—' },
    { icon: AlertCircle, label: 'Recent Issues', value: $historyStats.totalIssues },
  ]);
</script>

<div class="flex flex-col gap-6 p-6 overflow-auto h-full">
  <PageHeader title="PageLens" description="Web auditing dashboard" icon={Search} />

  <!-- Quick Analyze -->
  <form
    class="flex gap-3"
    onsubmit={(e) => {
      e.preventDefault();
      handleQuickAnalyze();
    }}
  >
    <Input
      type="url"
      placeholder="https://example.com"
      class="flex-1"
      bind:value={$analysisStore.url}
    />
    <Button type="submit" disabled={!$analysisStore.url.trim()}>
      <Scan class="mr-2 h-4 w-4" />
      Quick Analyze
    </Button>
  </form>

  <!-- Stats Row -->
  <div class="grid gap-4 md:grid-cols-3">
    {#each stats as stat}
      <StatCard icon={stat.icon} value={stat.value} label={stat.label} />
    {/each}
  </div>

  <Separator />

  <!-- Recent History -->
  <div class="flex items-center justify-between">
    <h2 class="text-lg font-semibold">Recent Analyses</h2>
    <Button variant="ghost" size="sm" onclick={() => push('/#history')}>
      View all
      <ChevronRight class="ml-1 h-4 w-4" />
    </Button>
  </div>

  {#if isLoading}
    <LoadingState />
  {:else if recentItems.length === 0}
    <EmptyState
      title="No analyses yet"
      description="Run your first analysis to see results here"
      icon={Scan}
    />
  {:else}
    <div class="grid gap-3">
      {#each recentItems as item (item.id)}
        {@const Icon = item.analysis_type === 'single' ? Scan : Search}
        <button
          class="text-left w-full"
          onclick={() => push(`/run/${item.id}`)}
        >
          <div class="flex items-center gap-4 p-4 rounded-lg border bg-card hover:shadow-md transition-shadow cursor-pointer">
            <!-- Score Badge -->
            <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg {getScoreColor(item.summary.seo_score)}">
              <span class="text-sm font-bold">
                {item.summary.seo_score ? Math.round(item.summary.seo_score) : '—'}
              </span>
            </div>

            <!-- Info -->
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2">
                <Icon class="h-3.5 w-3.5 text-indigo-500 shrink-0" />
                <span class="truncate text-sm font-medium">{item.name || item.url}</span>
              </div>
              <div class="flex items-center gap-3 mt-0.5 text-xs text-muted-foreground">
                <span class="flex items-center gap-1">
                  <History class="h-3 w-3" />
                  {formatDate(item.created_at)}
                </span>
                {#if item.summary.total_issues > 0}
                  <span class="text-amber-500">{item.summary.total_issues} issues</span>
                {/if}
              </div>
            </div>

            <ChevronRight class="h-4 w-4 text-muted-foreground shrink-0" />
          </div>
        </button>
      {/each}
    </div>
  {/if}
</div>
