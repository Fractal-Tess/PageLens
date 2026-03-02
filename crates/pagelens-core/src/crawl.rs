//! Multi-page crawling with link following

use crate::prelude::*;
use crate::browser::Browser;
use crate::seo::{SeoAnalyzer, SeoReport};
use crate::snapshot::{Snapshot, SnapshotExt, SnapshotOptions};
use futures::future::join_all;
use scraper::{Html, Selector};
use std::collections::{HashSet, VecDeque};
use std::sync::LazyLock;
use std::time::Instant;
use tokio::time::{Duration, timeout};

// Pre-compiled CSS selector for link extraction
static LINK_SELECTOR: LazyLock<Selector> = LazyLock::new(|| {
    Selector::parse("a[href]").expect("valid link selector")
});

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
    /// Pages successfully crawled.
    pub crawled_pages: usize,
    /// Pages that failed to crawl.
    pub failed_pages: usize,
    /// External links found (not crawled).
    pub external_links: usize,
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

/// Crawler for multi-page auditing.
pub struct Crawler<'a> {
    browser: &'a Browser,
    options: CrawlOptions,
}

impl<'a> Crawler<'a> {
    /// Create a new crawler with the given browser and options.
    pub fn new(browser: &'a Browser, options: CrawlOptions) -> Self {
        Self { browser, options }
    }

    /// Crawl starting from a seed URL.
    /// 
    /// This will:
    /// 1. Load the seed page
    /// 2. Extract all internal links
    /// 3. Recursively crawl linked pages up to max_pages/max_depth
    /// 4. Generate aggregate reports
    pub async fn crawl(&self, seed_url: &str) -> Result<CrawlResult> {
        self.crawl_with_callback(seed_url, |_page, _stats| {}).await
    }

    pub async fn crawl_with_callback<F>(&self, seed_url: &str, mut on_page: F) -> Result<CrawlResult>
    where
        F: FnMut(&CrawledPage, &CrawlStats),
    {
        let start_time = Instant::now();
        
        // Validate URL
        if !seed_url.starts_with("http://") && !seed_url.starts_with("https://") && !seed_url.starts_with("data:") {
            return Err(Error::InvalidUrl(seed_url.to_string()));
        }

        let mut result = CrawlResult {
            seed_url: seed_url.to_string(),
            pages: Vec::new(),
            skipped_urls: Vec::new(),
            stats: CrawlStats::default(),
            aggregate: AggregateMetrics::default(),
            summary: String::new(),
        };

        // Track visited URLs to avoid duplicates
        let mut visited: HashSet<String> = HashSet::new();
        let mut queued: HashSet<String> = HashSet::new();
        let mut to_visit: VecDeque<(String, usize)> = VecDeque::new();
        let mut total_page_load_ms: u64 = 0;
        to_visit.push_back((seed_url.to_string(), 0)); // (url, depth)
        queued.insert(seed_url.to_string());

        let max_concurrency = self.options.max_concurrency.max(1);

        while !to_visit.is_empty() {
            let mut batch = Vec::new();

            while batch.len() < max_concurrency {
                let Some((url, depth)) = to_visit.pop_front() else {
                    break;
                };
                queued.remove(&url);

                if result.pages.len() + batch.len() >= self.options.max_pages {
                    result.skipped_urls.push(url);
                    continue;
                }

                if self.options.max_depth > 0 && depth > self.options.max_depth {
                    result.skipped_urls.push(url);
                    continue;
                }

                if visited.contains(&url) {
                    continue;
                }

                visited.insert(url.clone());
                batch.push((url, depth));
            }

            if batch.is_empty() {
                break;
            }

            let batch_results = join_all(batch.into_iter().map(|(url, depth)| async move {
                let page_start = Instant::now();
                let crawled_page = self.crawl_single_page(&url, depth).await;
                let page_load_ms = page_start.elapsed().as_millis() as u64;
                (url, depth, crawled_page, page_load_ms)
            }))
            .await;

            for (url, depth, crawled_page, page_load_ms) in batch_results {
                total_page_load_ms += page_load_ms;

                result.stats.total_pages += 1;
                if crawled_page.success {
                    result.stats.crawled_pages += 1;
                } else {
                    result.stats.failed_pages += 1;
                }

                for link in &crawled_page.links_found {
                    if visited.contains(link) || queued.contains(link) {
                        continue;
                    }

                    if self.should_follow_link(&url, link) {
                        to_visit.push_back((link.clone(), depth + 1));
                        queued.insert(link.clone());
                    } else {
                        result.stats.external_links += 1;
                    }
                }

                result.pages.push(crawled_page);
                if let Some(last_page) = result.pages.last() {
                    on_page(last_page, &result.stats);
                }
            }

            if self.options.delay_ms > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(self.options.delay_ms)).await;
            }
        }

        // Calculate final stats
        result.stats.crawl_time_ms = start_time.elapsed().as_millis() as u64;
        if !result.pages.is_empty() {
            result.stats.avg_page_load_ms = total_page_load_ms / result.pages.len() as u64;
        }

        // Calculate aggregate metrics
        result.aggregate = self.calculate_aggregates(&result.pages);

        // Generate summary
        result.summary = self.generate_summary(&result);

        Ok(result)
    }

    /// Crawl a single page.
    async fn crawl_single_page(&self, url: &str, depth: usize) -> CrawledPage {
        let mut crawled = CrawledPage {
            url: url.to_string(),
            snapshot: Snapshot::new(url.to_string(), String::new()),
            seo_report: SeoReport::new(),
            depth,
            links_found: Vec::new(),
            success: false,
            error: None,
        };

        // For data URLs, extract content directly without browser navigation
        if url.starts_with("data:") {
            return self.crawl_data_url(url, depth).await;
        }

        let page_timeout = Duration::from_millis(self.options.page_timeout_ms);

        // Navigate to the page
        let page = match timeout(page_timeout, self.browser.navigate(url)).await {
            Ok(Ok(p)) => p,
            Ok(Err(e)) => {
                crawled.error = Some(format!("Navigation failed: {e}"));
                return crawled;
            }
            Err(_) => {
                crawled.error = Some(format!(
                    "Navigation timed out after {}ms",
                    self.options.page_timeout_ms
                ));
                return crawled;
            }
        };

        // Capture snapshot
        let snapshot = match timeout(page_timeout, page.snapshot(SnapshotOptions::default())).await {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                crawled.error = Some(format!("Snapshot failed: {e}"));
                return crawled;
            }
            Err(_) => {
                crawled.error = Some(format!(
                    "Snapshot timed out after {}ms",
                    self.options.page_timeout_ms
                ));
                return crawled;
            }
        };

        // Analyze SEO
        let seo_report = SeoAnalyzer::analyze(&snapshot);

        // Extract links from HTML
        let links = Self::extract_links(&snapshot.html, url);

        crawled.snapshot = snapshot;
        crawled.seo_report = seo_report;
        crawled.links_found = links;
        crawled.success = true;

        crawled
    }

    /// Handle data URLs by extracting content directly from the URL.
    async fn crawl_data_url(&self, url: &str, depth: usize) -> CrawledPage {
        let mut crawled = CrawledPage {
            url: url.to_string(),
            snapshot: Snapshot::new(url.to_string(), String::new()),
            seo_report: SeoReport::new(),
            depth,
            links_found: Vec::new(),
            success: false,
            error: None,
        };

        // Extract HTML from data URL
        let html = Self::extract_html_from_data_url(url);
        if html.is_empty() {
            crawled.error = Some("Failed to extract HTML from data URL".to_string());
            return crawled;
        }

        // Extract title from HTML
        let title = Self::extract_title_from_html(&html);

        // Create snapshot
        let mut snapshot = Snapshot::new(url.to_string(), title.clone());
        snapshot.html = html.clone();

        // Analyze SEO
        let seo_report = SeoAnalyzer::analyze(&snapshot);

        // Extract links from HTML
        let links = Self::extract_links(&html, url);

        crawled.snapshot = snapshot;
        crawled.seo_report = seo_report;
        crawled.links_found = links;
        crawled.success = true;

        crawled
    }

    /// Extract HTML content from a data URL.
    fn extract_html_from_data_url(url: &str) -> String {
        // Find the comma that separates the MIME type from the data
        if let Some(comma_pos) = url.find(',') {
            let data_part = &url[comma_pos + 1..];
            
            // For text/html data URLs, URL decode the content
            if url[..comma_pos].contains("text/html") {
                return Self::url_decode(data_part);
            }
            
            // Return as-is for other types
            return data_part.to_string();
        }
        
        String::new()
    }

    /// Extract title from HTML content.
    fn extract_title_from_html(html: &str) -> String {
        // Simple regex-like extraction for title tag
        if let Some(start) = html.to_lowercase().find("<title>") {
            let after_start = &html[start + 7..];
            if let Some(end) = after_start.to_lowercase().find("</title>") {
                return after_start[..end].trim().to_string();
            }
        }
        String::new()
    }

    /// Simple URL decoding for data URLs.
    fn url_decode(input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let mut chars = input.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '%' {
                let mut hex = String::with_capacity(2);
                if let Some(h1) = chars.next() {
                    hex.push(h1);
                }
                if let Some(h2) = chars.next() {
                    hex.push(h2);
                }
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                } else {
                    result.push('%');
                    result.push_str(&hex);
                }
            } else if ch == '+' {
                result.push(' ');
            } else {
                result.push(ch);
            }
        }
        
        result
    }

    /// Extract all links from HTML using CSS selectors.
    fn extract_links(html: &str, base_url: &str) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut links = Vec::new();

        // Parse the HTML and select all <a> tags with href attributes
        let document = Html::parse_fragment(html);

        for element in document.select(&LINK_SELECTOR) {
            if let Some(href) = element.value().attr("href") {
                // Skip anchors and javascript
                if href.starts_with('#') || href.starts_with("javascript:") {
                    continue;
                }

                // Resolve relative URLs
                let absolute_url = Self::resolve_url(base_url, href);

                if let Some(url) = absolute_url {
                    if seen.insert(url.clone()) {
                        links.push(url);
                    }
                }
            }
        }

        links
    }

    /// Resolve a potentially relative URL to absolute.
    fn resolve_url(base: &str, href: &str) -> Option<String> {
        // Already absolute
        if href.starts_with("http://") || href.starts_with("https://") {
            return Some(href.to_string());
        }
        
        // Data URLs - treat as absolute
        if href.starts_with("data:") {
            return Some(href.to_string());
        }
        
        // Protocol-relative
        if href.starts_with("//") {
            if let Some(pos) = base.find("://") {
                let protocol = &base[..pos];
                return Some(format!("{}:{}", protocol, href));
            }
            return None;
        }
        
        // For data URLs, we can't really resolve relative links meaningfully
        // Return as-is for testing purposes
        if base.starts_with("data:") {
            return Some(href.to_string());
        }
        
        // Parse base URL
        let base_parsed = match url::Url::parse(base) {
            Ok(u) => u,
            Err(_) => return None,
        };
        
        // Join with base
        match base_parsed.join(href) {
            Ok(u) => Some(u.to_string()),
            Err(_) => None,
        }
    }

    /// Determine if we should follow a link.
    fn should_follow_link(&self, current_url: &str, target_url: &str) -> bool {
        // Always follow data URLs in tests
        if current_url.starts_with("data:") || target_url.starts_with("data:") {
            return true;
        }

        let current_parsed = url::Url::parse(current_url).ok();
        let target_parsed = url::Url::parse(target_url).ok();

        let (current_host, target_host) = match (&current_parsed, &target_parsed) {
            (Some(c), Some(t)) => (
                c.host_str().unwrap_or("").to_string(),
                t.host_str().unwrap_or("").to_string(),
            ),
            _ => return false,
        };

        let is_external = current_host != target_host;

        if is_external && !self.options.follow_external_links {
            return false;
        }

        // Check subdomain restriction
        if self.options.same_subdomain_only && current_host != target_host {
            return false;
        }

        true
    }

    /// Calculate aggregate metrics.
    fn calculate_aggregates(&self, pages: &[CrawledPage]) -> AggregateMetrics {
        if pages.is_empty() {
            return AggregateMetrics::default();
        }

        let scores: Vec<f64> = pages.iter()
            .filter(|p| p.success)
            .map(|p| p.seo_report.score)
            .collect();

        if scores.is_empty() {
            return AggregateMetrics::default();
        }

        let avg = scores.iter().sum::<f64>() / scores.len() as f64;
        let min = scores.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = scores.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        let total_issues: usize = pages.iter()
            .map(|p| p.seo_report.issues.len())
            .sum();

        let total_errors: usize = pages.iter()
            .map(|p| p.seo_report.issues.iter().filter(|i| 
                matches!(i.severity, crate::seo::Severity::Error)).count())
            .sum();

        let total_warnings: usize = pages.iter()
            .map(|p| p.seo_report.issues.iter().filter(|i| 
                matches!(i.severity, crate::seo::Severity::Warning)).count())
            .sum();

        // Score distribution
        let mut distribution = vec![0, 0, 0, 0, 0];
        for score in &scores {
            let idx = match *score as usize {
                0..=20 => 0,
                21..=40 => 1,
                41..=60 => 2,
                61..=80 => 3,
                _ => 4,
            };
            distribution[idx] += 1;
        }

        AggregateMetrics {
            avg_seo_score: avg,
            min_seo_score: if min == f64::INFINITY { 0.0 } else { min },
            max_seo_score: if max == f64::NEG_INFINITY { 0.0 } else { max },
            total_issues,
            total_errors,
            total_warnings,
            score_distribution: distribution,
        }
    }

    /// Generate a human-readable summary.
    fn generate_summary(&self, result: &CrawlResult) -> String {
        let agg = &result.aggregate;
        let stats = &result.stats;
        
        format!(
            "Crawl complete: {} pages crawled in {}ms. \
             Average SEO score: {:.0}/100. \
             Total issues: {} ({} errors, {} warnings).",
            stats.crawled_pages,
            stats.crawl_time_ms,
            agg.avg_seo_score,
            agg.total_issues,
            agg.total_errors,
            agg.total_warnings
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_links() {
        let html = "<a href=\"/about\">About</a>\
                    <a href=\"https://example.com/page\">External</a>\
                    <a href=\"#anchor\">Anchor</a>\
                    <a href=\"javascript:void(0)\">JS</a>";
        
        let base = "https://mysite.com/";
        let links = Crawler::extract_links(html, base);
        
        assert_eq!(links.len(), 2);
        assert!(links.contains(&"https://mysite.com/about".to_string()));
        assert!(links.contains(&"https://example.com/page".to_string()));
    }

    #[test]
    fn test_resolve_url() {
        // Absolute URL
        assert_eq!(
            Crawler::resolve_url("https://example.com/", "https://other.com/page"),
            Some("https://other.com/page".to_string())
        );
        
        // Relative URL
        assert_eq!(
            Crawler::resolve_url("https://example.com/dir/", "page.html"),
            Some("https://example.com/dir/page.html".to_string())
        );
        
        // Root-relative URL
        assert_eq!(
            Crawler::resolve_url("https://example.com/", "/page"),
            Some("https://example.com/page".to_string())
        );
    }

    #[test]
    fn test_crawl_options_default() {
        let opts = CrawlOptions::default();
        assert_eq!(opts.max_pages, 50);
        assert_eq!(opts.max_depth, 3);
        assert!(!opts.follow_external_links);
    }
}
