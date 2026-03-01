#[derive(thiserror::Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    /// Could not find a Chromium/Chrome binary on the system.
    #[error("Chromium binary not found. Set PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH or ensure chromium/chrome is in PATH")]
    ChromiumNotFound,

    /// Failed to launch the browser.
    #[error("Failed to launch browser: {0}")]
    BrowserLaunchFailed(String),

    /// Failed to connect to the browser's DevTools protocol.
    #[error("Failed to connect to browser DevTools: {0}")]
    BrowserConnectionFailed(String),

    /// Invalid URL provided.
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Navigation failed.
    #[error("Navigation failed: {0}")]
    NavigationFailed(String),

    /// Failed to extract data from the page.
    #[error("Failed to extract page data: {0}")]
    ExtractionFailed(String),
}
