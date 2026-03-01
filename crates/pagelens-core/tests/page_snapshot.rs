//! Page snapshot tests — Step 2 of TDD
//!
//! Tests:
//! - can_capture_accessibility_tree — Gets the a11y tree from CDP
//! - can_capture_performance_timing — Gets navigation timing metrics
//! - can_capture_computed_styles — Extracts styles for contrast checking
//!
//! These tests use the running test applications instead of embedded HTML.
//! Run `bun run start-test-apps` from the project root before running tests.

mod common;

use pagelens_core::snapshot::SnapshotOptions;
use pagelens_core::SnapshotExt;
use std::time::Duration;
use tokio::time::timeout;

fn tree_contains_role(nodes: &[pagelens_core::snapshot::AccessibilityNode], role: &str) -> bool {
    nodes
        .iter()
        .any(|node| node.role == role || tree_contains_role(&node.children, role))
}

// ============================================================================
// Snapshot Tests (using shared browser)
// ============================================================================

#[tokio::test]
async fn can_capture_page_snapshot_from_nextjs() {
    let result = timeout(Duration::from_secs(15), async {
        if !common::nextjs_available().await {
            common::skip_or_fail("Next.js");
            return;
        }

        let snapshot = common::snapshot_from_url(&common::nextjs_url())
            .await
            .expect("Should capture snapshot");

        assert!(!snapshot.html.is_empty(), "HTML should be captured");
        assert!(!snapshot.title.is_empty(), "Title should be captured");
        assert!(
            snapshot.url.contains("localhost"),
            "URL should be localhost"
        );
    }).await;

    if result.is_err() {
        panic!("Test timed out after 15 seconds");
    }
}

#[tokio::test]
async fn can_capture_page_snapshot_from_svelte() {
    if !common::svelte_available().await {
        common::skip_or_fail("SvelteKit");
        return;
    }

    let snapshot = common::snapshot_from_url(&common::svelte_url())
        .await
        .expect("Should capture snapshot");

    assert!(!snapshot.html.is_empty(), "HTML should be captured");
    assert!(!snapshot.title.is_empty(), "Title should be captured");
}

#[tokio::test]
async fn can_capture_accessibility_tree_from_nextjs() {
    if !common::nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    let about_url = format!("{}/about", common::nextjs_url());
    let snapshot = common::snapshot_from_url(&about_url)
        .await
        .expect("Should capture snapshot");

    // Check accessibility tree - the JS implementation extracts roles from the page
    let a11y_tree = &snapshot.accessibility_tree;

    // Should have captured some accessibility nodes
    assert!(
        !a11y_tree.is_empty(),
        "Accessibility tree should not be empty"
    );

    // Look for expected roles (e.g., "heading" for the about page)
    let has_heading = tree_contains_role(a11y_tree, "heading");
    assert!(has_heading, "Accessibility tree should contain headings");
}

#[tokio::test]
async fn can_capture_performance_timing_from_nextjs() {
    if !common::nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    let snapshot = common::snapshot_from_url(&common::nextjs_url())
        .await
        .expect("Should capture snapshot");

    // Performance timing should be present
    let timing = &snapshot.performance_timing;
    assert!(
        timing.navigation_start > 0,
        "Navigation start should be set"
    );
}

#[tokio::test]
async fn can_capture_computed_styles_from_nextjs() {
    if !common::nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    let browser = common::create_browser().await;

    let page = browser
        .navigate(&common::nextjs_url())
        .await
        .expect("Should navigate to URL");

    // Capture snapshot with computed styles
    let options = SnapshotOptions {
        include_computed_styles: true,
        ..Default::default()
    };
    let snapshot = page
        .snapshot(options)
        .await
        .expect("Should capture snapshot");

    // Should have computed styles
    assert!(
        !snapshot.computed_styles.is_empty(),
        "Should have computed styles"
    );
}

#[tokio::test]
async fn snapshot_with_custom_options_from_nextjs() {
    if !common::nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    let browser = common::create_browser().await;

    let page = browser
        .navigate(&common::nextjs_url())
        .await
        .expect("Should navigate to URL");

    // Capture snapshot with minimal options
    let options = SnapshotOptions {
        include_html: true,
        include_accessibility_tree: false,
        include_performance_timing: false,
        include_computed_styles: false,
    };
    let snapshot = page
        .snapshot(options)
        .await
        .expect("Should capture snapshot");

    // Only HTML should be present
    assert!(!snapshot.html.is_empty(), "HTML should be captured");
    assert!(
        snapshot.accessibility_tree.is_empty(),
        "A11y tree should be empty"
    );
    assert!(snapshot.computed_styles.is_empty(), "Styles should be empty");
}

// ============================================================================
// Snapshot Content Tests (using shared browser)
// ============================================================================

#[tokio::test]
async fn snapshot_captures_headings_from_nextjs_about() {
    if !common::nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    let about_url = format!("{}/about", common::nextjs_url());
    let snapshot = common::snapshot_from_url(&about_url)
        .await
        .expect("Should capture snapshot");

    // Check headings are captured
    let html = snapshot.html;
    let lower = html.to_ascii_lowercase();
    assert!(lower.contains("<h1"), "Should capture h1");
}

#[tokio::test]
async fn snapshot_captures_images_from_nextjs_services() {
    if !common::nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    let services_url = format!("{}/services", common::nextjs_url());
    let snapshot = common::snapshot_from_url(&services_url)
        .await
        .expect("Should capture snapshot");

    // Check images are captured
    let html = snapshot.html;
    assert!(
        html.contains("<img") || html.contains("<IMG"),
        "Should capture img tags"
    );
}

#[tokio::test]
async fn snapshot_captures_links_from_nextjs() {
    if !common::nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    let snapshot = common::snapshot_from_url(&common::nextjs_url())
        .await
        .expect("Should capture snapshot");

    // Check links are captured
    let html = snapshot.html;
    assert!(
        html.contains("<a ") || html.contains("<A "),
        "Should capture anchor tags"
    );
}

#[tokio::test]
async fn snapshot_captures_svelte_page_structure() {
    if !common::svelte_available().await {
        common::skip_or_fail("SvelteKit");
        return;
    }

    let snapshot = common::snapshot_from_url(&common::svelte_url())
        .await
        .expect("Should capture snapshot");

    // SvelteKit pages should have basic structure
    let html = snapshot.html;
    assert!(html.contains("<html"), "Should have html tag");
    let lower = html.to_ascii_lowercase();
    assert!(lower.contains("<head"), "Should have head tag");
    assert!(lower.contains("<body"), "Should have body tag");
}

#[tokio::test]
async fn snapshot_extracts_referenced_assets_from_nextjs_about() {
    if !common::nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    let about_url = format!("{}/about", common::nextjs_url());
    let snapshot = common::snapshot_from_url(&about_url)
        .await
        .expect("Should capture snapshot");

    let assets = snapshot.referenced_assets;

    assert!(
        assets
            .javascript
            .iter()
            .any(|u| u.contains("/_next/static/") && u.ends_with(".js")),
        "Should capture Next.js JS bundles"
    );
    assert!(
        assets
            .stylesheets
            .iter()
            .any(|u| u.contains("/_next/static/") && u.ends_with(".css")),
        "Should capture Next.js CSS bundles"
    );
    assert!(
        assets.media.iter().any(|u| u.contains("/images/about-office.jpg")),
        "Should capture media references from page images"
    );
}

#[tokio::test]
async fn snapshot_extracts_referenced_assets_from_svelte_seo_page() {
    if !common::svelte_available().await {
        common::skip_or_fail("SvelteKit");
        return;
    }

    let seo_test_url = format!("{}/seo-test", common::svelte_url());
    let snapshot = common::snapshot_from_url(&seo_test_url)
        .await
        .expect("Should capture snapshot");

    let assets = snapshot.referenced_assets;

    assert!(
        assets
            .javascript
            .iter()
            .any(|u| u.contains("/_app/immutable/") && u.ends_with(".js")),
        "Should capture Svelte JS bundles"
    );
    assert!(
        assets
            .stylesheets
            .iter()
            .any(|u| u.contains("/_app/immutable/") && u.ends_with(".css")),
        "Should capture Svelte CSS bundles"
    );
}
