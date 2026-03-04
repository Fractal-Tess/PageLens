import { writable, derived, type Readable } from 'svelte/store';

// ============================================================================
// Types
// ============================================================================

export type AnalysisStatus = 'idle' | 'pending' | 'running' | 'completed' | 'failed';

export interface AnalysisProgress {
  stage: string;
  message: string;
  progress: number; // 0-100
}

export interface SingleAnalysisOptions {
  includeHtml: boolean;
  includeAccessibilityTree: boolean;
  includePerformanceTiming: boolean;
  includeComputedStyles: boolean;
}

export interface CrawlAnalysisOptions {
  maxPages: number;
  maxDepth: number;
  followExternalLinks: boolean;
  sameSubdomainOnly: boolean;
  pageTimeoutMs: number;
  delayMs: number;
  maxConcurrency: number;
}

export interface AnalysisState {
  url: string;
  name: string;
  status: AnalysisStatus;
  progress: AnalysisProgress;
  error: string | null;
  runId: string | null;
}

// ============================================================================
// Default Values
// ============================================================================

export const DEFAULT_SINGLE_OPTIONS: SingleAnalysisOptions = {
  includeHtml: true,
  includeAccessibilityTree: true,
  includePerformanceTiming: true,
  includeComputedStyles: false,
};

export const DEFAULT_CRAWL_OPTIONS: CrawlAnalysisOptions = {
  maxPages: 50,
  maxDepth: 3,
  followExternalLinks: false,
  sameSubdomainOnly: true,
  pageTimeoutMs: 30000,
  delayMs: 100,
  maxConcurrency: 4,
};

const DEFAULT_PROGRESS: AnalysisProgress = {
  stage: 'idle',
  message: 'Waiting to start...',
  progress: 0,
};

// ============================================================================
// Store Factory
// ============================================================================

export function createAnalysisStore() {
  const { subscribe, set, update } = writable<AnalysisState>({
    url: '',
    name: '',
    status: 'idle',
    progress: { ...DEFAULT_PROGRESS },
    error: null,
    runId: null,
  });

  return {
    subscribe,
    
    // Actions
    setUrl: (url: string) => update(s => ({ ...s, url })),
    setName: (name: string) => update(s => ({ ...s, name })),
    
    start: (runId: string) => update(s => ({
      ...s,
      runId,
      status: 'running',
      progress: { stage: 'starting', message: 'Analysis started...', progress: 0 },
      error: null,
    })),
    
    updateProgress: (progress: Partial<AnalysisProgress>) => update(s => ({
      ...s,
      progress: { ...s.progress, ...progress },
    })),
    
    complete: () => update(s => ({
      ...s,
      status: 'completed',
      progress: { stage: 'complete', message: 'Analysis completed', progress: 100 },
    })),
    
    fail: (error: string) => update(s => ({
      ...s,
      status: 'failed',
      error,
      progress: { stage: 'failed', message: error, progress: s.progress.progress },
    })),
    
    reset: () => set({
      url: '',
      name: '',
      status: 'idle',
      progress: { ...DEFAULT_PROGRESS },
      error: null,
      runId: null,
    }),
    
    // Derived state helpers
    isRunning: derived({ subscribe }, $s => $s.status === 'running' || $s.status === 'pending'),
    isComplete: derived({ subscribe }, $s => $s.status === 'completed'),
    hasError: derived({ subscribe }, $s => $s.status === 'failed'),
  };
}

// ============================================================================
// Options Stores
// ============================================================================

export function createSingleOptionsStore(initial: Partial<SingleAnalysisOptions> = {}) {
  const { subscribe, set, update } = writable<SingleAnalysisOptions>({
    ...DEFAULT_SINGLE_OPTIONS,
    ...initial,
  });

  return {
    subscribe,
    set,
    update,
    toggle: (key: keyof SingleAnalysisOptions) => update(s => ({ ...s, [key]: !s[key] })),
    reset: () => set({ ...DEFAULT_SINGLE_OPTIONS }),
  };
}

export function createCrawlOptionsStore(initial: Partial<CrawlAnalysisOptions> = {}) {
  const { subscribe, set, update } = writable<CrawlAnalysisOptions>({
    ...DEFAULT_CRAWL_OPTIONS,
    ...initial,
  });

  return {
    subscribe,
    set,
    update,
    updateField: <K extends keyof CrawlAnalysisOptions>(key: K, value: CrawlAnalysisOptions[K]) =>
      update(s => ({ ...s, [key]: value })),
    toggle: (key: 'followExternalLinks' | 'sameSubdomainOnly') => update(s => ({ ...s, [key]: !s[key] })),
    reset: () => set({ ...DEFAULT_CRAWL_OPTIONS }),
  };
}

// ============================================================================
// Global Analysis Store Instance
// ============================================================================

export const analysisStore = createAnalysisStore();
