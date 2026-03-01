//! Integration tests for test-apps (Next.js and SvelteKit sites)
//!
//! These tests verify page-specific SEO characteristics against the actual
//! built test applications. Basic SEO checks (meta tags, OG, Twitter, etc.)
//! are covered in seo_tests.rs. Crawl behavior is covered in crawl_tests.rs.
//!
//! This file focuses on:
//!   - Page-specific structural validation (heading hierarchy, structured data, etc.)
//!   - Intentionally poor SEO detection (services, legacy, draft pages)
//!   - Cross-framework comparisons
//!
//! Prerequisites:
//!   Run `bun run start-test-apps` from the project root to start both servers.

mod common;

use common::{nextjs_available, nextjs_url, svelte_available, svelte_url};

// ============================================================================
// Next.js — Page-specific structure tests
// ============================================================================

#[tokio::test]
async fn nextjs_about_page_has_proper_structure() {
    if !nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    let report = common::seo_report_from_url(&format!("{}/about", nextjs_url()))
        .await
        .expect("Should get SEO report");

    assert_eq!(
        report.headings.h1_count, 1,
        "About page should have one H1"
    );
    assert!(
        report.headings.h2_count >= 2,
        "About page should have multiple H2s, found {}",
        report.headings.h2_count
    );

    assert!(
        !report.structured_data.is_empty(),
        "About page should have structured data"
    );
}

#[tokio::test]
async fn nextjs_services_page_has_poor_seo() {
    if !nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    let report = common::seo_report_from_url(&format!("{}/services", nextjs_url()))
        .await
        .expect("Should get SEO report");

    // Services page intentionally has no H1
    assert_eq!(
        report.headings.h1_count, 0,
        "Services page should have no H1 (poor SEO)"
    );

    let has_heading_issues = report.issues.iter().any(|i| i.category == "headings");
    assert!(has_heading_issues, "Services page should have heading issues");

    let has_image_issues = report.issues.iter().any(|i| i.category == "images");
    assert!(has_image_issues, "Services page should have image alt issues");

    // Score should reflect poor SEO
    assert!(
        report.score < 80.0,
        "Services page should have a lower score due to issues, got {:.1}",
        report.score
    );
}

#[tokio::test]
async fn nextjs_legacy_page_has_multiple_h1s() {
    if !nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    let report = common::seo_report_from_url(&format!("{}/legacy", nextjs_url()))
        .await
        .expect("Should get SEO report");

    assert_eq!(
        report.headings.h1_count, 4,
        "Legacy page should have exactly 4 H1s"
    );

    let has_multiple_h1_issue = report
        .issues
        .iter()
        .any(|i| i.message.contains("Multiple H1") || i.message.contains("multiple H1"));
    assert!(has_multiple_h1_issue, "Should detect multiple H1 issue");
}

#[tokio::test]
async fn nextjs_draft_page_has_short_title() {
    if !nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    let report = common::seo_report_from_url(&format!("{}/draft", nextjs_url()))
        .await
        .expect("Should get SEO report");

    if let Some(title) = &report.meta.title {
        assert!(
            title.len() < 10,
            "Draft page should have short title (< 10 chars), got '{}' ({} chars)",
            title,
            title.len()
        );
    }

    let has_short_title_warning = report
        .issues
        .iter()
        .any(|i| i.message.contains("too short") && i.category == "meta");
    assert!(has_short_title_warning, "Should detect short title warning");
}

#[tokio::test]
async fn nextjs_blog_post_has_article_structured_data() {
    if !nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    let report =
        common::seo_report_from_url(&format!("{}/blog/solar-energy-2024", nextjs_url()))
            .await
            .expect("Should get SEO report");

    let has_article_schema = report
        .structured_data
        .iter()
        .any(|s| s.schema_type == "Article");
    assert!(
        has_article_schema,
        "Blog post should have Article structured data"
    );

    assert_eq!(
        report.headings.h1_count, 1,
        "Blog post should have one H1"
    );
}

// ============================================================================
// SvelteKit — Page-specific structure tests
// ============================================================================

#[tokio::test]
async fn svelte_homepage_has_proper_seo() {
    if !svelte_available().await {
        common::skip_or_fail("SvelteKit");
        return;
    }

    let report = common::seo_report_from_url(&svelte_url())
        .await
        .expect("Should get SEO report");

    assert!(
        report.meta.title.is_some(),
        "Svelte homepage should have a title"
    );
    assert!(
        report.headings.h1_count >= 1,
        "Svelte homepage should have at least one H1"
    );
    assert!(
        report.score > 0.0,
        "Svelte homepage should have a positive SEO score, got {:.1}",
        report.score
    );
}

#[tokio::test]
async fn svelte_seo_test_page_has_comprehensive_meta() {
    if !svelte_available().await {
        common::skip_or_fail("SvelteKit");
        return;
    }

    let report = common::seo_report_from_url(&format!("{}/seo-test", svelte_url()))
        .await
        .expect("Should get SEO report");

    // The seo-test page is designed with maximum metadata
    assert!(
        report.meta.title.is_some(),
        "SEO test page should have title"
    );
    assert!(
        report.meta.description.is_some(),
        "SEO test page should have description"
    );
    assert!(
        report.meta.viewport,
        "SEO test page should have viewport"
    );
    assert!(
        report.meta.charset,
        "SEO test page should have charset"
    );

    // Should have OG tags
    assert!(
        report.open_graph.title.is_some(),
        "SEO test page should have og:title"
    );
    assert!(
        report.open_graph.description.is_some(),
        "SEO test page should have og:description"
    );

    // Should have Twitter card
    assert!(
        report.twitter_card.card.is_some(),
        "SEO test page should have twitter:card"
    );

    // Should have structured data (TechArticle JSON-LD)
    assert!(
        !report.structured_data.is_empty(),
        "SEO test page should have structured data"
    );
}

#[tokio::test]
async fn svelte_products_page_loads_and_has_structure() {
    if !svelte_available().await {
        common::skip_or_fail("SvelteKit");
        return;
    }

    let snapshot = common::snapshot_from_url(&format!("{}/products", svelte_url()))
        .await
        .expect("Should capture snapshot");

    assert!(!snapshot.html.is_empty(), "Products page should have HTML content");
    assert!(!snapshot.title.is_empty(), "Products page should have a title");

    // Verify it has structural HTML elements
    assert!(
        snapshot.html.contains("<h1") || snapshot.html.contains("<H1"),
        "Products page should have a heading"
    );
}

#[tokio::test]
async fn svelte_blog_page_loads_and_has_structure() {
    if !svelte_available().await {
        common::skip_or_fail("SvelteKit");
        return;
    }

    let report = common::seo_report_from_url(&format!("{}/blog", svelte_url()))
        .await
        .expect("Should get SEO report");

    assert!(
        report.meta.title.is_some(),
        "Blog page should have a title"
    );
    assert!(
        report.headings.h1_count >= 1,
        "Blog page should have at least one H1"
    );
}

#[tokio::test]
async fn svelte_forms_page_loads() {
    if !svelte_available().await {
        common::skip_or_fail("SvelteKit");
        return;
    }

    let snapshot = common::snapshot_from_url(&format!("{}/forms", svelte_url()))
        .await
        .expect("Should capture snapshot");

    assert!(!snapshot.html.is_empty(), "Forms page should have HTML content");
    assert!(!snapshot.title.is_empty(), "Forms page should have a title");

    // Forms page should contain form elements
    assert!(
        snapshot.html.contains("<form") || snapshot.html.contains("<input"),
        "Forms page should contain form elements"
    );
}

// ============================================================================
// Cross-App Comparison Tests
// ============================================================================

#[tokio::test]
async fn compare_homepage_seo_scores() {
    let nextjs_ready = nextjs_available().await;
    let svelte_ready = svelte_available().await;

    if !nextjs_ready || !svelte_ready {
        common::skip_or_fail("Next.js + SvelteKit");
        return;
    }

    let nextjs_report = common::seo_report_from_url(&nextjs_url())
        .await
        .expect("Should get Next.js SEO report");

    let svelte_report = common::seo_report_from_url(&svelte_url())
        .await
        .expect("Should get SvelteKit SEO report");

    // Both should have reasonable scores
    assert!(
        nextjs_report.score > 50.0,
        "Next.js homepage should have decent SEO score, got {:.1}",
        nextjs_report.score
    );
    assert!(
        svelte_report.score > 50.0,
        "Svelte homepage should have decent SEO score, got {:.1}",
        svelte_report.score
    );

    // Both should have basic meta tags
    assert!(nextjs_report.meta.title.is_some(), "Next.js should have title");
    assert!(svelte_report.meta.title.is_some(), "SvelteKit should have title");
}
