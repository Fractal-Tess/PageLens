<script lang="ts">
  import { Button } from '../../ui/button'
  import {
    FileText,
    Globe,
    Clock,
    CircleCheck,
    ChevronRight,
    Pencil,
    Download,
    Trash2
  } from '@lucide/svelte'
  import type { HistoryItem } from '../../../stores/history.svelte'

  interface Props {
    item: HistoryItem
    formatDate: (isoString: string) => string
    formatDuration: (ms: number) => string
    getScoreColor: (score: number | null) => string
    onView: (item: HistoryItem) => void
    onRename: (item: HistoryItem) => void
    onExportRun: (item: HistoryItem) => void
    onDelete: (item: HistoryItem) => void
  }

  let {
    item,
    formatDate,
    formatDuration,
    getScoreColor,
    onView,
    onRename,
    onExportRun,
    onDelete
  }: Props = $props()
</script>

<div class="group rounded-lg border bg-card transition-shadow hover:shadow-md">
  <div
    class="flex flex-col gap-4 p-4 sm:flex-row sm:items-center sm:justify-between"
  >
    <div class="flex items-start gap-4">
      <div
        class="flex h-12 w-12 shrink-0 items-center justify-center rounded-lg {getScoreColor(
          item.summary.seo_score
        )}"
      >
        <span class="text-lg font-bold">
          {item.summary.seo_score ? Math.round(item.summary.seo_score) : '-'}
        </span>
      </div>

      <div class="min-w-0 flex-1">
        <div class="flex items-center gap-2">
          {#if item.analysis_type === 'single'}
            <FileText class="h-4 w-4 text-indigo-500" />
          {:else}
            <Globe class="h-4 w-4 text-violet-500" />
          {/if}
          <h4 class="truncate font-semibold">
            {item.name || 'Unnamed analysis'}
          </h4>
        </div>

        <p class="mt-1 truncate text-sm text-muted-foreground">{item.url}</p>

        <div class="mt-2 flex flex-wrap items-center gap-2">
          <span class="rounded border px-2 py-0.5 text-xs"
            >{item.analysis_type === 'single' ? 'Single' : 'Crawl'}</span
          >

          <span class="flex items-center gap-1 text-xs text-muted-foreground">
            <Clock class="h-3 w-3" />
            {formatDate(item.created_at)}
          </span>

          <span class="flex items-center gap-1 text-xs text-muted-foreground">
            <Clock class="h-3 w-3" />
            {formatDuration(item.summary.duration_ms)}
          </span>

          {#if item.summary.total_issues > 0}
            <span
              class="rounded bg-destructive px-2 py-0.5 text-xs text-destructive-foreground"
            >
              {item.summary.total_issues} issues
            </span>
          {:else}
            <span class="rounded bg-green-500 px-2 py-0.5 text-xs text-white">
              <CircleCheck class="mr-1 inline h-3 w-3" />
              Clean
            </span>
          {/if}
        </div>
      </div>
    </div>

    <div class="flex items-center gap-2">
      <Button variant="ghost" size="sm" onclick={() => onView(item)}>
        <ChevronRight class="mr-2 h-4 w-4" />
        View
      </Button>

      <Button variant="ghost" size="sm" onclick={() => onRename(item)}>
        <Pencil class="h-4 w-4" />
      </Button>

      <Button variant="ghost" size="sm" onclick={() => onExportRun(item)}>
        <Download class="h-4 w-4" />
      </Button>

      <Button
        variant="ghost"
        size="sm"
        class="text-red-500 hover:text-red-600"
        onclick={() => onDelete(item)}
      >
        <Trash2 class="h-4 w-4" />
      </Button>
    </div>
  </div>
</div>
