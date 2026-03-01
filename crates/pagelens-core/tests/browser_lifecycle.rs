//! Browser lifecycle tests — Step 1 of TDD
//!
//! Tests:
//! - can_find_chromium — Resolves the binary from $PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH, then fallback
//! - can_launch_browser — Spawns headless Chromium, connects via CDP, shuts down cleanly
//! - can_navigate_and_get_html — Navigates to a URL, returns the rendered HTML
//! - can_get_page_title — Basic DOM extraction
//!
//! These tests use the running test applications instead of embedded HTML.
//! Run `bun run start-test-apps` from the project root before running tests.
//!
//! IMPORTANT: These tests must run sequentially (not in parallel) because Chromium
//! can have port conflicts when multiple instances are launched simultaneously.
//! Use: `cargo test --test browser_lifecycle -- --test-threads=1`

mod common;

use pagelens_core::browser::{Browser, ChromiumLocator};

// ============================================================================
// Chromium Discovery Tests
// ============================================================================

#[test]
fn can_find_chromium_via_env_var() {
    // When PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH is set, we should use it
    // This test verifies the locator logic works correctly
    let locator = ChromiumLocator::new();

    // If the env var is set, we should get that path
    if std::env::var("PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH").is_ok() {
        let path = locator.find().expect("Should find Chromium via env var");
        assert!(path.exists(), "Chromium path should exist");
        println!("Found Chromium at: {}", path.display());
    }
}

#[test]
fn can_find_chromium_in_path() {
    // The locator should try common binary names in PATH
    let locator = ChromiumLocator::new();

    // This may or may not succeed depending on the system,
    // but it shouldn't panic
    match locator.find() {
        Ok(path) => {
            println!("Found Chromium in PATH at: {}", path.display());
            assert!(path.exists());
        }
        Err(e) => {
            println!("Chromium not found in PATH (expected in some environments): {e}");
        }
    }
}

// ============================================================================
// Browser Launch & Lifecycle Tests (using isolated browsers)
// ============================================================================

#[tokio::test]
async fn can_launch_and_shutdown_browser() {
    // This test launches a browser instance and ensures it shuts down cleanly
    let browser = Browser::launch().await.expect("Should launch browser");

    // Browser should be connected
    assert!(browser.is_connected(), "Browser should be connected after launch");

    // Explicit shutdown should work (consumes browser)
    browser
        .shutdown()
        .await
        .expect("Should shutdown cleanly");
}

#[tokio::test]
async fn browser_auto_shutdown_on_drop() {
    // Browser should clean up when dropped
    {
        let browser = Browser::launch().await.expect("Should launch browser");
        assert!(browser.is_connected());
        // browser dropped here
    }

    // If we get here without hanging or panicking, drop worked
}

// ============================================================================
// Navigation & DOM Extraction Tests (using shared browser)
// ============================================================================

#[tokio::test]
async fn can_navigate_to_nextjs_homepage() {
    use std::time::Duration;
    use tokio::time::timeout;

    let result = timeout(Duration::from_secs(30), async {
        if !common::nextjs_available().await {
            common::skip_or_fail("Next.js");
            return;
        }

        let snapshot = common::snapshot_from_url(&common::nextjs_url())
            .await
            .expect("Should capture snapshot");

        assert!(!snapshot.html.is_empty(), "Page should have content");
        assert!(snapshot.html.contains("<html"), "Page should contain html tag");
    }).await;

    if result.is_err() {
        panic!("Test timed out after 30 seconds - test apps may not be running. Run 'bun run start-test-apps' first.");
    }
}

#[tokio::test]
async fn can_navigate_to_svelte_homepage() {
    use std::time::Duration;
    use tokio::time::timeout;

    let result = timeout(Duration::from_secs(30), async {
        if !common::svelte_available().await {
            common::skip_or_fail("SvelteKit");
            return;
        }

        let snapshot = common::snapshot_from_url(&common::svelte_url())
            .await
            .expect("Should capture snapshot");

        assert!(!snapshot.html.is_empty(), "Page should have content");
        assert!(snapshot.html.contains("<html"), "Page should contain html tag");
    }).await;

    if result.is_err() {
        panic!("Test timed out after 30 seconds - test apps may not be running. Run 'bun run start-test-apps' first.");
    }
}

#[tokio::test]
async fn can_get_page_title_from_nextjs() {
    use std::time::Duration;
    use tokio::time::timeout;

    let result = timeout(Duration::from_secs(30), async {
        if !common::nextjs_available().await {
            common::skip_or_fail("Next.js");
            return;
        }

        let browser = common::create_browser().await;

        let about_url = format!("{}/about", common::nextjs_url());
        let page = browser
            .navigate(&about_url)
            .await
            .expect("Should navigate to About page");

        let title = page.title().await.expect("Should get page title");

        // The about page should have a meaningful title
        assert!(!title.is_empty(), "Title should not be empty");
        assert!(
            title.to_lowercase().contains("about"),
            "Title should contain 'about', got: {}",
            title
        );
    }).await;

    if result.is_err() {
        panic!("Test timed out after 30 seconds - test apps may not be running. Run 'bun run start-test-apps' first.");
    }
}

#[tokio::test]
async fn can_get_page_title_from_svelte() {
    use std::time::Duration;
    use tokio::time::timeout;

    let result = timeout(Duration::from_secs(30), async {
        if !common::svelte_available().await {
            common::skip_or_fail("SvelteKit");
            return;
        }

        let browser = common::create_browser().await;

        let page = browser
            .navigate(&common::svelte_url())
            .await
            .expect("Should navigate to SvelteKit homepage");

        let title = page.title().await.expect("Should get page title");

        // The homepage should have a title
        assert!(!title.is_empty(), "Title should not be empty");
    }).await;

    if result.is_err() {
        panic!("Test timed out after 30 seconds - test apps may not be running. Run 'bun run start-test-apps' first.");
    }
}

#[tokio::test]
async fn can_navigate_to_example_com() {
    use std::time::Duration;
    use tokio::time::timeout;

    let result = timeout(Duration::from_secs(30), async {
        let browser = Browser::launch().await.expect("Should launch browser");

        let page = browser
            .navigate("https://example.com")
            .await
            .expect("Should navigate to example.com");

        let html = page.html().await.expect("Should get HTML");

        // The about page should have meaningful content
        assert!(!html.is_empty(), "Should have HTML content");
        assert!(
            html.contains("<html"),
            "Should contain html tag"
        );

        browser.shutdown().await.ok();
    }).await;

    if result.is_err() {
        panic!("Test timed out after 30 seconds - test apps may not be running. Run 'bun run start-test-apps' first.");
    }
}

// ============================================================================
// Error Handling Tests (using isolated browsers)
// ============================================================================

#[tokio::test]
async fn navigation_to_invalid_url_fails_gracefully() {
    let browser = Browser::launch().await.expect("Should launch browser");

    // Invalid URL should return an error, not panic
    let result = browser.navigate("not-a-valid-url").await;
    assert!(result.is_err(), "Invalid URL should result in error");

    browser.shutdown().await.ok();
}

#[tokio::test]
async fn navigation_to_nonexistent_host_fails_gracefully() {
    use std::time::Duration;
    use tokio::time::timeout;

    let result = timeout(Duration::from_secs(30), async {
        let browser = Browser::launch().await.expect("Should launch browser");

        // This should timeout or fail gracefully - not panic
        let result = browser.navigate("http://localhost:59999").await;
        // Navigation to nonexistent host should fail
        assert!(
            result.is_err(),
            "Navigation to nonexistent host should fail"
        );

        browser.shutdown().await.ok();
    }).await;

    if result.is_err() {
        panic!("Test timed out after 30 seconds");
    }
}
