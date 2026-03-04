import { writable, derived } from 'svelte/store';

// ============================================================================
// Types
// ============================================================================

export interface HistoryItemSummary {
  seo_score: number | null;
  total_issues: number;
  error_count: number;
  warning_count: number;
  page_count: number;
  duration_ms: number;
}

export interface HistoryItem {
  id: string;
  url: string;
  name: string | null;
  analysis_type: 'single' | 'crawl';
  status: 'pending' | 'running' | 'completed' | 'failed';
  created_at: string;
  summary: HistoryItemSummary;
}

export interface HistoryState {
  items: HistoryItem[];
  isLoading: boolean;
  error: string | null;
  searchQuery: string;
}

export interface HistoryDialogState {
  rename: { open: boolean; item: HistoryItem | null; newName: string };
  delete: { open: boolean; item: HistoryItem | null };
  deleteAll: { open: boolean };
  export: { open: boolean; path: string };
  import: { open: boolean; path: string };
  exportRun: { open: boolean; item: HistoryItem | null; path: string };
}

// ============================================================================
// Store Factory
// ============================================================================

export function createHistoryStore() {
  const { subscribe, set, update } = writable<HistoryState>({
    items: [],
    isLoading: false,
    error: null,
    searchQuery: '',
  });

  return {
    subscribe,
    
    // Actions
    setItems: (items: HistoryItem[]) => update(s => ({ ...s, items })),
    addItem: (item: HistoryItem) => update(s => ({ ...s, items: [item, ...s.items] })),
    updateItem: (id: string, updates: Partial<HistoryItem>) => update(s => ({
      ...s,
      items: s.items.map(item => item.id === id ? { ...item, ...updates } : item),
    })),
    removeItem: (id: string) => update(s => ({
      ...s,
      items: s.items.filter(item => item.id !== id),
    })),
    clear: () => update(s => ({ ...s, items: [] })),
    
    setLoading: (isLoading: boolean) => update(s => ({ ...s, isLoading })),
    setError: (error: string | null) => update(s => ({ ...s, error })),
    setSearchQuery: (searchQuery: string) => update(s => ({ ...s, searchQuery })),
    
    reset: () => set({
      items: [],
      isLoading: false,
      error: null,
      searchQuery: '',
    }),
  };
}

export function createHistoryDialogStore() {
  const { subscribe, set, update } = writable<HistoryDialogState>({
    rename: { open: false, item: null, newName: '' },
    delete: { open: false, item: null },
    deleteAll: { open: false },
    export: { open: false, path: '' },
    import: { open: false, path: '' },
    exportRun: { open: false, item: null, path: '' },
  });

  return {
    subscribe,
    
    // Rename dialog
    openRename: (item: HistoryItem) => update(s => ({
      ...s,
      rename: { open: true, item, newName: item.name || '' },
    })),
    closeRename: () => update(s => ({ ...s, rename: { ...s.rename, open: false } })),
    setRenameName: (newName: string) => update(s => ({ ...s, rename: { ...s.rename, newName } })),
    
    // Delete dialog
    openDelete: (item: HistoryItem) => update(s => ({
      ...s,
      delete: { open: true, item },
    })),
    closeDelete: () => update(s => ({ ...s, delete: { ...s.delete, open: false } })),
    
    // Delete all dialog
    openDeleteAll: () => update(s => ({ ...s, deleteAll: { open: true } })),
    closeDeleteAll: () => update(s => ({ ...s, deleteAll: { open: false } })),
    
    // Export dialog
    openExport: () => update(s => ({ ...s, export: { open: true, path: '' } })),
    closeExport: () => update(s => ({ ...s, export: { ...s.export, open: false } })),
    setExportPath: (path: string) => update(s => ({ ...s, export: { ...s.export, path } })),
    
    // Import dialog
    openImport: () => update(s => ({ ...s, import: { open: true, path: '' } })),
    closeImport: () => update(s => ({ ...s, import: { ...s.import, open: false } })),
    setImportPath: (path: string) => update(s => ({ ...s, import: { ...s.import, path } })),
    
    // Export run dialog
    openExportRun: (item: HistoryItem) => update(s => ({
      ...s,
      exportRun: { open: true, item, path: '' },
    })),
    closeExportRun: () => update(s => ({ ...s, exportRun: { ...s.exportRun, open: false } })),
    setExportRunPath: (path: string) => update(s => ({ ...s, exportRun: { ...s.exportRun, path } })),
    
    reset: () => set({
      rename: { open: false, item: null, newName: '' },
      delete: { open: false, item: null },
      deleteAll: { open: false },
      export: { open: false, path: '' },
      import: { open: false, path: '' },
      exportRun: { open: false, item: null, path: '' },
    }),
  };
}

// ============================================================================
// Derived Stores
// ============================================================================

export function createFilteredHistoryStore(historyStore: ReturnType<typeof createHistoryStore>) {
  return derived(historyStore, $history => {
    const query = $history.searchQuery.toLowerCase();
    if (!query) return $history.items;
    
    return $history.items.filter(item =>
      item.url.toLowerCase().includes(query) ||
      (item.name?.toLowerCase().includes(query) ?? false)
    );
  });
}

export function createHistoryStatsStore(historyStore: ReturnType<typeof createHistoryStore>) {
  return derived(historyStore, $history => {
    const items = $history.items;
    const scored = items.filter(i => i.summary.seo_score != null);
    
    return {
      total: items.length,
      avgScore: scored.length > 0
        ? scored.reduce((sum, i) => sum + (i.summary.seo_score ?? 0), 0) / scored.length
        : null,
      totalIssues: items.reduce((sum, i) => sum + i.summary.total_issues, 0),
    };
  });
}

// ============================================================================
// Global Store Instances
// ============================================================================

export const historyStore = createHistoryStore();
export const historyDialogStore = createHistoryDialogStore();
export const filteredHistory = createFilteredHistoryStore(historyStore);
export const historyStats = createHistoryStatsStore(historyStore);
