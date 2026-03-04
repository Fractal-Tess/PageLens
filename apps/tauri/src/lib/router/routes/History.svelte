<script lang="ts">
  import { onMount } from 'svelte';
  import { push } from 'svelte-spa-router';
  import { toast } from 'svelte-sonner';
  import { invoke } from '@tauri-apps/api/core';
  import {
    HistoryToolbar,
    HistoryList,
    HistoryDialogs,
    historyStore,
    historyDialogStore,
    filteredHistory,
    type HistoryItem,
  } from '@pagelens/ui';
  import { commands } from '$lib/ipc';

  onMount(() => {
    loadHistory();
  });

  async function loadHistory() {
    historyStore.setLoading(true);
    try {
      const result = await commands.listHistory(100, 0);
      if (result.status === 'error') {
        toast.error('Failed to load history');
        console.error(result.error);
      } else {
        historyStore.setItems(result.data);
      }
    } catch (err) {
      toast.error('Failed to load history');
      console.error(err);
    } finally {
      historyStore.setLoading(false);
    }
  }

  async function confirmRename() {
    const { item, newName } = $historyDialogStore.rename;
    if (!item) return;

    try {
      const result = await commands.updateHistoryItem(item.id, {
        name: newName.trim() || null,
      });
      if (result.status === 'error') {
        toast.error('Failed to rename item');
      } else {
        toast.success('Item renamed');
        await loadHistory();
        historyDialogStore.closeRename();
      }
    } catch (err) {
      toast.error('Failed to rename item');
      console.error(err);
    }
  }

  async function confirmDelete() {
    const { item } = $historyDialogStore.delete;
    if (!item) return;

    try {
      const result = await commands.deleteHistoryItem(item.id);
      if (result.status === 'error') {
        toast.error('Failed to delete item');
      } else {
        toast.success('Item deleted');
        await loadHistory();
        historyDialogStore.closeDelete();
      }
    } catch (err) {
      toast.error('Failed to delete item');
      console.error(err);
    }
  }

  async function confirmDeleteAll() {
    try {
      const result = await commands.deleteAllHistory();
      if (result.status === 'error') {
        toast.error('Failed to delete all items');
      } else {
        toast.success(`Deleted ${result.data} items`);
        await loadHistory();
        historyDialogStore.closeDeleteAll();
      }
    } catch (err) {
      toast.error('Failed to delete all items');
      console.error(err);
    }
  }

  async function confirmExport() {
    const { path } = $historyDialogStore.export;
    if (!path.trim()) {
      toast.error('Please enter a file path');
      return;
    }

    try {
      const result = await commands.exportHistory(path.trim());
      if (result.status === 'error') {
        toast.error(`Export failed: ${result.error}`);
      } else {
        toast.success('History exported successfully');
        historyDialogStore.closeExport();
      }
    } catch (err) {
      toast.error(`Export failed: ${err instanceof Error ? err.message : String(err)}`);
    }
  }

  async function confirmRunExport() {
    const { item, path } = $historyDialogStore.exportRun;
    if (!item) return;
    if (!path.trim()) {
      toast.error('Please enter a file path');
      return;
    }

    try {
      await invoke('export_history_item', {
        id: item.id,
        path: path.trim(),
      });
      toast.success('Analysis run exported successfully');
      historyDialogStore.closeExportRun();
    } catch (err) {
      toast.error(`Export failed: ${err instanceof Error ? err.message : String(err)}`);
    }
  }

  async function confirmImport() {
    const { path } = $historyDialogStore.import;
    if (!path.trim()) {
      toast.error('Please enter a file path');
      return;
    }

    try {
      const result = await commands.importHistory(path.trim());
      if (result.status === 'error') {
        toast.error(`Import failed: ${result.error}`);
      } else {
        toast.success(`Imported ${result.data} items`);
        await loadHistory();
        historyDialogStore.closeImport();
      }
    } catch (err) {
      toast.error(`Import failed: ${err instanceof Error ? err.message : String(err)}`);
    }
  }

  function formatDate(isoString: string): string {
    return new Date(isoString).toLocaleString();
  }

  function formatDuration(ms: number): string {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(1)}s`;
  }

  function getScoreColor(score: number | null): string {
    if (score === null) return 'bg-muted text-muted-foreground';
    if (score >= 80) return 'bg-green-500 text-white';
    if (score >= 50) return 'bg-amber-500 text-white';
    return 'bg-red-500 text-white';
  }

  function viewRun(item: HistoryItem) {
    push(`/run/${item.id}`);
  }
</script>

<div class="h-full flex flex-col p-6 gap-6 overflow-auto">
  <HistoryToolbar
    searchQuery={$historyStore.searchQuery}
    itemCount={$historyStore.items.length}
    onSearchChange={value => historyStore.setSearchQuery(value)}
    onImport={() => historyDialogStore.openImport()}
    onExport={() => historyDialogStore.openExport()}
    onRefresh={loadHistory}
    onDeleteAll={() => historyDialogStore.openDeleteAll()}
  />

  <HistoryList
    items={$filteredHistory}
    isLoading={$historyStore.isLoading}
    searchQuery={$historyStore.searchQuery}
    {formatDate}
    {formatDuration}
    {getScoreColor}
    onView={viewRun}
    onRename={item => historyDialogStore.openRename(item)}
    onExportRun={item => historyDialogStore.openExportRun(item)}
    onDelete={item => historyDialogStore.openDelete(item)}
  />

  <HistoryDialogs
    historyState={$historyStore}
    dialogState={$historyDialogStore}
    onCloseRename={() => historyDialogStore.closeRename()}
    onCloseDelete={() => historyDialogStore.closeDelete()}
    onCloseDeleteAll={() => historyDialogStore.closeDeleteAll()}
    onCloseExportRun={() => historyDialogStore.closeExportRun()}
    onCloseExport={() => historyDialogStore.closeExport()}
    onCloseImport={() => historyDialogStore.closeImport()}
    onRenameInput={value => historyDialogStore.setRenameName(value)}
    onExportRunInput={value => historyDialogStore.setExportRunPath(value)}
    onExportInput={value => historyDialogStore.setExportPath(value)}
    onImportInput={value => historyDialogStore.setImportPath(value)}
    onConfirmRename={confirmRename}
    onConfirmDelete={confirmDelete}
    onConfirmDeleteAll={confirmDeleteAll}
    onConfirmExportRun={confirmRunExport}
    onConfirmExport={confirmExport}
    onConfirmImport={confirmImport}
  />
</div>
