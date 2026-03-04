// ============================================================================
// Analysis Component Types
// ============================================================================

export type AnalysisType = 'single' | 'crawl' | 'site_files';

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

export interface AnalysisOptions {
  type: AnalysisType;
  url: string;
  name: string;
  singleOptions: SingleAnalysisOptions;
  crawlOptions: CrawlAnalysisOptions;
}

export interface AnalysisFormState {
  isSubmitting: boolean;
  error: string | null;
}
