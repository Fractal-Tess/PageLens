use crate::browser::Browser;
use crate::seo::{SeoAnalyzer, SeoReport};
use crate::snapshot::{Snapshot, SnapshotExt, SnapshotOptions};
use tokio::time::{timeout, Duration};

use super::types::{CrawlOptions, CrawledPage};
use super::url;

pub(super) async fn crawl_single_page(
    browser: &Browser,
    options: &CrawlOptions,
    url: &str,
    depth: usize,
) -> CrawledPage {
    let mut crawled = CrawledPage {
        url: url.to_string(),
        snapshot: Snapshot::new(url.to_string(), String::new()),
        seo_report: SeoReport::new(),
        depth,
        links_found: Vec::new(),
        success: false,
        error: None,
    };

    if url.starts_with("data:") {
        return crawl_data_url(url, depth).await;
    }

    let page_timeout = Duration::from_millis(options.page_timeout_ms);

    let page = match timeout(page_timeout, browser.navigate(url)).await {
        Ok(Ok(page)) => page,
        Ok(Err(err)) => {
            crawled.error = Some(format!("Navigation failed: {err}"));
            return crawled;
        }
        Err(_) => {
            crawled.error = Some(format!(
                "Navigation timed out after {}ms",
                options.page_timeout_ms
            ));
            return crawled;
        }
    };

    let snapshot = match timeout(page_timeout, page.snapshot(SnapshotOptions::default())).await {
        Ok(Ok(snapshot)) => snapshot,
        Ok(Err(err)) => {
            crawled.error = Some(format!("Snapshot failed: {err}"));
            return crawled;
        }
        Err(_) => {
            crawled.error = Some(format!(
                "Snapshot timed out after {}ms",
                options.page_timeout_ms
            ));
            return crawled;
        }
    };

    let seo_report = SeoAnalyzer::analyze(&snapshot);
    let links = url::extract_links(&snapshot.html, url);

    crawled.snapshot = snapshot;
    crawled.seo_report = seo_report;
    crawled.links_found = links;
    crawled.success = true;

    crawled
}

async fn crawl_data_url(url: &str, depth: usize) -> CrawledPage {
    let mut crawled = CrawledPage {
        url: url.to_string(),
        snapshot: Snapshot::new(url.to_string(), String::new()),
        seo_report: SeoReport::new(),
        depth,
        links_found: Vec::new(),
        success: false,
        error: None,
    };

    let html = url::extract_html_from_data_url(url);
    if html.is_empty() {
        crawled.error = Some("Failed to extract HTML from data URL".to_string());
        return crawled;
    }

    let title = url::extract_title_from_html(&html);

    let mut snapshot = Snapshot::new(url.to_string(), title);
    snapshot.html = html.clone();

    let seo_report = SeoAnalyzer::analyze(&snapshot);
    let links = url::extract_links(&html, url);

    crawled.snapshot = snapshot;
    crawled.seo_report = seo_report;
    crawled.links_found = links;
    crawled.success = true;

    crawled
}
