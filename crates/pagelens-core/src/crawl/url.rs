//! URL resolution and link extraction

use scraper::{Html, Selector};
use std::collections::HashSet;
use std::sync::LazyLock;

// Pre-compiled CSS selector for link extraction
static LINK_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("a[href]").expect("valid link selector"));

/// Extract all links from HTML using CSS selectors.
pub fn extract_links(html: &str, base_url: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut links = Vec::new();

    // Parse the HTML and select all <a> tags with href attributes
    let document = Html::parse_fragment(html);

    for element in document.select(&LINK_SELECTOR) {
        if let Some(href) = element.value().attr("href") {
            // Skip anchors and javascript
            if href.starts_with('#') || href.starts_with("javascript:") {
                continue;
            }

            // Resolve relative URLs
            let absolute_url = resolve_url(base_url, href);

            if let Some(url) = absolute_url {
                if seen.insert(url.clone()) {
                    links.push(url);
                }
            }
        }
    }

    links
}

/// Resolve a potentially relative URL to absolute.
pub fn resolve_url(base: &str, href: &str) -> Option<String> {
    // Already absolute
    if href.starts_with("http://") || href.starts_with("https://") {
        return Some(href.to_string());
    }

    // Data URLs - treat as absolute
    if href.starts_with("data:") {
        return Some(href.to_string());
    }

    // Protocol-relative
    if href.starts_with("//") {
        if let Some(pos) = base.find("://") {
            let protocol = &base[..pos];
            return Some(format!("{}:{}", protocol, href));
        }
        return None;
    }

    // For data URLs, we can't really resolve relative links meaningfully
    // Return as-is for testing purposes
    if base.starts_with("data:") {
        return Some(href.to_string());
    }

    // Parse base URL
    let base_parsed = match url::Url::parse(base) {
        Ok(u) => u,
        Err(_) => return None,
    };

    // Join with base
    match base_parsed.join(href) {
        Ok(u) => Some(u.to_string()),
        Err(_) => None,
    }
}

/// Handle data URLs by extracting content directly from the URL.
pub fn extract_html_from_data_url(url: &str) -> String {
    // Find the comma that separates the MIME type from the data
    if let Some(comma_pos) = url.find(',') {
        let data_part = &url[comma_pos + 1..];

        // For text/html data URLs, URL decode the content
        if url[..comma_pos].contains("text/html") {
            return url_decode(data_part);
        }

        // Return as-is for other types
        return data_part.to_string();
    }

    String::new()
}

/// Extract title from HTML content.
pub fn extract_title_from_html(html: &str) -> String {
    // Simple regex-like extraction for title tag
    if let Some(start) = html.to_lowercase().find("<title>") {
        let after_start = &html[start + 7..];
        if let Some(end) = after_start.to_lowercase().find("</title>") {
            return after_start[..end].trim().to_string();
        }
    }
    String::new()
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
