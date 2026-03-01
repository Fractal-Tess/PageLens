export const themes = ['dark', 'light'] as const
export type Theme = (typeof themes)[number]

// ============================================================================
// Analysis Types (mirroring Rust backend types)
// ============================================================================

export type AnalysisType = 'single' | 'crawl'

export interface AnalysisSummary {
  seo_score: number | null
  page_count: number
  total_issues: number
  error_count: number
  warning_count: number
  duration_ms: number
}

export interface AnalysisRun {
  id: string
  url: string
  created_at: string
  name: string | null
  analysis_type: AnalysisType
  payload_json: string
  summary: AnalysisSummary
}

export interface HistoryListItem {
  id: string
  url: string
  created_at: string
  name: string | null
  analysis_type: AnalysisType
  summary: AnalysisSummary
}

export interface AnalysisResult {
  run: AnalysisRun
}

// ============================================================================
// Analysis Input Types
// ============================================================================

export interface SnapshotOptionsInput {
  include_html: boolean
  include_accessibility_tree: boolean
  include_performance_timing: boolean
  include_computed_styles: boolean
}

export interface AnalyzeUrlInput {
  url: string
  name?: string
  options?: SnapshotOptionsInput
}

export interface CrawlOptionsInput {
  max_pages: number
  max_depth: number
  follow_external_links: boolean
  same_subdomain_only: boolean
  page_timeout_ms: number
  delay_ms: number
  max_concurrency: number
}

export interface CrawlUrlInput {
  url: string
  name?: string
  options?: CrawlOptionsInput
}

// ============================================================================
// Progress Events
// ============================================================================

export interface AnalysisProgressEvent {
  stage: string
  message: string
  progress: number | null
}

// ============================================================================
// History Update
// ============================================================================

export interface UpdateAnalysisRun {
  name?: string
}
