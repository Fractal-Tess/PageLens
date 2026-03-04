<script lang="ts">
  import { onMount } from 'svelte'
  import { push } from 'svelte-spa-router'
  import { Button } from '@pagelens/ui/shadcn/button'
  import { Input } from '@pagelens/ui/shadcn/input'
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle
  } from '@pagelens/ui/shadcn/card'
  import { Badge } from '@pagelens/ui/shadcn/badge'
  import { Separator } from '@pagelens/ui/shadcn/separator'
  import { toast } from 'svelte-sonner'
  import {
    Scan,
    History,
    ChevronRight,
    Clock,
    FileText,
    Globe,
    Gauge,
    AlertCircle,
    Search
  } from '@lucide/svelte'
  import { commands, type HistoryListItem } from '$lib/ipc'

  let url = $state('')
  let recentItems = $state<HistoryListItem[]>([])
  let totalCount = $state(0)
  let isLoading = $state(true)

  onMount(async () => {
    try {
      const result = await commands.listHistory(5, 0)
      if (result.status === 'ok') {
        recentItems = result.data
      }
      const allResult = await commands.listHistory(null, null)
      if (allResult.status === 'ok') {
        totalCount = allResult.data.length
      }
    } catch {
      // silently fail on dashboard
    } finally {
      isLoading = false
    }
  })

  function handleQuickAnalyze() {
    if (!url.trim()) {
      toast.error('Please enter a URL')
      return
    }
    push(`/#simple?url=${encodeURIComponent(url.trim())}`)
  }

  function formatDate(iso: string): string {
    return new Date(iso).toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  }

  function scoreColor(score: number | null): string {
    if (score == null) return 'bg-muted text-muted-foreground'
    if (score >= 80) return 'bg-green-500 text-white'
    if (score >= 50) return 'bg-amber-500 text-white'
    return 'bg-red-500 text-white'
  }

  const avgScore = $derived(() => {
    const scored = recentItems.filter(i => i.summary.seo_score != null)
    if (scored.length === 0) return null
    return (
      scored.reduce((sum, i) => sum + (i.summary.seo_score ?? 0), 0) /
      scored.length
    )
  })
</script>

<div class="flex flex-col gap-6 p-6 overflow-auto h-full">
  <!-- Welcome -->
  <div class="flex items-center gap-4">
    <div
      class="flex h-12 w-12 items-center justify-center rounded-xl bg-gradient-to-br from-indigo-500 to-violet-600 shadow-lg"
    >
      <Search class="h-6 w-6 text-white" />
    </div>
    <div>
      <h1 class="text-3xl font-bold tracking-tight">PageLens</h1>
      <p class="text-muted-foreground">Web auditing dashboard</p>
    </div>
  </div>

  <!-- Quick Analyze -->
  <Card>
    <CardContent class="pt-6">
      <form
        class="flex gap-3"
        onsubmit={e => {
          e.preventDefault()
          handleQuickAnalyze()
        }}
      >
        <Input
          type="url"
          placeholder="https://example.com"
          class="flex-1"
          bind:value={url}
        />
        <Button type="submit" disabled={!url.trim()}>
          <Scan class="mr-2 h-4 w-4" />
          Quick Analyze
        </Button>
      </form>
    </CardContent>
  </Card>

  <!-- Stats Row -->
  <div class="grid gap-4 md:grid-cols-3">
    <Card>
      <CardContent class="flex items-center gap-4 pt-6">
        <div
          class="flex h-10 w-10 items-center justify-center rounded-lg bg-indigo-500/10"
        >
          <History class="h-5 w-5 text-indigo-500" />
        </div>
        <div>
          <p class="text-2xl font-bold">{totalCount}</p>
          <p class="text-xs text-muted-foreground">Total Analyses</p>
        </div>
      </CardContent>
    </Card>
    <Card>
      <CardContent class="flex items-center gap-4 pt-6">
        <div
          class="flex h-10 w-10 items-center justify-center rounded-lg bg-green-500/10"
        >
          <Gauge class="h-5 w-5 text-green-500" />
        </div>
        <div>
          <p class="text-2xl font-bold">
            {#if avgScore() != null}
              {Math.round(avgScore()!)}
            {:else}
              —
            {/if}
          </p>
          <p class="text-xs text-muted-foreground">Avg Score (recent)</p>
        </div>
      </CardContent>
    </Card>
    <Card>
      <CardContent class="flex items-center gap-4 pt-6">
        <div
          class="flex h-10 w-10 items-center justify-center rounded-lg bg-amber-500/10"
        >
          <AlertCircle class="h-5 w-5 text-amber-500" />
        </div>
        <div>
          <p class="text-2xl font-bold">
            {recentItems.reduce((s, i) => s + i.summary.total_issues, 0)}
          </p>
          <p class="text-xs text-muted-foreground">Recent Issues</p>
        </div>
      </CardContent>
    </Card>
  </div>

  <Separator />

  <!-- Recent History -->
  <div class="flex items-center justify-between">
    <h2 class="text-lg font-semibold">Recent Analyses</h2>
    <Button variant="ghost" size="sm" href="/#history">
      View all
      <ChevronRight class="ml-1 h-4 w-4" />
    </Button>
  </div>

  {#if isLoading}
    <div class="text-center text-muted-foreground py-8">Loading...</div>
  {:else if recentItems.length === 0}
    <Card class="border-dashed">
      <CardContent class="flex flex-col items-center py-12 text-center">
        <Scan class="h-12 w-12 text-muted-foreground mb-4" />
        <h3 class="text-lg font-semibold">No analyses yet</h3>
        <p class="text-sm text-muted-foreground mt-1">
          Run your first analysis to see results here
        </p>
      </CardContent>
    </Card>
  {:else}
    <div class="grid gap-3">
      {#each recentItems as item (item.id)}
        <a
          href="/#/run/{item.id}"
          class="block"
          onclick={e => {
            e.preventDefault()
            push(`/run/${item.id}`)
          }}
        >
          <Card class="transition-shadow hover:shadow-md cursor-pointer">
            <CardContent class="flex items-center gap-4 py-3 px-4">
              <!-- Score -->
              <div
                class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg {scoreColor(
                  item.summary.seo_score
                )}"
              >
                <span class="text-sm font-bold">
                  {item.summary.seo_score
                    ? Math.round(item.summary.seo_score)
                    : '—'}
                </span>
              </div>

              <!-- Info -->
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2">
                  {#if item.analysis_type === 'single'}
                    <FileText class="h-3.5 w-3.5 text-indigo-500 shrink-0" />
                  {:else}
                    <Globe class="h-3.5 w-3.5 text-violet-500 shrink-0" />
                  {/if}
                  <span class="truncate text-sm font-medium">
                    {item.name || item.url}
                  </span>
                </div>
                <div
                  class="flex items-center gap-3 mt-0.5 text-xs text-muted-foreground"
                >
                  <span class="flex items-center gap-1">
                    <Clock class="h-3 w-3" />
                    {formatDate(item.created_at)}
                  </span>
                  {#if item.summary.total_issues > 0}
                    <span class="text-amber-500">
                      {item.summary.total_issues} issues
                    </span>
                  {/if}
                </div>
              </div>

              <ChevronRight class="h-4 w-4 text-muted-foreground shrink-0" />
            </CardContent>
          </Card>
        </a>
      {/each}
    </div>
  {/if}
</div>
