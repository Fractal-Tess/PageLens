<script lang="ts">
  import { History } from '@lucide/svelte'
  import EmptyState from '../layout/empty-state.svelte'
  import LoadingState from '../layout/loading-state.svelte'
  import HistoryItemCard from './history-item-card.svelte'
  import type { HistoryItem } from '../../../stores/history.svelte'

  interface Props {
    items: HistoryItem[]
    isLoading: boolean
    searchQuery: string
    formatDate: (isoString: string) => string
    formatDuration: (ms: number) => string
    getScoreColor: (score: number | null) => string
    onView: (item: HistoryItem) => void
    onRename: (item: HistoryItem) => void
    onExportRun: (item: HistoryItem) => void
    onDelete: (item: HistoryItem) => void
  }

  let {
    items,
    isLoading,
    searchQuery,
    formatDate,
    formatDuration,
    getScoreColor,
    onView,
    onRename,
    onExportRun,
    onDelete
  }: Props = $props()
</script>

{#if isLoading}
  <LoadingState message="Loading history..." />
{:else if items.length === 0}
  <EmptyState
    title={searchQuery ? 'No matching results' : 'No history yet'}
    description={searchQuery
      ? 'Try a different search term'
      : 'Run an analysis to see results here'}
    icon={History}
  />
{:else}
  <div class="grid gap-4">
    {#each items as item (item.id)}
      <HistoryItemCard
        {item}
        {formatDate}
        {formatDuration}
        {getScoreColor}
        {onView}
        {onRename}
        {onExportRun}
        {onDelete}
      />
    {/each}
  </div>
{/if}
