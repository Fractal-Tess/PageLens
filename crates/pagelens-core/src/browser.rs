//! Browser management — Chromium download, launch, and lifecycle.

use crate::prelude::*;
use pagelens_logging::{debug, error, info, warn};
use chromiumoxide::cdp::browser_protocol::network::{
    EnableParams, EventLoadingFailed, EventLoadingFinished, EventRequestWillBeSent,
    EventResponseReceived, Headers, SetExtraHttpHeadersParams,
};
use chromiumoxide::{Browser as ChromeBrowser, BrowserConfig, Page as ChromePage};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::task::JoinHandle;

fn sort_by_start_time(records: &mut [crate::snapshot::NetworkRequestRecord]) {
    records.sort_by(|a, b| {
        a.request_start_time_s
            .partial_cmp(&b.request_start_time_s)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

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
    /// The chrome browser handle
    browser: Option<ChromeBrowser>,
    /// User data directory temp folder - stored to prevent premature cleanup
    _user_data_dir: Option<tempfile::TempDir>,
    handler_task: Option<JoinHandle<()>>,
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
        info!(binary = %binary_path.display(), "Launching Chromium browser");

        // Create a unique temp directory for this browser instance's user data
        // This prevents "SingletonLock" conflicts when running multiple browsers
        let user_data_dir = tempfile::tempdir()
            .map_err(|e| Error::BrowserLaunchFailed(format!("Failed to create temp dir: {e}")))?;
        let user_data_path = user_data_dir.path().to_path_buf();

        // Configure the browser with minimal flags
        // --disable-dev-shm-usage is needed for containerized environments
        // --disable-setuid-sandbox helps with permission issues
        // --remote-debugging-port=0 lets Chrome pick an available port
        let config = BrowserConfig::builder()
            .chrome_executable(binary_path)
            .user_data_dir(user_data_path)
            .arg("--headless=new")
            .arg("--no-sandbox")
            .arg("--disable-gpu")
            .arg("--disable-dev-shm-usage")
            .arg("--disable-setuid-sandbox")
            .arg("--disable-web-security")
            .arg("--disable-features=IsolateOrigins,site-per-process")
            .build()
            .map_err(|e| Error::BrowserLaunchFailed(e.to_string()))?;

        // Launch the browser and get both the browser handle and handler
        let (browser, mut handler) = ChromeBrowser::launch(config)
            .await
            .map_err(|e| Error::BrowserLaunchFailed(e.to_string()))?;

        // Spawn the handler in a background task to drive the WebSocket connection
        // This must be polled continuously for the browser to work
        let handler_task = tokio::spawn(async move {
            use futures::StreamExt;
            loop {
                match handler.next().await {
                    Some(Ok(msg)) => {
                        // Process the message normally
                        let _ = msg;
                    }
                    Some(Err(e)) => {
                        error!(error = %e, "Browser handler error");
                        break;
                    }
                    None => {
                        warn!("Browser handler stream ended");
                        break;
                    }
                }
            }
        });

        Ok(Self {
            browser: Some(browser),
            _user_data_dir: Some(user_data_dir),
            handler_task: Some(handler_task),
        })
    }

    /// Check if the browser is connected and running.
    pub fn is_connected(&self) -> bool {
        self.browser.is_some()
    }

    /// Get a reference to the underlying chrome browser.
    fn get_browser(&self) -> Result<&ChromeBrowser> {
        self.browser
            .as_ref()
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
        self.navigate_with_headers(url, &HashMap::new()).await
    }

    pub async fn navigate_with_headers(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
    ) -> Result<Page> {
        // Validate URL
        if !url.starts_with("http://")
            && !url.starts_with("https://")
            && !url.starts_with("data:")
            && !url.starts_with("file://")
        {
            return Err(Error::InvalidUrl(url.to_string()));
        }

        debug!(url = %url, has_headers = !headers.is_empty(), "Navigating page");

        let browser = self.get_browser()?;
        let collected_network_requests: Arc<
            tokio::sync::Mutex<Vec<crate::snapshot::NetworkRequestRecord>>,
        > = Arc::new(tokio::sync::Mutex::new(Vec::new()));

        let cdp_page = browser
            .new_page("about:blank")
            .await
            .map_err(|e| Error::NavigationFailed(e.to_string()))?;

        if !headers.is_empty() {
            let headers = serde_json::to_value(headers)
                .map_err(|e| Error::NavigationFailed(format!("Invalid headers: {e}")))?;
            cdp_page
                .set_extra_headers(SetExtraHttpHeadersParams::new(Headers::new(headers)))
                .await
                .map_err(|e| Error::NavigationFailed(format!("Failed setting headers: {e}")))?;
        }

        if !url.starts_with("data:") {
            let _ = cdp_page.execute(EnableParams::default()).await;

            if let (
                Ok(mut request_stream),
                Ok(mut response_stream),
                Ok(mut finished_stream),
                Ok(mut failed_stream),
            ) = (
                cdp_page.event_listener::<EventRequestWillBeSent>().await,
                cdp_page.event_listener::<EventResponseReceived>().await,
                cdp_page.event_listener::<EventLoadingFinished>().await,
                cdp_page.event_listener::<EventLoadingFailed>().await,
            ) {
                let collected = Arc::clone(&collected_network_requests);
                tokio::spawn(async move {
                    use futures::StreamExt;

                    let mut by_request: HashMap<String, crate::snapshot::NetworkRequestRecord> =
                        HashMap::new();

                    loop {
                        tokio::select! {
                            req = request_stream.next() => {
                                let Some(req) = req else { break; };
                                let key = format!("{:?}", req.request_id);
                                let mut record = by_request.remove(&key).unwrap_or_default();
                                record.url = req.request.url.clone();
                                record.resource_type = req.r#type.as_ref().map(|t| format!("{:?}", t));
                                record.was_redirect = Some(req.redirect_response.is_some());
                                if let Some(redirect_response) = &req.redirect_response {
                                    record.redirect_from_url = Some(redirect_response.url.clone());
                                    record.redirect_status_code = Some(redirect_response.status as u16);
                                }
                                record.request_start_time_s = Some(*req.timestamp.inner());
                                by_request.insert(key, record);

                                let mut out = collected.lock().await;
                                let mut values: Vec<_> = by_request.values().cloned().collect();
                                sort_by_start_time(&mut values);
                                *out = values;
                            }
                            resp = response_stream.next() => {
                                let Some(resp) = resp else { break; };
                                let key = format!("{:?}", resp.request_id);
                                let mut record = by_request.remove(&key).unwrap_or_default();
                                record.url = resp.response.url.clone();
                                record.response_start_time_s = Some(*resp.timestamp.inner());
                                record.status_code = Some(resp.response.status as u16);
                                record.encoded_data_length = Some(resp.response.encoded_data_length);
                                record.from_cache = Some(
                                    resp.response.from_disk_cache.unwrap_or(false)
                                        || resp.response.from_prefetch_cache.unwrap_or(false),
                                );
                                record.mime_type = Some(resp.response.mime_type.clone());
                                record.content_encoding = resp
                                    .response
                                    .headers
                                    .0
                                    .get("content-encoding")
                                    .map(|value| {
                                        value
                                            .as_str()
                                            .map(std::string::ToString::to_string)
                                            .unwrap_or_else(|| value.to_string())
                                    });
                                by_request.insert(key, record);

                                let mut out = collected.lock().await;
                                let mut values: Vec<_> = by_request.values().cloned().collect();
                                sort_by_start_time(&mut values);
                                *out = values;
                            }
                            finished = finished_stream.next() => {
                                let Some(finished) = finished else { break; };
                                let key = format!("{:?}", finished.request_id);
                                let mut record = by_request.remove(&key).unwrap_or_default();
                                record.end_time_s = Some(*finished.timestamp.inner());
                                record.encoded_data_length = Some(finished.encoded_data_length);
                                record.failed = Some(false);

                                if let (Some(start), Some(end)) = (record.request_start_time_s, record.end_time_s) {
                                    if end >= start {
                                        record.duration_ms = Some((end - start) * 1000.0);
                                    }
                                }

                                by_request.insert(key, record);

                                let mut out = collected.lock().await;
                                let mut values: Vec<_> = by_request.values().cloned().collect();
                                sort_by_start_time(&mut values);
                                *out = values;
                            }
                            failed = failed_stream.next() => {
                                let Some(failed) = failed else { break; };
                                let key = format!("{:?}", failed.request_id);
                                let mut record = by_request.remove(&key).unwrap_or_default();
                                record.end_time_s = Some(*failed.timestamp.inner());
                                record.failed = Some(true);
                                record.failure_text = Some(failed.error_text.clone());

                                if let (Some(start), Some(end)) = (record.request_start_time_s, record.end_time_s) {
                                    if end >= start {
                                        record.duration_ms = Some((end - start) * 1000.0);
                                    }
                                }

                                by_request.insert(key, record);

                                let mut out = collected.lock().await;
                                let mut values: Vec<_> = by_request.values().cloned().collect();
                                sort_by_start_time(&mut values);
                                *out = values;
                            }
                        }
                    }

                    let mut out = collected.lock().await;
                    let mut values: Vec<_> = by_request.into_values().collect();
                    sort_by_start_time(&mut values);
                    *out = values;
                });
            }
        }

        cdp_page
            .goto(url)
            .await
            .map_err(|e| Error::NavigationFailed(e.to_string()))?;

        // Wait for navigation to complete
        if !url.starts_with("data:") {
            let _ = cdp_page.wait_for_navigation().await;
        } else {
            // For data URLs, wait for the content to be available
            // Data URLs load synchronously but we need to ensure the DOM is ready
            let mut attempts = 0;
            while attempts < 10 {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                if let Ok(Some(_)) = cdp_page.get_title().await {
                    // Title is available, page is likely ready
                    break;
                }
                attempts += 1;
            }
        }

        // Check if the browser ended up on a Chrome error page (e.g. host unreachable)
        if let Ok(Some(final_url)) = cdp_page.url().await {
            if final_url.starts_with("chrome-error://") {
                return Err(Error::NavigationFailed(format!(
                    "Failed to load {}: page unreachable",
                    url
                )));
            }
        }

        Ok(Page {
            cdp_page: Some(cdp_page),
            url: url.to_string(),
            network_requests: collected_network_requests,
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
        debug!("Shutting down browser instance");
        if let Some(browser) = self.browser.take() {
            let _ = browser.close().await;
        }

        if let Some(handler_task) = self.handler_task.take() {
            handler_task.abort();
            let _ = handler_task.await;
        }

        Ok(())
    }
}

impl Drop for Browser {
    fn drop(&mut self) {
        // The browser will be closed when the handle is dropped
        let _ = self.browser.take();
        if let Some(handler_task) = self.handler_task.take() {
            handler_task.abort();
        }
    }
}

/// A handle to a specific page/tab in the browser.
///
/// Used to interact with a loaded page and extract data from it.
pub struct Page {
    /// The chrome page handle (crate-visible for snapshot access)
    pub(crate) cdp_page: Option<ChromePage>,
    /// The URL of the page
    url: String,
    pub(crate) network_requests:
        Arc<tokio::sync::Mutex<Vec<crate::snapshot::NetworkRequestRecord>>>,
}

impl Page {
    /// Get the URL of this page.
    pub fn url(&self) -> &str {
        &self.url
    }

    pub async fn network_requests(&self) -> Vec<crate::snapshot::NetworkRequestRecord> {
        self.network_requests.lock().await.clone()
    }

    /// Get the full HTML content of the page.
    ///
    /// Returns the rendered DOM as a string.
    pub async fn html(&self) -> Result<String> {
        if let Some(cdp_page) = &self.cdp_page {
            // Use the DOM API to get the documentElement's outerHTML
            // Retry a few times if we get empty content (page might still be loading)
            let mut attempts = 0;
            while attempts < 5 {
                match cdp_page
                    .evaluate("document.documentElement.outerHTML")
                    .await
                {
                    Ok(result) => match result.into_value::<String>() {
                        Ok(html) if !html.is_empty() => return Ok(html),
                        _ => {}
                    },
                    Err(_) => {}
                }
                attempts += 1;
                if attempts < 5 {
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
            }

            // If all attempts failed, return empty string
            Ok(String::new())
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
            // Try JavaScript first as it's more reliable
            match cdp_page.evaluate("document.title").await {
                Ok(result) => {
                    if let Ok(title) = result.into_value::<String>() {
                        if !title.is_empty() {
                            return Ok(title);
                        }
                    }
                }
                Err(_) => {}
            }

            // Fallback to CDP get_title
            let title = cdp_page
                .get_title()
                .await
                .map_err(|e| Error::ExtractionFailed(e.to_string()))?;
            Ok(title.unwrap_or_default())
        } else {
            Ok(String::new())
        }
    }
}

/// Simple URL decoding for data URLs.
pub fn url_decode(input: &str) -> String {
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
