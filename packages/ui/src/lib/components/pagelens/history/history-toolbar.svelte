<script lang="ts">
  import { Button } from '../../ui/button'
  import { Input } from '../../ui/input'
  import { Separator } from '../../ui/separator'
  import PageHeader from '../layout/page-header.svelte'
  import {
    History,
    Search,
    Trash2,
    Download,
    Upload,
    RefreshCw
  } from '@lucide/svelte'

  interface Props {
    searchQuery: string
    itemCount: number
    onSearchChange: (value: string) => void
    onImport: () => void
    onExport: () => void
    onRefresh: () => void
    onDeleteAll: () => void
  }

  let {
    searchQuery,
    itemCount,
    onSearchChange,
    onImport,
    onExport,
    onRefresh,
    onDeleteAll
  }: Props = $props()
</script>

<PageHeader
  title="History"
  description="View and manage past analyses"
  icon={History}
>
  {#snippet actions()}
    <Button variant="outline" size="sm" onclick={onImport}>
      <Upload class="mr-2 h-4 w-4" />
      Import
    </Button>
    <Button variant="outline" size="sm" onclick={onExport}>
      <Download class="mr-2 h-4 w-4" />
      Export
    </Button>
    <Button variant="outline" size="sm" onclick={onRefresh}>
      <RefreshCw class="mr-2 h-4 w-4" />
      Refresh
    </Button>
  {/snippet}
</PageHeader>

<Separator />

<div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
  <div class="relative max-w-sm">
    <Search class="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
    <Input
      type="search"
      placeholder="Search by URL or name..."
      class="pl-8"
      value={searchQuery}
      oninput={e => onSearchChange(e.currentTarget.value)}
    />
  </div>

  {#if itemCount > 0}
    <Button variant="destructive" size="sm" onclick={onDeleteAll}>
      <Trash2 class="mr-2 h-4 w-4" />
      Delete All
    </Button>
  {/if}
</div>
