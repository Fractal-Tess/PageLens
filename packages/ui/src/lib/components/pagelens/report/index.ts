// ============================================================================
// Report Components - Compound Pattern
// ============================================================================

export { default as Report } from './report.svelte'
export { default as ReportHeader } from './report-header.svelte'
export { default as ReportScoreCard } from './report-score-card.svelte'
export { default as ReportStatsGrid } from './report-stats-grid.svelte'
export { default as ReportTabPanel } from './report-tab-panel.svelte'

// Re-export sub-components
export { default as ScoreGauge } from './score-gauge.svelte'
export { default as IssuesTable } from './issues-table.svelte'
export { default as MetaDetails } from './meta-details.svelte'
export { default as SocialMeta } from './social-meta.svelte'
export { default as HeadingStructure } from './heading-structure.svelte'
export { default as ImagesTable } from './images-table.svelte'
export { default as PerformanceTimeline } from './performance-timeline.svelte'
export { default as CrawlOverview } from './crawl-overview.svelte'
export { default as CrawlPagesTable } from './crawl-pages-table.svelte'
export { default as SiteFilesReportCard } from './site-files-report.svelte'
export { default as DomTree } from './dom-tree.svelte'

// Types
export type * from './types'
