<script lang="ts">
  import { onMount } from 'svelte'
  import { Button } from '$components/ui/button'
  import { Input } from '$components/ui/input'
  import { Label } from '$components/ui/label'
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle
  } from '$components/ui/card'
  import { Badge } from '$components/ui/badge'
  import { Separator } from '$components/ui/separator'
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
    DialogFooter,
    DialogClose
  } from '$components/ui/dialog'
  import { toast } from 'svelte-sonner'
  import {
    History,
    Search,
    Trash2,
    Download,
    Upload,
    ChevronRight,
    FileText,
    Globe,
    Clock,
    CheckCircle2,
    RefreshCw,
    Edit3,
    AlertTriangle
  } from '@lucide/svelte'
  import { push } from 'svelte-spa-router'
  import { invoke } from '@tauri-apps/api/core'
  import { commands, type HistoryListItem } from '$lib/ipc'

  // ============================================================================
  // State
  // ============================================================================

  let historyItems = $state<HistoryListItem[]>([])
  let isLoading = $state(true)
  let searchQuery = $state('')
  let isRenameDialogOpen = $state(false)
  let isDeleteDialogOpen = $state(false)
  let isDeleteAllDialogOpen = $state(false)
  let isExportDialogOpen = $state(false)
  let isRunExportDialogOpen = $state(false)
  let isImportDialogOpen = $state(false)
  let itemToAction = $state<HistoryListItem | null>(null)
  let newName = $state('')
  let exportPath = $state('')
  let runExportPath = $state('')
  let importPath = $state('')

  // ============================================================================
  // Lifecycle
  // ============================================================================

  onMount(() => {
    loadHistory()
  })

  // ============================================================================
  // Computed
  // ============================================================================

  let filteredItems = $derived(
    historyItems.filter(item => {
      const query = searchQuery.toLowerCase()
      return (
        item.url.toLowerCase().includes(query) ||
        (item.name?.toLowerCase().includes(query) ?? false)
      )
    })
  )

  // ============================================================================
  // Handlers
  // ============================================================================

  async function loadHistory() {
    isLoading = true
    try {
      const result = await commands.listHistory(100, 0)
      if (result.status === 'error') {
        toast.error('Failed to load history')
        console.error(result.error)
      } else {
        historyItems = result.data
      }
    } catch (err) {
      toast.error('Failed to load history')
      console.error(err)
    } finally {
      isLoading = false
    }
  }

  function openRenameDialog(item: HistoryListItem) {
    itemToAction = item
    newName = item.name || ''
    isRenameDialogOpen = true
  }

  async function confirmRename() {
    if (!itemToAction) return

    try {
      const result = await commands.updateHistoryItem(itemToAction.id, {
        name: newName.trim() || null
      })
      if (result.status === 'error') {
        toast.error('Failed to rename item')
        console.error(result.error)
      } else {
        toast.success('Item renamed')
        await loadHistory()
        isRenameDialogOpen = false
      }
    } catch (err) {
      toast.error('Failed to rename item')
      console.error(err)
    }
  }

  function openDeleteDialog(item: HistoryListItem) {
    itemToAction = item
    isDeleteDialogOpen = true
  }

  async function confirmDelete() {
    if (!itemToAction) return

    try {
      const result = await commands.deleteHistoryItem(itemToAction.id)
      if (result.status === 'error') {
        toast.error('Failed to delete item')
        console.error(result.error)
      } else {
        toast.success('Item deleted')
        await loadHistory()
        isDeleteDialogOpen = false
      }
    } catch (err) {
      toast.error('Failed to delete item')
      console.error(err)
    }
  }

  async function confirmDeleteAll() {
    try {
      const result = await commands.deleteAllHistory()
      if (result.status === 'error') {
        toast.error('Failed to delete all items')
        console.error(result.error)
      } else {
        toast.success(`Deleted ${result.data} items`)
        await loadHistory()
        isDeleteAllDialogOpen = false
      }
    } catch (err) {
      toast.error('Failed to delete all items')
      console.error(err)
    }
  }

  async function confirmExport() {
    if (!exportPath.trim()) {
      toast.error('Please enter a file path')
      return
    }

    try {
      const result = await commands.exportHistory(exportPath.trim())
      if (result.status === 'error') {
        toast.error(`Export failed: ${result.error}`)
      } else {
        toast.success('History exported successfully')
        isExportDialogOpen = false
        exportPath = ''
      }
    } catch (err) {
      toast.error(
        `Export failed: ${err instanceof Error ? err.message : String(err)}`
      )
    }
  }

  function openRunExportDialog(item: HistoryListItem) {
    itemToAction = item
    runExportPath = ''
    isRunExportDialogOpen = true
  }

  async function confirmRunExport() {
    if (!itemToAction) return
    if (!runExportPath.trim()) {
      toast.error('Please enter a file path')
      return
    }

    try {
      await invoke('export_history_item', {
        id: itemToAction.id,
        path: runExportPath.trim()
      })
      toast.success('Analysis run exported successfully')
      isRunExportDialogOpen = false
      runExportPath = ''
      itemToAction = null
    } catch (err) {
      toast.error(
        `Export failed: ${err instanceof Error ? err.message : String(err)}`
      )
    }
  }

  async function confirmImport() {
    if (!importPath.trim()) {
      toast.error('Please enter a file path')
      return
    }

    try {
      const result = await commands.importHistory(importPath.trim())
      if (result.status === 'error') {
        toast.error(`Import failed: ${result.error}`)
      } else {
        toast.success(`Imported ${result.data} items`)
        await loadHistory()
        isImportDialogOpen = false
        importPath = ''
      }
    } catch (err) {
      toast.error(
        `Import failed: ${err instanceof Error ? err.message : String(err)}`
      )
    }
  }

  function formatDate(isoString: string): string {
    return new Date(isoString).toLocaleString()
  }

  function formatDuration(ms: number): string {
    if (ms < 1000) return `${ms}ms`
    return `${(ms / 1000).toFixed(1)}s`
  }

  function getScoreColor(score: number | null): string {
    if (score === null) return 'bg-muted text-muted-foreground'
    if (score >= 80) return 'bg-green-500 text-white'
    if (score >= 50) return 'bg-amber-500 text-white'
    return 'bg-red-500 text-white'
  }
</script>

<div class="h-full flex flex-col p-6 gap-6 overflow-auto">
  <!-- Header -->
  <div
    class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between"
  >
    <div class="flex items-center gap-4">
      <div
        class="flex h-12 w-12 items-center justify-center rounded-xl bg-gradient-to-br from-amber-500 to-orange-600 shadow-lg"
      >
        <History class="h-6 w-6 text-white" />
      </div>
      <div>
        <h1 class="text-3xl font-bold tracking-tight">History</h1>
        <p class="text-muted-foreground">View and manage past analyses</p>
      </div>
    </div>

    <div class="flex gap-2">
      <Button
        variant="outline"
        size="sm"
        onclick={() => (isImportDialogOpen = true)}
      >
        <Upload class="mr-2 h-4 w-4" />
        Import
      </Button>
      <Button
        variant="outline"
        size="sm"
        onclick={() => (isExportDialogOpen = true)}
      >
        <Download class="mr-2 h-4 w-4" />
        Export
      </Button>
      <Button variant="outline" size="sm" onclick={loadHistory}>
        <RefreshCw class="mr-2 h-4 w-4" />
        Refresh
      </Button>
    </div>
  </div>

  <Separator />

  <!-- Search and Actions -->
  <div
    class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between"
  >
    <div class="relative max-w-sm">
      <Search class="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
      <Input
        type="search"
        placeholder="Search by URL or name..."
        class="pl-8"
        bind:value={searchQuery}
      />
    </div>

    {#if historyItems.length > 0}
      <Button
        variant="destructive"
        size="sm"
        onclick={() => (isDeleteAllDialogOpen = true)}
      >
        <Trash2 class="mr-2 h-4 w-4" />
        Delete All
      </Button>
    {/if}
  </div>

  <!-- History List -->
  {#if isLoading}
    <div class="flex flex-1 items-center justify-center">
      <div class="flex items-center gap-2 text-muted-foreground">
        <RefreshCw class="h-5 w-5 animate-spin" />
        <span>Loading history...</span>
      </div>
    </div>
  {:else if filteredItems.length === 0}
    <Card class="border-dashed">
      <CardContent
        class="flex flex-col items-center justify-center py-16 text-center"
      >
        <div
          class="flex h-16 w-16 items-center justify-center rounded-full bg-muted"
        >
          <History class="h-8 w-8 text-muted-foreground" />
        </div>
        <h3 class="mt-4 text-lg font-semibold">
          {searchQuery ? 'No matching results' : 'No history yet'}
        </h3>
        <p class="mt-2 max-w-xs text-sm text-muted-foreground">
          {searchQuery
            ? 'Try a different search term'
            : 'Run an analysis to see results here'}
        </p>
      </CardContent>
    </Card>
  {:else}
    <div class="grid gap-4">
      {#each filteredItems as item (item.id)}
        <Card class="group transition-shadow hover:shadow-md">
          <CardContent class="p-4">
            <div
              class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between"
            >
              <!-- Left: Info -->
              <div class="flex items-start gap-4">
                <!-- Score Badge -->
                <div
                  class="flex h-12 w-12 shrink-0 items-center justify-center rounded-lg {getScoreColor(
                    item.summary.seo_score
                  )}"
                >
                  <span class="text-lg font-bold">
                    {item.summary.seo_score
                      ? Math.round(item.summary.seo_score)
                      : '-'}
                  </span>
                </div>

                <!-- Details -->
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

                  <p class="mt-1 truncate text-sm text-muted-foreground">
                    {item.url}
                  </p>

                  <div class="mt-2 flex flex-wrap items-center gap-2">
                    <Badge variant="outline" class="text-xs">
                      {item.analysis_type === 'single' ? 'Single' : 'Crawl'}
                    </Badge>

                    <span
                      class="flex items-center gap-1 text-xs text-muted-foreground"
                    >
                      <Clock class="h-3 w-3" />
                      {formatDate(item.created_at)}
                    </span>

                    <span
                      class="flex items-center gap-1 text-xs text-muted-foreground"
                    >
                      <Clock class="h-3 w-3" />
                      {formatDuration(item.summary.duration_ms)}
                    </span>

                    {#if item.summary.total_issues > 0}
                      <Badge variant="destructive" class="text-xs">
                        {item.summary.total_issues} issues
                      </Badge>
                    {:else}
                      <Badge variant="default" class="bg-green-500 text-xs">
                        <CheckCircle2 class="mr-1 h-3 w-3" />
                        Clean
                      </Badge>
                    {/if}
                  </div>
                </div>
              </div>

              <!-- Right: Actions -->
              <div class="flex items-center gap-2">
                <Button
                  variant="ghost"
                  size="sm"
                  onclick={() => push(`/run/${item.id}`)}
                >
                  <ChevronRight class="mr-2 h-4 w-4" />
                  View
                </Button>

                <Button
                  variant="ghost"
                  size="sm"
                  onclick={() => openRenameDialog(item)}
                >
                  <Edit3 class="h-4 w-4" />
                </Button>

                <Button
                  variant="ghost"
                  size="sm"
                  onclick={() => openRunExportDialog(item)}
                >
                  <Download class="h-4 w-4" />
                </Button>

                <Button
                  variant="ghost"
                  size="sm"
                  class="text-red-500 hover:text-red-600"
                  onclick={() => openDeleteDialog(item)}
                >
                  <Trash2 class="h-4 w-4" />
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>
      {/each}
    </div>
  {/if}
</div>

<!-- Rename Dialog -->
<Dialog bind:open={isRenameDialogOpen}>
  <DialogContent>
    <DialogHeader>
      <DialogTitle>Rename Analysis</DialogTitle>
      <DialogDescription>Enter a new name for this analysis</DialogDescription>
    </DialogHeader>

    <div class="space-y-4 py-4">
      <div class="space-y-2">
        <Label for="new-name">Name</Label>
        <Input
          id="new-name"
          placeholder="My Analysis"
          bind:value={newName}
          onkeydown={e => e.key === 'Enter' && confirmRename()}
        />
      </div>
    </div>

    <DialogFooter>
      <DialogClose>
        <Button variant="outline">Cancel</Button>
      </DialogClose>
      <Button onclick={confirmRename}>Save</Button>
    </DialogFooter>
  </DialogContent>
</Dialog>

<!-- Delete Dialog -->
<Dialog bind:open={isDeleteDialogOpen}>
  <DialogContent>
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2 text-red-600">
        <AlertTriangle class="h-5 w-5" />
        Delete Analysis
      </DialogTitle>
      <DialogDescription>
        Are you sure you want to delete "{itemToAction?.name ||
          'this analysis'}"? This action cannot be undone.
      </DialogDescription>
    </DialogHeader>

    <DialogFooter>
      <DialogClose>
        <Button variant="outline">Cancel</Button>
      </DialogClose>
      <Button variant="destructive" onclick={confirmDelete}>
        <Trash2 class="mr-2 h-4 w-4" />
        Delete
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>

<!-- Delete All Dialog -->
<Dialog bind:open={isDeleteAllDialogOpen}>
  <DialogContent>
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2 text-red-600">
        <AlertTriangle class="h-5 w-5" />
        Delete All History
      </DialogTitle>
      <DialogDescription>
        Are you sure you want to delete all {historyItems.length} analyses? This action
        cannot be undone.
      </DialogDescription>
    </DialogHeader>

    <DialogFooter>
      <DialogClose>
        <Button variant="outline">Cancel</Button>
      </DialogClose>
      <Button variant="destructive" onclick={confirmDeleteAll}>
        <Trash2 class="mr-2 h-4 w-4" />
        Delete All
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>

<Dialog bind:open={isRunExportDialogOpen}>
  <DialogContent>
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2">
        <Download class="h-5 w-5" />
        Export Analysis Run
      </DialogTitle>
      <DialogDescription>
        Export "{itemToAction?.name || 'this analysis'}" and its page rows to a
        JSON file
      </DialogDescription>
    </DialogHeader>

    <div class="space-y-4 py-4">
      <div class="space-y-2">
        <Label for="run-export-path">File Path</Label>
        <Input
          id="run-export-path"
          placeholder="/home/user/pagelens-run-export.json"
          bind:value={runExportPath}
        />
        <p class="text-xs text-muted-foreground">
          Enter the full path where this run export file should be saved
        </p>
      </div>
    </div>

    <DialogFooter>
      <DialogClose>
        <Button variant="outline">Cancel</Button>
      </DialogClose>
      <Button onclick={confirmRunExport}>
        <Download class="mr-2 h-4 w-4" />
        Export Run
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>

<!-- Export Dialog -->
<Dialog bind:open={isExportDialogOpen}>
  <DialogContent>
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2">
        <Download class="h-5 w-5" />
        Export History
      </DialogTitle>
      <DialogDescription>Export all history to a JSON file</DialogDescription>
    </DialogHeader>

    <div class="space-y-4 py-4">
      <div class="space-y-2">
        <Label for="export-path">File Path</Label>
        <Input
          id="export-path"
          placeholder="/home/user/pagelens-export.json"
          bind:value={exportPath}
        />
        <p class="text-xs text-muted-foreground">
          Enter the full path where the export file should be saved
        </p>
      </div>
    </div>

    <DialogFooter>
      <DialogClose>
        <Button variant="outline">Cancel</Button>
      </DialogClose>
      <Button onclick={confirmExport}>
        <Download class="mr-2 h-4 w-4" />
        Export
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>

<!-- Import Dialog -->
<Dialog bind:open={isImportDialogOpen}>
  <DialogContent>
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2">
        <Upload class="h-5 w-5" />
        Import History
      </DialogTitle>
      <DialogDescription>Import analyses from a JSON file</DialogDescription>
    </DialogHeader>

    <div class="space-y-4 py-4">
      <div class="space-y-2">
        <Label for="import-path">File Path</Label>
        <Input
          id="import-path"
          placeholder="/home/user/pagelens-export.json"
          bind:value={importPath}
        />
        <p class="text-xs text-muted-foreground">
          Enter the full path to the JSON file to import
        </p>
      </div>
    </div>

    <DialogFooter>
      <DialogClose>
        <Button variant="outline">Cancel</Button>
      </DialogClose>
      <Button onclick={confirmImport}>
        <Upload class="mr-2 h-4 w-4" />
        Import
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
