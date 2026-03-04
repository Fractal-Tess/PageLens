//! Crawl-related data types

use crate::seo::SeoReport;
use crate::snapshot::Snapshot;

/// Options for crawling a website.
#[derive(Debug, Clone)]
pub struct CrawlOptions {
    /// Maximum number of pages to crawl.
    pub max_pages: usize,
    /// Maximum crawl depth (0 = unlimited).
    pub max_depth: usize,
    /// Whether to follow links to external domains.
    pub follow_external_links: bool,
    /// Whether to stay within the same subdomain.
    pub same_subdomain_only: bool,
    /// Timeout for each page load (milliseconds).
    pub page_timeout_ms: u64,
    /// Delay between page requests (milliseconds).
    pub delay_ms: u64,
    pub max_concurrency: usize,
}

impl Default for CrawlOptions {
    fn default() -> Self {
        Self {
            max_pages: 50,
            max_depth: 3,
            follow_external_links: false,
            same_subdomain_only: true,
            page_timeout_ms: 30000,
            delay_ms: 100,
            max_concurrency: 4,
        }
    }
}

/// Result of crawling a single page.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CrawledPage {
    /// The URL of the page.
    pub url: String,
    /// The page snapshot.
    pub snapshot: Snapshot,
    /// The SEO report for the page.
    pub seo_report: SeoReport,
    /// Crawl depth (0 = seed page).
    pub depth: usize,
    /// Links found on this page.
    pub links_found: Vec<String>,
    /// Whether the page was successfully crawled.
    pub success: bool,
    /// Error message if crawling failed.
    pub error: Option<String>,
}

/// Crawl statistics.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CrawlStats {
    /// Total pages discovered.
    pub total_pages: usize,
    pub queued_pages: usize,
    pub running_pages: usize,
    /// Pages successfully crawled.
    pub crawled_pages: usize,
    /// Pages that failed to crawl.
    pub failed_pages: usize,
    /// External links found (not crawled).
    pub external_links: usize,
    pub links_found_total: usize,
    /// Total crawl time in milliseconds.
    pub crawl_time_ms: u64,
    /// Average page load time in milliseconds.
    pub avg_page_load_ms: u64,
}

/// Aggregate metrics across all crawled pages.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AggregateMetrics {
    /// Average SEO score across all pages.
    pub avg_seo_score: f64,
    /// Minimum SEO score.
    pub min_seo_score: f64,
    /// Maximum SEO score.
    pub max_seo_score: f64,
    /// Total number of issues across all pages.
    pub total_issues: usize,
    /// Total number of errors.
    pub total_errors: usize,
    /// Total number of warnings.
    pub total_warnings: usize,
    /// Distribution of scores (ranges: 0-20, 21-40, 41-60, 61-80, 81-100).
    pub score_distribution: Vec<usize>,
}

/// Complete crawl result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CrawlResult {
    /// The seed URL where crawling started.
    pub seed_url: String,
    /// All pages that were crawled.
    pub pages: Vec<CrawledPage>,
    /// URLs that were discovered but not crawled (limit reached).
    pub skipped_urls: Vec<String>,
    /// Crawl statistics.
    pub stats: CrawlStats,
    /// Aggregate metrics.
    pub aggregate: AggregateMetrics,
    /// Human-readable summary.
    #[serde(skip)]
    pub summary: String,
}
