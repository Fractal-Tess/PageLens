//! Crawl tests — Multi-page auditing with link following
//!
//! Tests:
//! - can_crawl_single_page — Basic crawl of one page
//! - respects_max_pages_limit — Stop at max_pages
//! - tracks_crawl_statistics — Statistics collection
//! - aggregates_seo_scores — Score aggregation across pages
//! - handles_invalid_url — Error handling for bad URLs
//! - crawl_respects_follow_external_links — External link filtering
//! - crawl_generates_summary_report — Summary generation
//!
//! Integration tests use the running test applications.
//! Run `bun run start-apps` from the project root before running tests.

mod common;

use pagelens_core::browser::Browser;
use pagelens_core::crawl::{CrawlOptions, Crawler};

/// HTML for a simple multi-page site (used for unit-style tests)
const PAGE_INDEX: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Home - Crawl Test Site</title>
    <meta name="description" content="Home page of crawl test site">
</head>
<body>
    <h1>Welcome to Crawl Test</h1>
    <nav>
        <a href="/about">About</a>
        <a href="/products">Products</a>
        <a href="https://external.com">External Link</a>
    </nav>
    <p>This is the home page.</p>
</body>
</html>
"#;

// ============================================================================
// Basic Crawl Tests (data URLs — share a single browser)
// ============================================================================

#[tokio::test]
async fn can_crawl_single_page() {
    let browser = Browser::launch().await.expect("Should launch browser");

    let crawler = Crawler::new(&browser, CrawlOptions::default());

    let encoded = urlencoding::encode(PAGE_INDEX);
    let data_url = format!("data:text/html,{}", encoded);

    let result = crawler.crawl(&data_url).await;

    assert!(result.is_ok(), "Crawl should succeed");
    let crawl_result = result.unwrap();

    assert!(
        !crawl_result.pages.is_empty(),
        "Should have at least 1 page"
    );

    let seed_page = crawl_result
        .pages
        .iter()
        .find(|p| p.url == data_url)
        .expect("Should have the seed page");
    assert_eq!(
        seed_page.seo_report.meta.title.as_deref(),
        Some("Home - Crawl Test Site")
    );

    assert!(
        !seed_page.links_found.is_empty(),
        "Should find links on the page"
    );
}

#[tokio::test]
async fn respects_max_pages_limit() {
    let browser = Browser::launch().await.expect("Should launch browser");

    let options = CrawlOptions {
        max_pages: 2,
        ..Default::default()
    };
    let crawler = Crawler::new(&browser, options);

    let encoded = urlencoding::encode(PAGE_INDEX);
    let data_url = format!("data:text/html,{}", encoded);

    let result = crawler.crawl(&data_url).await;

    assert!(result.is_ok(), "Crawl should succeed");
    let crawl_result = result.unwrap();

    assert!(
        crawl_result.pages.len() <= 2,
        "Should not crawl more than max_pages"
    );
}

#[tokio::test]
async fn tracks_crawl_statistics() {
    let browser = Browser::launch().await.expect("Should launch browser");

    let crawler = Crawler::new(&browser, CrawlOptions::default());

    let encoded = urlencoding::encode(PAGE_INDEX);
    let data_url = format!("data:text/html,{}", encoded);

    let result = crawler.crawl(&data_url).await;

    assert!(result.is_ok(), "Crawl should succeed");
    let crawl_result = result.unwrap();

    assert!(
        crawl_result.stats.total_pages > 0,
        "Should have total_pages stat"
    );
    assert!(
        crawl_result.stats.crawl_time_ms > 0,
        "Should have crawl_time_ms stat"
    );
}

#[tokio::test]
async fn aggregates_seo_scores() {
    let browser = Browser::launch().await.expect("Should launch browser");

    let crawler = Crawler::new(&browser, CrawlOptions::default());

    let encoded = urlencoding::encode(PAGE_INDEX);
    let data_url = format!("data:text/html,{}", encoded);

    let result = crawler.crawl(&data_url).await;

    assert!(result.is_ok(), "Crawl should succeed");
    let crawl_result = result.unwrap();

    assert!(
        crawl_result.aggregate.avg_seo_score >= 0.0
            && crawl_result.aggregate.avg_seo_score <= 100.0,
        "Average SEO score should be between 0 and 100"
    );

    let total_issues: usize = crawl_result
        .pages
        .iter()
        .map(|p| p.seo_report.issues.len())
        .sum();
    assert_eq!(
        crawl_result.aggregate.total_issues, total_issues,
        "Total issues should match sum of all page issues"
    );
}

#[tokio::test]
async fn handles_invalid_url_gracefully() {
    let browser = Browser::launch().await.expect("Should launch browser");

    let crawler = Crawler::new(&browser, CrawlOptions::default());

    let result = crawler.crawl("not-a-valid-url").await;

    assert!(result.is_err(), "Should fail for invalid URL");
}

#[tokio::test]
async fn crawl_respects_follow_external_links_option() {
    let browser = Browser::launch().await.expect("Should launch browser");

    let options = CrawlOptions {
        follow_external_links: false,
        max_pages: 1,
        ..Default::default()
    };
    let crawler = Crawler::new(&browser, options);

    let encoded = urlencoding::encode(PAGE_INDEX);
    let data_url = format!("data:text/html,{}", encoded);

    let result = crawler.crawl(&data_url).await.unwrap();

    assert_eq!(result.pages.len(), 1, "Should respect max_pages limit");

    assert!(
        !result.pages[0].links_found.is_empty(),
        "Should find links on the page"
    );
}

#[tokio::test]
async fn crawl_generates_summary_report() {
    let browser = Browser::launch().await.expect("Should launch browser");

    let options = CrawlOptions {
        max_pages: 1,
        ..Default::default()
    };
    let crawler = Crawler::new(&browser, options);

    let encoded = urlencoding::encode(PAGE_INDEX);
    let data_url = format!("data:text/html,{}", encoded);

    let result = crawler.crawl(&data_url).await;

    assert!(result.is_ok(), "Crawl should succeed");
    let crawl_result = result.unwrap();

    let summary = crawl_result.summary;
    assert!(
        summary.contains("pages crawled"),
        "Summary should mention pages crawled: {}",
        summary
    );
    assert!(
        summary.contains("Average SEO score"),
        "Summary should mention average SEO score: {}",
        summary
    );
}

// ============================================================================
// Integration Tests with Test Apps (using shared browser)
// ============================================================================

#[tokio::test]
async fn can_crawl_nextjs_test_app() {
    if !common::nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    let options = CrawlOptions {
        max_pages: 10,
        max_depth: 2,
        follow_external_links: false,
        ..Default::default()
    };

    let crawl_result = common::crawl_url(&common::nextjs_url(), options)
        .await
        .expect("Crawl should succeed");

    assert!(
        crawl_result.pages.len() >= 3,
        "Should crawl at least 3 pages, found {}",
        crawl_result.pages.len()
    );

    assert!(
        crawl_result.aggregate.avg_seo_score > 0.0,
        "Average SEO score should be positive"
    );
    assert!(
        crawl_result.stats.crawl_time_ms > 0,
        "Crawl time should be recorded"
    );
}

#[tokio::test]
async fn can_crawl_svelte_test_app() {
    if !common::svelte_available().await {
        common::skip_or_fail("SvelteKit");
        return;
    }

    let options = CrawlOptions {
        max_pages: 10,
        max_depth: 2,
        follow_external_links: false,
        ..Default::default()
    };

    let crawl_result = common::crawl_url(&common::svelte_url(), options)
        .await
        .expect("Crawl should succeed");

    assert!(
        crawl_result.pages.len() >= 2,
        "Should crawl at least 2 pages, found {}",
        crawl_result.pages.len()
    );

    assert!(
        crawl_result.aggregate.avg_seo_score > 0.0,
        "Average SEO score should be positive"
    );
}

#[tokio::test]
async fn crawl_respects_depth_limit_on_nextjs() {
    if !common::nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    let options = CrawlOptions {
        max_pages: 20,
        max_depth: 1,
        follow_external_links: false,
        ..Default::default()
    };

    let result = common::crawl_url(&common::nextjs_url(), options)
        .await
        .expect("Crawl should succeed");

    assert!(
        result.pages.len() >= 2,
        "Should crawl homepage and direct links"
    );

    // All pages should be at depth 0 or 1
    for page in &result.pages {
        assert!(
            page.depth <= 1,
            "Page {} should be at depth <= 1, was {}",
            page.url,
            page.depth
        );
    }
}

#[tokio::test]
async fn crawl_does_not_follow_external_links() {
    if !common::nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    let options = CrawlOptions {
        max_pages: 20,
        max_depth: 2,
        follow_external_links: false,
        ..Default::default()
    };

    let result = common::crawl_url(&common::nextjs_url(), options)
        .await
        .expect("Crawl should succeed");

    // All crawled URLs should be from localhost
    for page in &result.pages {
        assert!(
            page.url.contains("localhost"),
            "Should only crawl localhost URLs, found: {}",
            page.url
        );
    }
}

#[tokio::test]
async fn crawl_handles_404s_gracefully() {
    if !common::nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    let options = CrawlOptions {
        max_pages: 1,
        max_depth: 1,
        ..Default::default()
    };

    let nonexistent_url = format!("{}/this-page-does-not-exist-12345", common::nextjs_url());

    let result = common::crawl_url(&nonexistent_url, options).await;

    // The crawl itself should succeed (it visited the URL), but the page
    // content should reflect the 404 state
    match result {
        Ok(crawl_result) => {
            assert_eq!(
                crawl_result.pages.len(),
                1,
                "Should have attempted exactly 1 page"
            );
            // The page was "visited" — either it loaded a 404 page or failed.
            // Either way, stats should reflect it.
            assert_eq!(
                crawl_result.stats.total_pages, 1,
                "Should record 1 total page attempt"
            );
        }
        Err(_) => {
            // Navigation error is also acceptable for truly unreachable URLs
        }
    }
}

#[tokio::test]
async fn crawl_detects_seo_issues_across_pages() {
    if !common::nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    let options = CrawlOptions {
        max_pages: 10,
        max_depth: 2,
        ..Default::default()
    };

    let result = common::crawl_url(&common::nextjs_url(), options)
        .await
        .expect("Crawl should succeed");

    // The aggregate should reflect total issues
    let total_issues: usize = result.pages.iter().map(|p| p.seo_report.issues.len()).sum();
    assert_eq!(
        result.aggregate.total_issues, total_issues,
        "Aggregate total_issues should match sum of all page issues"
    );

    // At least some pages should have issues (services, legacy, draft pages have intentional issues)
    assert!(
        total_issues > 0,
        "Should find at least some SEO issues across the site"
    );
}

#[tokio::test]
async fn crawl_finds_internal_links_on_nextjs() {
    if !common::nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    let options = CrawlOptions {
        max_pages: 5,
        max_depth: 1,
        ..Default::default()
    };

    let result = common::crawl_url(&common::nextjs_url(), options)
        .await
        .expect("Crawl should succeed");

    // The homepage should have links
    let homepage = result
        .pages
        .iter()
        .find(|p| p.url == common::nextjs_url() || p.url == format!("{}/", common::nextjs_url()));

    assert!(homepage.is_some(), "Should have crawled the homepage");

    let homepage = homepage.unwrap();
    assert!(
        !homepage.links_found.is_empty(),
        "Homepage should have internal links"
    );
}

#[tokio::test]
async fn compare_crawl_results_between_test_apps() {
    let nextjs_ready = common::nextjs_available().await;
    let svelte_ready = common::svelte_available().await;

    if !nextjs_ready || !svelte_ready {
        common::skip_or_fail("Next.js + SvelteKit");
        return;
    }

    let options = CrawlOptions {
        max_pages: 10,
        max_depth: 2,
        follow_external_links: false,
        ..Default::default()
    };

    // Both crawls reuse the shared browser (no need for two browser launches)
    let nextjs_result = common::crawl_url(&common::nextjs_url(), options.clone())
        .await
        .expect("Next.js crawl should succeed");

    let svelte_result = common::crawl_url(&common::svelte_url(), options)
        .await
        .expect("SvelteKit crawl should succeed");

    // Both should have found multiple pages
    assert!(
        nextjs_result.pages.len() >= 3,
        "Next.js should have at least 3 pages"
    );
    assert!(
        svelte_result.pages.len() >= 2,
        "SvelteKit should have at least 2 pages"
    );

    // Both should have aggregate scores in valid range
    assert!(
        nextjs_result.aggregate.avg_seo_score >= 0.0
            && nextjs_result.aggregate.avg_seo_score <= 100.0,
        "Next.js avg score should be in range"
    );
    assert!(
        svelte_result.aggregate.avg_seo_score >= 0.0
            && svelte_result.aggregate.avg_seo_score <= 100.0,
        "SvelteKit avg score should be in range"
    );
}
