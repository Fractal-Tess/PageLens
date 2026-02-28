//! Browser management — Chromium download, launch, and lifecycle.

use crate::prelude::*;
use std::path::PathBuf;

/// Environment variable for Playwright Chromium path
const PLAYWRIGHT_CHROMIUM_ENV: &str = "PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH";

/// Common Chromium/Chrome binary names to search for
const CHROMIUM_BINARIES: &[&str] = &[
    "chromium",
    "chromium-browser",
    "chrome",
    "google-chrome",
    "google-chrome-stable",
    "msedge",
    "microsoft-edge",
    "microsoft-edge-stable",
];

/// Locates a Chromium binary on the system.
/// 
/// Search order:
/// 1. `$PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH` environment variable
/// 2. System PATH for common binary names
pub struct ChromiumLocator;

impl ChromiumLocator {
    /// Create a new Chromium locator.
    pub fn new() -> Self {
        Self
    }

    /// Find a Chromium binary.
    /// 
    /// Returns the path to the Chromium executable, or an error if not found.
    pub fn find(&self) -> Result<PathBuf> {
        // 1. Check environment variable first
        if let Ok(path) = std::env::var(PLAYWRIGHT_CHROMIUM_ENV) {
            if !path.is_empty() {
                let path = PathBuf::from(path);
                if path.exists() {
                    return Ok(path);
                }
            }
        }

        // 2. Search PATH for common binary names
        for binary in CHROMIUM_BINARIES {
            if let Ok(path) = which::which(binary) {
                return Ok(path);
            }
        }

        Err(Error::ChromiumNotFound)
    }
}

impl Default for ChromiumLocator {
    fn default() -> Self {
        Self::new()
    }
}

/// A handle to a running Chromium browser instance.
/// 
/// The browser is automatically shut down when this handle is dropped,
/// unless explicitly shut down earlier with [`Browser::shutdown`].
pub struct Browser {
    /// The chromiumoxide browser handle
    browser: Option<chromiumoxide::Browser>,
}

impl Browser {
    /// Launch a new headless Chromium browser instance.
    /// 
    /// This will:
    /// 1. Find a Chromium binary using [`ChromiumLocator`]
    /// 2. Spawn the browser with headless flags and CDP enabled
    /// 3. Connect via the Chrome DevTools Protocol
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// use pagelens_core::browser::Browser;
    /// 
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let browser = Browser::launch().await?;
    /// // Use browser...
    /// browser.shutdown().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn launch() -> Result<Self> {
        let locator = ChromiumLocator::new();
        let binary_path = locator.find()?;

        // Create a unique temp directory for this browser instance's user data
        // This prevents "SingletonLock" conflicts when running multiple browsers
        let user_data_dir = tempfile::tempdir()
            .map_err(|e| Error::BrowserLaunchFailed(format!("Failed to create temp dir: {e}")))?;
        let user_data_path = user_data_dir.path().to_path_buf();
        
        // Keep the temp dir alive by leaking it (it will be cleaned up on process exit)
        // In production, we'd want to properly clean this up on browser shutdown
        std::mem::forget(user_data_dir);

        // Configure the browser
        let config = chromiumoxide::browser::BrowserConfig::builder()
            .chrome_executable(binary_path)
            .user_data_dir(user_data_path)
            .arg("--headless")
            .arg("--no-sandbox")
            .arg("--disable-gpu")
            .arg("--disable-dev-shm-usage")
            .arg("--disable-setuid-sandbox")
            .arg("--no-first-run")
            .arg("--no-zygote")
            .arg("--disable-blink-features=AutomationControlled")
            .build()
            .map_err(|e| Error::BrowserLaunchFailed(e.to_string()))?;

        // Launch the browser and get both the browser handle and handler
        let (browser, mut handler) = chromiumoxide::Browser::launch(config)
            .await
            .map_err(|e| Error::BrowserLaunchFailed(e.to_string()))?;

        // Spawn the handler in a background task to drive the WebSocket connection
        tokio::spawn(async move {
            use futures::StreamExt;
            while let Some(_) = handler.next().await {}
        });

        Ok(Self {
            browser: Some(browser),
        })
    }

    /// Check if the browser is connected and running.
    pub fn is_connected(&self) -> bool {
        self.browser.is_some()
    }

    /// Get a reference to the underlying chromiumoxide browser.
    fn get_browser(&self) -> Result<&chromiumoxide::Browser> {
        self.browser.as_ref()
            .ok_or_else(|| Error::BrowserConnectionFailed("Browser not connected".to_string()))
    }

    /// Navigate to a URL and return a Page handle.
    /// 
    /// # Arguments
    /// 
    /// * `url` - The URL to navigate to
    /// 
    /// # Errors
    /// 
    /// Returns an error if navigation fails or the URL is invalid.
    pub async fn navigate(&self, url: &str) -> Result<Page> {
        // Validate URL
        if !url.starts_with("http://") 
            && !url.starts_with("https://") 
            && !url.starts_with("data:") 
            && !url.starts_with("file://") {
            return Err(Error::InvalidUrl(url.to_string()));
        }

        let browser = self.get_browser()?;

        // Create a new page and navigate to the URL
        let cdp_page = browser.new_page(url).await
            .map_err(|e| Error::NavigationFailed(e.to_string()))?;

        // Wait a moment for the page to settle
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;

        Ok(Page {
            cdp_page: Some(cdp_page),
            url: url.to_string(),
        })
    }

    /// Shut down the browser gracefully.
    /// 
    /// This closes all pages and the browser connection.
    /// It's safe to call multiple times.
    pub async fn shutdown(mut self) -> Result<()> {
        self.shutdown_internal().await
    }

    /// Internal shutdown method that doesn't consume self.
    async fn shutdown_internal(&mut self) -> Result<()> {
        if let Some(mut browser) = self.browser.take() {
            let _ = browser.close().await;
        }
        Ok(())
    }
}

impl Drop for Browser {
    fn drop(&mut self) {
        // The browser will be closed when the handle is dropped
        let _ = self.browser.take();
    }
}

/// A handle to a specific page/tab in the browser.
/// 
/// Used to interact with a loaded page and extract data from it.
pub struct Page {
    /// The chromiumoxide page handle (crate-visible for snapshot access)
    pub(crate) cdp_page: Option<chromiumoxide::Page>,
    /// The URL of the page
    url: String,
}

impl Page {
    /// Get the URL of this page.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Get the full HTML content of the page.
    /// 
    /// Returns the rendered DOM as a string.
    pub async fn html(&self) -> Result<String> {
        if let Some(cdp_page) = &self.cdp_page {
            // Use the DOM API to get the documentElement's outerHTML
            let html = cdp_page.evaluate("document.documentElement.outerHTML").await
                .map_err(|e| Error::ExtractionFailed(e.to_string()))?
                .into_value::<String>()
                .map_err(|e| Error::ExtractionFailed(format!("Failed to extract HTML: {:?}", e)))?;
            Ok(html)
        } else if self.url.starts_with("data:") {
            // Extract HTML from data URL (fallback)
            if let Some(comma_pos) = self.url.find(',') {
                let encoded = &self.url[comma_pos + 1..];
                return Ok(url_decode(encoded));
            }
            Ok(String::new())
        } else {
            Err(Error::ExtractionFailed("Page not available".to_string()))
        }
    }

    /// Get the title of the page.
    pub async fn title(&self) -> Result<String> {
        if let Some(cdp_page) = &self.cdp_page {
            let title = cdp_page.get_title().await
                .map_err(|e| Error::ExtractionFailed(e.to_string()))?;
            Ok(title.unwrap_or_default())
        } else {
            Ok(String::new())
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_decode() {
        assert_eq!(url_decode("hello%20world"), "hello world");
        assert_eq!(url_decode("foo+bar"), "foo bar");
        assert_eq!(url_decode("test%3C%3E"), "test<>");
    }
}
