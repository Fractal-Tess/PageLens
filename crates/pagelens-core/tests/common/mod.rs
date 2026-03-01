#![allow(dead_code)]
//! Common test utilities for integration tests with test-apps
//!
//! This module provides helper functions for connecting to the running
//! test applications (Next.js and SvelteKit).
//!
//! Prerequisites:
//!   - Next.js test app should be running on port 44791 (or $NEXTJS_PORT)
//!   - SvelteKit test app should be running on port 44792 (or $SVELTE_PORT)
//!   - Run `bun run start-test-apps` from the project root to start both

use pagelens_core::browser::Browser;
use pagelens_core::crawl::{CrawlOptions, CrawlResult, Crawler};
use pagelens_core::seo::{SeoAnalyzer, SeoReport};
use pagelens_core::snapshot::{Snapshot, SnapshotExt, SnapshotOptions};
use std::time::Duration;

// ============================================================================
// Browser Creation
// ============================================================================

/// Create a new isolated browser instance for testing.
///
/// Each test gets its own browser to avoid connection sharing issues.
/// Tests using this function should run sequentially (--test-threads=1)
/// to avoid port conflicts with multiple Chromium instances.
pub async fn create_browser() -> Browser {
    Browser::launch()
        .await
        .expect("Failed to launch browser for tests")
}

// ============================================================================
// Snapshot & SEO Helpers (using shared browser)
// ============================================================================

/// Navigate to a URL and capture a snapshot using a fresh browser instance.
pub async fn snapshot_from_url(url: &str) -> anyhow::Result<Snapshot> {
    let browser = create_browser().await;

    let page = browser
        .navigate(url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to navigate to {}: {}", url, e))?;

    let snapshot = page
        .snapshot(SnapshotOptions::default())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to capture snapshot: {}", e))?;

    Ok(snapshot)
}

/// Navigate to a URL and analyze SEO using the shared browser.
pub async fn seo_report_from_url(url: &str) -> anyhow::Result<SeoReport> {
    let snapshot = snapshot_from_url(url).await?;
    Ok(SeoAnalyzer::analyze(&snapshot))
}

/// Assert that a page has a minimum SEO score.
pub async fn assert_seo_score(url: &str, min_score: f64) -> anyhow::Result<()> {
    let report = seo_report_from_url(url).await?;
    anyhow::ensure!(
        report.score >= min_score,
        "SEO score {:.1} is below minimum {:.1}",
        report.score,
        min_score
    );
    Ok(())
}

/// Known meta tag types for compile-time safety.
pub enum MetaTag {
    Title,
    Description,
    Viewport,
    Charset,
    OgTitle,
    OgDescription,
    Canonical,
}

impl MetaTag {
    /// Parse a tag name string into a MetaTag variant.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "title" => Some(Self::Title),
            "description" => Some(Self::Description),
            "viewport" => Some(Self::Viewport),
            "charset" => Some(Self::Charset),
            "og:title" => Some(Self::OgTitle),
            "og:description" => Some(Self::OgDescription),
            "canonical" => Some(Self::Canonical),
            _ => None,
        }
    }
}

/// Assert that a page has specific meta tags.
pub async fn assert_has_meta_tags(url: &str, expected: &[&str]) -> anyhow::Result<()> {
    let report = seo_report_from_url(url).await?;

    for tag in expected {
        let meta_tag = MetaTag::from_str(tag)
            .ok_or_else(|| anyhow::anyhow!("Unknown meta tag: {}", tag))?;

        match meta_tag {
            MetaTag::Title => anyhow::ensure!(report.meta.title.is_some(), "Missing title meta tag"),
            MetaTag::Description => anyhow::ensure!(
                report.meta.description.is_some(),
                "Missing description meta tag"
            ),
            MetaTag::Viewport => anyhow::ensure!(report.meta.viewport, "Missing viewport meta tag"),
            MetaTag::Charset => anyhow::ensure!(report.meta.charset, "Missing charset meta tag"),
            MetaTag::OgTitle => anyhow::ensure!(
                report.open_graph.title.is_some(),
                "Missing og:title meta tag"
            ),
            MetaTag::OgDescription => anyhow::ensure!(
                report.open_graph.description.is_some(),
                "Missing og:description meta tag"
            ),
            MetaTag::Canonical => {
                anyhow::ensure!(report.canonical_url.is_some(), "Missing canonical URL")
            }
        }
    }
    Ok(())
}

// ============================================================================
// Crawl Helpers (using shared browser)
// ============================================================================

/// Crawl a URL using a fresh browser instance with given options.
pub async fn crawl_url(url: &str, options: CrawlOptions) -> anyhow::Result<CrawlResult> {
    let browser = create_browser().await;

    let crawler = Crawler::new(&browser, options);
    crawler
        .crawl(url)
        .await
        .map_err(|e| anyhow::anyhow!("Crawl failed for {}: {}", url, e))
}

// ============================================================================
// Environment & Server Helpers
// ============================================================================

/// Check if we're running in CI (tests should fail loudly if servers are unavailable)
pub fn is_ci() -> bool {
    std::env::var("CI").is_ok()
}

/// Skip or panic depending on environment.
///
/// In CI, panics so tests don't silently pass when servers are down.
/// Locally, prints a skip message and returns so the caller can `return` early.
pub fn skip_or_fail(label: &str) {
    let msg = format!("{} not available. {}", label, SKIP_MESSAGE);
    if is_ci() {
        panic!("{}\nTests must not be silently skipped in CI!", msg);
    }
    eprintln!("SKIPPED: {}", msg);
}

/// Default ports for test apps (production/start mode)
pub const DEFAULT_NEXTJS_PORT: u16 = 44791;
pub const DEFAULT_SVELTE_PORT: u16 = 44792;

/// Get the Next.js test app base URL
pub fn nextjs_url() -> String {
    std::env::var("NEXTJS_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .map(|port: u16| format!("http://localhost:{}", port))
        .unwrap_or_else(|| format!("http://localhost:{}", DEFAULT_NEXTJS_PORT))
}

/// Get the SvelteKit test app base URL
pub fn svelte_url() -> String {
    std::env::var("SVELTE_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .map(|port: u16| format!("http://localhost:{}", port))
        .unwrap_or_else(|| format!("http://localhost:{}", DEFAULT_SVELTE_PORT))
}

/// Generic check if a server is available at the given URL
async fn server_available(url: &str) -> bool {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };
    client.get(url).send().await.is_ok()
}

/// Check if Next.js server is available
pub async fn nextjs_available() -> bool {
    server_available(&nextjs_url()).await
}

/// Check if SvelteKit server is available
pub async fn svelte_available() -> bool {
    server_available(&svelte_url()).await
}

/// Skip message when test servers are not available
pub const SKIP_MESSAGE: &str = "Test servers not available. \
    Run 'bun run start-test-apps' from the project root first. \
    Expected: Next.js on http://localhost:44791, SvelteKit on http://localhost:44792";

/// Get a list of all page paths available in the Next.js test app
pub fn nextjs_page_paths() -> Vec<&'static str> {
    vec![
        "/",
        "/about",
        "/blog",
        "/blog/solar-energy-2024",
        "/contact",
        "/draft",
        "/legacy",
        "/services",
        "/slow-page",
    ]
}

/// Get a list of all page paths available in the SvelteKit test app
pub fn svelte_page_paths() -> Vec<&'static str> {
    vec![
        "/",
        "/blog",
        "/complex-layout",
        "/forms",
        "/image-map",
        "/products",
        "/seo-test",
        "/slow-page",
    ]
}
