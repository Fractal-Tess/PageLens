//! SEO analysis tests — Step 3 of TDD
//!
//! Tests for SEO checks:
//! - Meta tags (title, description, viewport, charset)
//! - Open Graph tags
//! - Twitter Card tags
//! - Canonical URL
//! - Headings hierarchy (H1-H6)
//! - Image alt attributes
//! - Structured data (JSON-LD)
//!
//! These tests use the running test applications instead of embedded HTML.
//! Run `bun run start-test-apps` from the project root before running tests.

mod common;

use pagelens_core::seo::Severity;

// ============================================================================
// SEO Analyzer Tests - Next.js Test App
// ============================================================================

#[tokio::test]
async fn nextjs_homepage_has_complete_meta_tags() {
    if !common::nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    common::assert_has_meta_tags(
        &common::nextjs_url(),
        &["title", "description", "viewport", "charset"],
    )
    .await
    .expect("Should have complete meta tags");
}

#[tokio::test]
async fn nextjs_homepage_has_open_graph_tags() {
    if !common::nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    common::assert_has_meta_tags(
        &common::nextjs_url(),
        &["og:title", "og:description"],
    )
    .await
    .expect("Should have Open Graph tags");
}

#[tokio::test]
async fn nextjs_homepage_has_twitter_card_tags() {
    if !common::nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    let report = common::seo_report_from_url(&common::nextjs_url())
        .await
        .expect("Should get SEO report");

    assert!(
        report.twitter_card.card.is_some(),
        "Homepage should have Twitter card type"
    );
    assert!(
        report.twitter_card.title.is_some(),
        "Homepage should have Twitter title"
    );
}

#[tokio::test]
async fn nextjs_homepage_has_canonical_url() {
    if !common::nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    common::assert_has_meta_tags(&common::nextjs_url(), &["canonical"])
        .await
        .expect("Should have canonical URL");
}

#[tokio::test]
async fn nextjs_homepage_has_single_h1() {
    if !common::nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    let report = common::seo_report_from_url(&common::nextjs_url())
        .await
        .expect("Should get SEO report");

    assert_eq!(
        report.headings.h1_count, 1,
        "Homepage should have exactly one H1"
    );
}

#[tokio::test]
async fn nextjs_seo_score_is_calculated() {
    if !common::nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    let report = common::seo_report_from_url(&common::nextjs_url())
        .await
        .expect("Should get SEO report");

    assert!(
        report.score >= 0.0 && report.score <= 100.0,
        "Score should be between 0 and 100, got {:.1}",
        report.score
    );
}

#[tokio::test]
async fn nextjs_seo_issues_have_severity() {
    if !common::nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    let services_url = format!("{}/services", common::nextjs_url());
    let report = common::seo_report_from_url(&services_url)
        .await
        .expect("Should get SEO report");

    // Services page should have issues (intentionally poor SEO)
    assert!(
        !report.issues.is_empty(),
        "Services page should have SEO issues"
    );

    // Check that issues have severity levels
    for issue in &report.issues {
        assert!(
            matches!(
                issue.severity,
                Severity::Error | Severity::Warning | Severity::Info
            ),
            "Issue should have a valid severity"
        );
    }
}

// ============================================================================
// SEO Analyzer Tests - SvelteKit Test App
// ============================================================================

#[tokio::test]
async fn svelte_homepage_has_meta_tags() {
    if !common::svelte_available().await {
        common::skip_or_fail("SvelteKit");
        return;
    }

    let report = common::seo_report_from_url(&common::svelte_url())
        .await
        .expect("Should get SEO report");

    // SvelteKit homepage should have basic meta tags
    assert!(report.meta.title.is_some(), "Should have title");
    assert!(
        report.score > 0.0,
        "Should have a positive SEO score, got {:.1}",
        report.score
    );
}

#[tokio::test]
async fn svelte_seo_test_page_has_issues() {
    if !common::svelte_available().await {
        common::skip_or_fail("SvelteKit");
        return;
    }

    let seo_test_url = format!("{}/seo-test", common::svelte_url());
    let report = common::seo_report_from_url(&seo_test_url)
        .await
        .expect("Should get SEO report");

    // SEO test page should have intentional issues
    assert!(
        !report.issues.is_empty(),
        "SEO test page should have at least one issue"
    );

    // Check that issues have valid categories
    for issue in &report.issues {
        assert!(
            ["meta", "open-graph", "headings", "images", "canonical", "structured-data"]
                .contains(&issue.category.as_str()),
            "Issue category '{}' is not a known category",
            issue.category
        );
    }
}
