//! Page snapshot capture — DOM, accessibility tree, performance timing, and styles.

use crate::browser::Page;
use crate::prelude::*;
use scraper::{Html, Selector};
use std::collections::{HashMap, HashSet};
use url::Url;

/// Options for capturing a page snapshot.
#[derive(Debug, Clone)]
pub struct SnapshotOptions {
    /// Include the full HTML content.
    pub include_html: bool,
    /// Include the accessibility tree.
    pub include_accessibility_tree: bool,
    /// Include performance timing metrics.
    pub include_performance_timing: bool,
    /// Include computed styles for elements.
    pub include_computed_styles: bool,
    pub include_network_metadata: bool,
}

impl Default for SnapshotOptions {
    fn default() -> Self {
        Self {
            include_html: true,
            include_accessibility_tree: true,
            include_performance_timing: true,
            include_computed_styles: false, // Expensive, disabled by default
            include_network_metadata: true,
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct MainResourceNetwork {
    pub final_url: Option<String>,
    pub status_code: Option<u16>,
    pub headers: HashMap<String, String>,
    pub fetch_error: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct NetworkRequestRecord {
    pub url: String,
    pub redirect_from_url: Option<String>,
    pub redirect_status_code: Option<u16>,
    pub resource_type: Option<String>,
    pub status_code: Option<u16>,
    pub encoded_data_length: Option<f64>,
    pub from_cache: Option<bool>,
    pub was_redirect: Option<bool>,
    pub failed: Option<bool>,
    pub failure_text: Option<String>,
    pub content_encoding: Option<String>,
    pub mime_type: Option<String>,
    pub request_start_time_s: Option<f64>,
    pub response_start_time_s: Option<f64>,
    pub end_time_s: Option<f64>,
    pub duration_ms: Option<f64>,
}

/// A single node in the accessibility tree.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AccessibilityNode {
    /// The role of this node (e.g., "main", "navigation", "heading")
    pub role: String,
    /// The accessible name/label for this node.
    pub name: Option<String>,
    /// The level if this is a heading (1-6).
    pub level: Option<u8>,
    /// Child nodes.
    pub children: Vec<AccessibilityNode>,
}

/// Performance timing metrics for a page navigation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceTiming {
    /// When navigation started (Unix timestamp in milliseconds).
    pub navigation_start: u64,
    /// When the DOM became interactive.
    pub dom_interactive: Option<u64>,
    /// When the DOM content was loaded.
    pub dom_content_loaded: Option<u64>,
    /// When the page was fully loaded.
    pub load_complete: Option<u64>,
    /// Time to first byte (TTFB) in milliseconds.
    pub response_start: Option<u64>,
    pub first_paint: Option<u64>,
    pub first_contentful_paint: Option<u64>,
    pub largest_contentful_paint: Option<u64>,
    pub cumulative_layout_shift: Option<f64>,
    pub interaction_to_next_paint: Option<u64>,
}

/// Computed style information for an element.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComputedStyle {
    /// CSS selector for the element.
    pub selector: String,
    /// Foreground color (e.g., "rgb(0, 0, 0)").
    pub color: Option<String>,
    /// Background color.
    pub background_color: Option<String>,
    /// Font size.
    pub font_size: Option<String>,
    /// Font weight.
    pub font_weight: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CaptureIssue {
    pub stage: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ReferencedAssets {
    pub javascript: Vec<String>,
    pub stylesheets: Vec<String>,
    pub media: Vec<String>,
    pub fonts: Vec<String>,
}

/// A comprehensive snapshot of a web page.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Snapshot {
    /// The URL of the page.
    pub url: String,
    /// The page title.
    pub title: String,
    /// The full HTML content (if requested).
    pub html: String,
    /// The accessibility tree (if requested).
    pub accessibility_tree: Vec<AccessibilityNode>,
    /// Performance timing metrics (if requested).
    pub performance_timing: PerformanceTiming,
    /// Computed styles for visible elements (if requested).
    pub computed_styles: Vec<ComputedStyle>,
    pub referenced_assets: ReferencedAssets,
    pub favicon_url: Option<String>,
    pub main_resource_network: MainResourceNetwork,
    pub network_requests: Vec<NetworkRequestRecord>,
    pub capture_issues: Vec<CaptureIssue>,
}

impl Snapshot {
    /// Create a new empty snapshot.
    pub fn new(url: String, title: String) -> Self {
        Self {
            url,
            title,
            html: String::new(),
            accessibility_tree: Vec::new(),
            performance_timing: PerformanceTiming {
                navigation_start: 0,
                dom_interactive: None,
                dom_content_loaded: None,
                load_complete: None,
                response_start: None,
                first_paint: None,
                first_contentful_paint: None,
                largest_contentful_paint: None,
                cumulative_layout_shift: None,
                interaction_to_next_paint: None,
            },
            computed_styles: Vec::new(),
            referenced_assets: ReferencedAssets::default(),
            favicon_url: None,
            main_resource_network: MainResourceNetwork::default(),
            network_requests: Vec::new(),
            capture_issues: Vec::new(),
        }
    }
}

/// Extension trait for Page to add snapshot functionality.
pub trait SnapshotExt {
    /// Capture a snapshot of the current page state.
    fn snapshot(
        &self,
        options: SnapshotOptions,
    ) -> impl std::future::Future<Output = Result<Snapshot>> + Send;
}

impl SnapshotExt for Page {
    async fn snapshot(&self, options: SnapshotOptions) -> Result<Snapshot> {
        let url = self.url().to_string();
        let title = self.title().await?;

        let mut snapshot = Snapshot::new(url, title);

        // Capture HTML if requested
        if options.include_html {
            snapshot.html = self.html().await?;
            let (assets, favicon_url) = extract_referenced_assets(&snapshot.html, &snapshot.url);
            snapshot.referenced_assets = assets;
            snapshot.favicon_url = favicon_url;
        }

        // Capture accessibility tree if requested
        if options.include_accessibility_tree {
            match capture_accessibility_tree(self).await {
                Ok(tree) => snapshot.accessibility_tree = tree,
                Err(err) => snapshot.capture_issues.push(CaptureIssue {
                    stage: "accessibility_tree".to_string(),
                    message: err.to_string(),
                }),
            }
        }

        // Capture performance timing if requested
        if options.include_performance_timing {
            match capture_performance_timing(self).await {
                Ok(timing) => snapshot.performance_timing = timing,
                Err(err) => snapshot.capture_issues.push(CaptureIssue {
                    stage: "performance_timing".to_string(),
                    message: err.to_string(),
                }),
            }
        }

        // Capture computed styles if requested
        if options.include_computed_styles {
            match capture_computed_styles(self).await {
                Ok(styles) => snapshot.computed_styles = styles,
                Err(err) => snapshot.capture_issues.push(CaptureIssue {
                    stage: "computed_styles".to_string(),
                    message: err.to_string(),
                }),
            }
        }

        if options.include_network_metadata {
            match capture_main_resource_network(self).await {
                Ok(network) => snapshot.main_resource_network = network,
                Err(err) => snapshot.capture_issues.push(CaptureIssue {
                    stage: "network_metadata".to_string(),
                    message: err.to_string(),
                }),
            }

            snapshot.network_requests = self.network_requests().await;
        }

        Ok(snapshot)
    }
}

async fn capture_main_resource_network(page: &Page) -> Result<MainResourceNetwork> {
    let js = r#"
        (async function() {
            try {
                const response = await fetch(window.location.href, {
                    method: 'GET',
                    cache: 'no-store',
                    credentials: 'same-origin',
                    redirect: 'follow'
                });

                const wanted = [
                    'cache-control',
                    'content-encoding',
                    'content-type',
                    'content-security-policy',
                    'strict-transport-security',
                    'x-content-type-options',
                    'x-frame-options',
                    'referrer-policy',
                    'permissions-policy',
                    'x-robots-tag',
                    'cross-origin-opener-policy',
                    'cross-origin-embedder-policy',
                    'cross-origin-resource-policy'
                ];

                const headers = {};
                for (const name of wanted) {
                    const value = response.headers.get(name);
                    if (value !== null) {
                        headers[name] = value;
                    }
                }

                return JSON.stringify({
                    final_url: response.url || window.location.href,
                    status_code: response.status,
                    headers,
                    fetch_error: null,
                });
            } catch (err) {
                return JSON.stringify({
                    final_url: window.location.href,
                    status_code: null,
                    headers: {},
                    fetch_error: String(err),
                });
            }
        })()
    "#;

    let cdp_page = page
        .cdp_page()
        .ok_or_else(|| Error::ExtractionFailed("CDP page unavailable".to_string()))?;

    let result = cdp_page
        .evaluate(js)
        .await
        .map_err(|e| Error::ExtractionFailed(format!("Network metadata evaluation failed: {e}")))?;

    let json_str = result
        .into_value::<String>()
        .map_err(|e| Error::ExtractionFailed(format!("Network metadata decode failed: {e}")))?;

    serde_json::from_str::<MainResourceNetwork>(&json_str)
        .map_err(|e| Error::ExtractionFailed(format!("Network metadata JSON parse failed: {e}")))
}

/// Extract referenced assets from HTML. Returns `(assets, favicon_url)`.
fn extract_referenced_assets(html: &str, page_url: &str) -> (ReferencedAssets, Option<String>) {
    let document = Html::parse_document(html);
    let base = Url::parse(page_url).ok();

    let mut javascript = Vec::new();
    let mut js_seen = HashSet::new();

    if let Ok(script_selector) = Selector::parse("script[src]") {
        for el in document.select(&script_selector) {
            if let Some(src) = el.value().attr("src") {
                push_asset(&mut javascript, &mut js_seen, src, base.as_ref());
            }
        }
    }

    if let Ok(modulepreload_selector) = Selector::parse(r#"link[rel="modulepreload"][href]"#) {
        for el in document.select(&modulepreload_selector) {
            if let Some(href) = el.value().attr("href") {
                push_asset(&mut javascript, &mut js_seen, href, base.as_ref());
            }
        }
    }

    let mut stylesheets = Vec::new();
    let mut css_seen = HashSet::new();

    if let Ok(stylesheet_selector) = Selector::parse(r#"link[rel="stylesheet"][href]"#) {
        for el in document.select(&stylesheet_selector) {
            if let Some(href) = el.value().attr("href") {
                push_asset(&mut stylesheets, &mut css_seen, href, base.as_ref());
            }
        }
    }

    let mut media = Vec::new();
    let mut media_seen = HashSet::new();
    let media_selectors = [
        "img[src]",
        "img[srcset]",
        "source[src]",
        "source[srcset]",
        "video[src]",
        "audio[src]",
    ];

    for pattern in media_selectors {
        if let Ok(selector) = Selector::parse(pattern) {
            for el in document.select(&selector) {
                if let Some(src) = el.value().attr("src") {
                    push_asset(&mut media, &mut media_seen, src, base.as_ref());
                }
                if let Some(srcset) = el.value().attr("srcset") {
                    for candidate in parse_srcset_urls(srcset) {
                        push_asset(&mut media, &mut media_seen, &candidate, base.as_ref());
                    }
                }
            }
        }
    }

    // Extract font preloads
    let mut fonts = Vec::new();
    let mut fonts_seen = HashSet::new();
    if let Ok(font_selector) = Selector::parse(r#"link[rel="preload"][as="font"][href]"#) {
        for el in document.select(&font_selector) {
            if let Some(href) = el.value().attr("href") {
                push_asset(&mut fonts, &mut fonts_seen, href, base.as_ref());
            }
        }
    }

    // Extract favicon: prefer icon, then shortcut icon, then apple-touch-icon
    let favicon_url = extract_favicon(&document, base.as_ref());

    (
        ReferencedAssets {
            javascript,
            stylesheets,
            media,
            fonts,
        },
        favicon_url,
    )
}

fn extract_favicon(document: &Html, base: Option<&Url>) -> Option<String> {
    let candidates = [
        r#"link[rel="icon"][href]"#,
        r#"link[rel="shortcut icon"][href]"#,
        r#"link[rel="apple-touch-icon"][href]"#,
    ];
    for pattern in candidates {
        if let Ok(selector) = Selector::parse(pattern) {
            if let Some(el) = document.select(&selector).next() {
                if let Some(href) = el.value().attr("href") {
                    let normalized = normalize_asset_url(href, base);
                    if !normalized.is_empty() {
                        return Some(normalized);
                    }
                }
            }
        }
    }
    None
}

fn parse_srcset_urls(srcset: &str) -> Vec<String> {
    srcset
        .split(',')
        .filter_map(|entry| entry.split_whitespace().next())
        .filter(|candidate| !candidate.is_empty())
        .map(std::string::ToString::to_string)
        .collect()
}

fn push_asset(
    collection: &mut Vec<String>,
    seen: &mut HashSet<String>,
    raw: &str,
    base: Option<&Url>,
) {
    if raw.is_empty() || raw.starts_with("data:") || raw.starts_with("javascript:") {
        return;
    }

    let normalized = normalize_asset_url(raw, base);
    if seen.insert(normalized.clone()) {
        collection.push(normalized);
    }
}

fn normalize_asset_url(raw: &str, base: Option<&Url>) -> String {
    if let Ok(parsed) = Url::parse(raw) {
        return parsed.to_string();
    }

    if let Some(base_url) = base {
        if let Ok(joined) = base_url.join(raw) {
            return joined.to_string();
        }
    }

    raw.to_string()
}

/// Capture the accessibility tree from the page.
async fn capture_accessibility_tree(page: &Page) -> Result<Vec<AccessibilityNode>> {
    // For now, return a simple stub implementation
    // In the full implementation, this would use CDP's Accessibility domain

    // Try to get accessibility tree via JavaScript evaluation
    let js = r#"
        (function() {
            function getAccessibleTree(element, depth = 0) {
                if (depth > 10) return null; // Limit depth
                
                const nodes = [];
                const children = element.children;
                
                for (let i = 0; i < children.length; i++) {
                    const child = children[i];
                    const role = child.getAttribute('role') || getImplicitRole(child);
                    
                    if (role) {
                        const node = {
                            role: role,
                            name: child.getAttribute('aria-label') || 
                                  child.getAttribute('aria-labelledby') ||
                                  child.textContent?.substring(0, 100) || null,
                            level: null,
                            children: []
                        };
                        
                        if (role === 'heading') {
                            const tag = child.tagName;
                            if (tag && tag.length === 2 && tag[0] === 'H') {
                                node.level = parseInt(tag[1]) || null;
                            }
                        }
                        
                        const childNodes = getAccessibleTree(child, depth + 1);
                        if (childNodes && childNodes.length > 0) {
                            node.children = childNodes;
                        }
                        
                        nodes.push(node);
                    } else {
                        // Still recurse into elements without roles
                        const childNodes = getAccessibleTree(child, depth + 1);
                        if (childNodes) {
                            nodes.push(...childNodes);
                        }
                    }
                }
                
                return nodes;
            }
            
            function getImplicitRole(element) {
                const tag = element.tagName?.toLowerCase();
                if (!tag) return null;
                
                switch (tag) {
                    case 'main': return 'main';
                    case 'nav': return 'navigation';
                    case 'header': return 'banner';
                    case 'footer': return 'contentinfo';
                    case 'aside': return 'complementary';
                    case 'section': return 'region';
                    case 'article': return 'article';
                    case 'h1': case 'h2': case 'h3': case 'h4': case 'h5': case 'h6':
                        return 'heading';
                    case 'a': return element.href ? 'link' : null;
                    case 'button': return 'button';
                    case 'input': 
                        const type = element.type;
                        if (type === 'submit' || type === 'button') return 'button';
                        return 'textbox';
                    case 'img': return 'img';
                    case 'form': return 'form';
                    case 'ul': case 'ol': return 'list';
                    case 'li': return 'listitem';
                    default: return null;
                }
            }
            
            return JSON.stringify(getAccessibleTree(document.body));
        })()
    "#;

    let cdp_page = page
        .cdp_page()
        .ok_or_else(|| Error::ExtractionFailed("CDP page unavailable".to_string()))?;

    let result = cdp_page
        .evaluate(js)
        .await
        .map_err(|e| Error::ExtractionFailed(format!("Accessibility evaluation failed: {e}")))?;

    let json_str = result
        .into_value::<String>()
        .map_err(|e| Error::ExtractionFailed(format!("Accessibility result decode failed: {e}")))?;

    serde_json::from_str::<Vec<AccessibilityNode>>(&json_str)
        .map_err(|e| Error::ExtractionFailed(format!("Accessibility JSON parse failed: {e}")))
}

/// Capture performance timing metrics.
async fn capture_performance_timing(page: &Page) -> Result<PerformanceTiming> {
    let js = r#"
        (async function() {
            const origin = performance.timeOrigin || 0;
            const navEntry = performance.getEntriesByType('navigation')[0] || null;

            const supported =
                typeof PerformanceObserver !== 'undefined'
                    ? PerformanceObserver.supportedEntryTypes || []
                    : [];

            let lcpStart = null;
            let clsValue = 0;
            const interactionById = new Map();

            const observers = [];
            function addObserver(type, handler, options = {}) {
                if (!supported.includes(type)) return;
                try {
                    const observer = new PerformanceObserver((list) => {
                        for (const entry of list.getEntries()) {
                            handler(entry);
                        }
                    });
                    observer.observe({ type, buffered: true, ...options });
                    observers.push(observer);
                } catch (_) {
                    // Ignore unsupported observer settings
                }
            }

            addObserver('largest-contentful-paint', (entry) => {
                if (typeof entry.startTime === 'number' && Number.isFinite(entry.startTime)) {
                    lcpStart = entry.startTime;
                }
            });

            addObserver('layout-shift', (entry) => {
                const value = typeof entry.value === 'number' && Number.isFinite(entry.value)
                    ? entry.value
                    : 0;
                if (!entry.hadRecentInput) {
                    clsValue += value;
                }
            });

            addObserver(
                'event',
                (entry) => {
                    const interactionId = typeof entry.interactionId === 'number'
                        ? entry.interactionId
                        : 0;
                    const duration = typeof entry.duration === 'number' && Number.isFinite(entry.duration)
                        ? entry.duration
                        : 0;
                    if (interactionId > 0 && duration >= 0) {
                        const previous = interactionById.get(interactionId) ?? 0;
                        if (duration > previous) {
                            interactionById.set(interactionId, duration);
                        }
                    }
                },
                { durationThreshold: 16 }
            );

            // first-input can expose the first interaction even when event timing
            // thresholds/filtering would otherwise miss it.
            addObserver('first-input', (entry) => {
                const duration = typeof entry.duration === 'number' && Number.isFinite(entry.duration)
                    ? entry.duration
                    : 0;
                if (duration >= 0) {
                    const key = -1;
                    const previous = interactionById.get(key) ?? 0;
                    if (duration > previous) {
                        interactionById.set(key, duration);
                    }
                }
            });

            // Give the browser one more turn after load to flush buffered entries.
            await new Promise((resolve) => setTimeout(resolve, 250));
            for (const observer of observers) {
                observer.disconnect();
            }

            const paintEntries = performance.getEntriesByType('paint') || [];
            const firstPaint = paintEntries.find((e) => e.name === 'first-paint') || null;
            const firstContentfulPaint =
                paintEntries.find((e) => e.name === 'first-contentful-paint') || null;

            if (lcpStart == null) {
                const lcpEntries = performance.getEntriesByType('largest-contentful-paint') || [];
                const latestLcp = lcpEntries.length ? lcpEntries[lcpEntries.length - 1] : null;
                if (latestLcp && typeof latestLcp.startTime === 'number') {
                    lcpStart = latestLcp.startTime;
                }
            }

            // If the observer did not run, fallback to direct collection.
            if (clsValue === 0) {
                const clsEntries = performance.getEntriesByType('layout-shift') || [];
                for (const entry of clsEntries) {
                    if (!entry?.hadRecentInput && typeof entry.value === 'number' && Number.isFinite(entry.value)) {
                        clsValue += entry.value;
                    }
                }
            }

            // INP approximation in lab: p98 of max interaction durations by interactionId.
            // If there are no interactions, this stays null (same as Lighthouse's N/A behavior).
            let inpCandidate = null;
            const interactionDurations = Array.from(interactionById.values())
                .filter((value) => Number.isFinite(value) && value >= 0)
                .sort((a, b) => a - b);
            if (interactionDurations.length > 0) {
                const index = Math.max(0, Math.ceil(interactionDurations.length * 0.98) - 1);
                inpCandidate = interactionDurations[index];
            }

            if (navEntry) {
                return JSON.stringify({
                    navigation_start: Math.round(origin + navEntry.startTime),
                    dom_interactive: navEntry.domInteractive
                        ? Math.round(origin + navEntry.domInteractive)
                        : null,
                    dom_content_loaded: navEntry.domContentLoadedEventEnd
                        ? Math.round(origin + navEntry.domContentLoadedEventEnd)
                        : null,
                    load_complete: navEntry.loadEventEnd ? Math.round(origin + navEntry.loadEventEnd) : null,
                    response_start: navEntry.responseStart ? Math.round(origin + navEntry.responseStart) : null,
                    first_paint: firstPaint ? Math.round(origin + firstPaint.startTime) : null,
                    first_contentful_paint: firstContentfulPaint
                        ? Math.round(origin + firstContentfulPaint.startTime)
                        : null,
                    largest_contentful_paint: lcpStart != null ? Math.round(origin + lcpStart) : null,
                    cumulative_layout_shift: Number.isFinite(clsValue) ? Number(clsValue.toFixed(4)) : null,
                    interaction_to_next_paint: inpCandidate != null ? Math.round(inpCandidate) : null
                });
            }

            // Fallback to legacy performance.timing when navigation entry is unavailable.
            const timing = performance.timing;
            return JSON.stringify({
                navigation_start: timing.navigationStart,
                dom_interactive: timing.domInteractive || null,
                dom_content_loaded: timing.domContentLoadedEventEnd || null,
                load_complete: timing.loadEventEnd || null,
                response_start: timing.responseStart || null,
                first_paint: firstPaint ? Math.round(origin + firstPaint.startTime) : null,
                first_contentful_paint: firstContentfulPaint ? Math.round(origin + firstContentfulPaint.startTime) : null,
                largest_contentful_paint: lcpStart != null ? Math.round(origin + lcpStart) : null,
                cumulative_layout_shift: Number.isFinite(clsValue) ? Number(clsValue.toFixed(4)) : null,
                interaction_to_next_paint: inpCandidate != null ? Math.round(inpCandidate) : null
            });
        })()
    "#;

    let cdp_page = page
        .cdp_page()
        .ok_or_else(|| Error::ExtractionFailed("CDP page unavailable".to_string()))?;

    let result = cdp_page
        .evaluate(js)
        .await
        .map_err(|e| Error::ExtractionFailed(format!("Performance evaluation failed: {e}")))?;

    let json_str = result
        .into_value::<String>()
        .map_err(|e| Error::ExtractionFailed(format!("Performance result decode failed: {e}")))?;

    let timing = serde_json::from_str::<serde_json::Value>(&json_str)
        .map_err(|e| Error::ExtractionFailed(format!("Performance JSON parse failed: {e}")))?;

    Ok(PerformanceTiming {
        navigation_start: timing["navigation_start"].as_u64().unwrap_or(0),
        dom_interactive: timing["dom_interactive"].as_u64(),
        dom_content_loaded: timing["dom_content_loaded"].as_u64(),
        load_complete: timing["load_complete"].as_u64(),
        response_start: timing["response_start"].as_u64(),
        first_paint: timing["first_paint"].as_u64(),
        first_contentful_paint: timing["first_contentful_paint"].as_u64(),
        largest_contentful_paint: timing["largest_contentful_paint"].as_u64(),
        cumulative_layout_shift: timing["cumulative_layout_shift"].as_f64(),
        interaction_to_next_paint: timing["interaction_to_next_paint"].as_u64(),
    })
}

/// Capture computed styles for visible elements.
async fn capture_computed_styles(page: &Page) -> Result<Vec<ComputedStyle>> {
    let js = r#"
        (function() {
            const styles = [];
            const elements = document.querySelectorAll('p, h1, h2, h3, h4, h5, h6, a, button, span, div');
            const canvas = document.createElement('canvas');
            const ctx = canvas.getContext('2d');

            function normalizeColor(value) {
                if (!ctx || !value) return null;
                try {
                    ctx.fillStyle = '#000';
                    ctx.fillStyle = value;
                    return ctx.fillStyle || null;
                } catch (_) {
                    return null;
                }
            }

            function isTransparent(value) {
                if (!value) return true;
                const normalized = normalizeColor(value) || value;
                return normalized === 'transparent' || normalized === 'rgba(0, 0, 0, 0)';
            }

            function effectiveBackground(el) {
                let current = el;
                while (current) {
                    const computed = window.getComputedStyle(current);
                    const candidate = normalizeColor(computed.backgroundColor);
                    if (candidate && !isTransparent(candidate)) {
                        return candidate;
                    }
                    current = current.parentElement;
                }
                return 'rgb(255, 255, 255)';
            }
            
            elements.forEach((el, index) => {
                const computed = window.getComputedStyle(el);
                const rect = el.getBoundingClientRect();
                
                // Only include visible elements
                if (rect.width > 0 && rect.height > 0) {
                    styles.push({
                        selector: el.tagName.toLowerCase() + (el.id ? '#' + el.id : '') + 
                                  (el.className ? '.' + el.className.split(' ').join('.') : ''),
                        color: normalizeColor(computed.color) || computed.color || null,
                        background_color: effectiveBackground(el),
                        font_size: computed.fontSize || null,
                        font_weight: computed.fontWeight || null
                    });
                }
            });
            
            return JSON.stringify(styles.slice(0, 100)); // Limit to 100 elements
        })()
    "#;

    let cdp_page = page
        .cdp_page()
        .ok_or_else(|| Error::ExtractionFailed("CDP page unavailable".to_string()))?;

    let result = cdp_page
        .evaluate(js)
        .await
        .map_err(|e| Error::ExtractionFailed(format!("Computed styles evaluation failed: {e}")))?;

    let json_str = result.into_value::<String>().map_err(|e| {
        Error::ExtractionFailed(format!("Computed styles result decode failed: {e}"))
    })?;

    serde_json::from_str::<Vec<ComputedStyle>>(&json_str)
        .map_err(|e| Error::ExtractionFailed(format!("Computed styles JSON parse failed: {e}")))
}

/// Internal extension to access the underlying CDP page
impl Page {
    /// Get access to the underlying chrome page (internal use only).
    pub(crate) fn cdp_page(&self) -> Option<&chromiumoxide::Page> {
        self.cdp_page.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_js_css_and_media_assets() {
        let html = r#"
            <html>
              <head>
                <script src="/assets/app.js"></script>
                <link rel="modulepreload" href="./chunk.js">
                <link rel="stylesheet" href="/assets/app.css">
              </head>
              <body>
                <img src="/img/hero.jpg" />
                <source srcset="/img/hero-1x.jpg 1x, /img/hero-2x.jpg 2x" />
              </body>
            </html>
        "#;

        let (assets, _favicon) = extract_referenced_assets(html, "https://example.com/page");

        assert!(assets
            .javascript
            .contains(&"https://example.com/assets/app.js".to_string()));
        assert!(assets
            .javascript
            .contains(&"https://example.com/chunk.js".to_string()));
        assert!(assets
            .stylesheets
            .contains(&"https://example.com/assets/app.css".to_string()));
        assert!(assets
            .media
            .contains(&"https://example.com/img/hero.jpg".to_string()));
        assert!(assets
            .media
            .contains(&"https://example.com/img/hero-1x.jpg".to_string()));
    }

    #[test]
    fn deduplicates_assets_and_ignores_data_urls() {
        let html = r#"
            <html>
              <head>
                <script src="/assets/app.js"></script>
                <script src="/assets/app.js"></script>
              </head>
              <body>
                <img src="data:image/png;base64,abc" />
                <img src="/img/one.png" />
                <img src="/img/one.png" />
              </body>
            </html>
        "#;

        let (assets, _favicon) = extract_referenced_assets(html, "https://example.com");

        assert_eq!(assets.javascript.len(), 1);
        assert_eq!(
            assets.media,
            vec!["https://example.com/img/one.png".to_string()]
        );
    }
}
