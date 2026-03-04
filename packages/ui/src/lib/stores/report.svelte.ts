import { writable, derived } from 'svelte/store'

// ============================================================================
// Types
// ============================================================================

export type Severity = 'Error' | 'Warning' | 'Info'

export interface Issue {
  severity: Severity
  message: string
  category: string
}

export interface MetaInfo {
  charset: boolean
  viewport: boolean
  title: string | null
  description: string | null
  language: string | null
  robots: string | null
}

export interface OpenGraphInfo {
  title: string | null
  description: string | null
  og_type: string | null
  url: string | null
  image: string | null
}

export interface TwitterCardInfo {
  card: string | null
  title: string | null
  description: string | null
  image: string | null
}

export interface HeadingsInfo {
  h1_count: number
  h2_count: number
  h3_count: number
  h4_count: number
  h5_count: number
  h6_count: number
  structure: string[]
}

export interface ImageInfo {
  src: string
  alt: string | null
  has_alt: boolean
}

export interface StructuredData {
  schema_type: string
  raw: string
}

export interface SeoReport {
  score: number
  meta: MetaInfo
  open_graph: OpenGraphInfo
  twitter_card: TwitterCardInfo
  canonical_url: string | null
  headings: HeadingsInfo
  images: ImageInfo[]
  structured_data: StructuredData[]
  issues: Issue[]
}

export interface PerformanceTiming {
  navigation_start: number
  dom_interactive: number | null
  dom_content_loaded: number | null
  load_complete: number | null
  response_start: number | null
  first_paint: number | null
  first_contentful_paint: number | null
}

export interface AccessibilityNode {
  role: string | null
  name: string | null
  level: number | null
  children: AccessibilityNode[]
}

export interface CrawledPage {
  url: string
  depth: number
  success: boolean
  error: string | null
  links_found: string[]
  seo_report: SeoReport
}

export interface SitemapEntry {
  url: string
  found: boolean
  status: number | null
  kind: string
  urls: string[]
  child_sitemaps: string[]
  parse_error: string | null
}

export interface SiteFileEntry {
  path: string
  found: boolean
  status: number | null
}

export interface SiteFilesReport {
  robots: {
    found: boolean
    status: number | null
    disallow_all_for_star: boolean
    sitemap_directives: string[]
  }
  sitemaps: SitemapEntry[]
  misc_files: SiteFileEntry[]
  issues: Issue[]
}

export interface CrawlStats {
  total_pages: number
  crawled_pages: number
  failed_pages: number
  external_links: number
  crawl_time_ms: number
  avg_page_load_ms: number
}

export interface AggregateMetrics {
  avg_seo_score: number
  min_seo_score: number
  max_seo_score: number
  total_issues: number
  total_errors: number
  total_warnings: number
  score_distribution: number[]
}

export interface ReportViewState {
  activeTab: string
  expandedSections: Set<string>
  showRawJson: boolean
}

// ============================================================================
// Utility Functions
// ============================================================================

export function getScoreColor(score: number): string {
  if (score >= 80) return 'text-green-500'
  if (score >= 50) return 'text-amber-500'
  return 'text-red-500'
}

export function getScoreBgColor(score: number): string {
  if (score >= 80) return 'bg-green-500'
  if (score >= 50) return 'bg-amber-500'
  return 'bg-red-500'
}

export function getSeverityVariant(
  severity: Severity
): 'destructive' | 'default' | 'secondary' | 'outline' {
  switch (severity) {
    case 'Error':
      return 'destructive'
    case 'Warning':
      return 'default'
    default:
      return 'secondary'
  }
}

export function getSeverityClass(severity: Severity): string {
  return severity === 'Warning'
    ? 'bg-amber-500 hover:bg-amber-500/80 text-white'
    : ''
}

export function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`
  return `${(ms / 1000).toFixed(1)}s`
}

// ============================================================================
// Store Factories
// ============================================================================

export function createReportViewStore(defaultTab = 'issues') {
  const { subscribe, set, update } = writable<ReportViewState>({
    activeTab: defaultTab,
    expandedSections: new Set(),
    showRawJson: false
  })

  return {
    subscribe,

    setActiveTab: (tab: string) => update(s => ({ ...s, activeTab: tab })),

    toggleSection: (section: string) =>
      update(s => {
        const expanded = new Set(s.expandedSections)
        if (expanded.has(section)) {
          expanded.delete(section)
        } else {
          expanded.add(section)
        }
        return { ...s, expandedSections: expanded }
      }),

    toggleRawJson: () => update(s => ({ ...s, showRawJson: !s.showRawJson })),

    reset: () =>
      set({
        activeTab: defaultTab,
        expandedSections: new Set(),
        showRawJson: false
      })
  }
}

export function createIssuesStore(issues: Issue[] = []) {
  const { subscribe, set, update } = writable<Issue[]>(issues)

  return {
    subscribe,
    set,

    // Derived stores
    errors: derived({ subscribe }, $issues =>
      $issues.filter(i => i.severity === 'Error')
    ),
    warnings: derived({ subscribe }, $issues =>
      $issues.filter(i => i.severity === 'Warning')
    ),
    byCategory: derived({ subscribe }, $issues => {
      return $issues.reduce(
        (acc, issue) => {
          const bucket = acc[issue.category] ?? []
          bucket.push(issue)
          acc[issue.category] = bucket
          return acc
        },
        {} as Record<string, Issue[]>
      )
    }),

    reset: () => set([])
  }
}

// ============================================================================
// Global Store Instances
// ============================================================================

export const reportViewStore = createReportViewStore()
