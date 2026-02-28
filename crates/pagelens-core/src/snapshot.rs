//! Page snapshot capture — DOM, accessibility tree, performance timing, and styles.

use crate::prelude::*;
use crate::browser::Page;

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
}

impl Default for SnapshotOptions {
    fn default() -> Self {
        Self {
            include_html: true,
            include_accessibility_tree: true,
            include_performance_timing: true,
            include_computed_styles: false, // Expensive, disabled by default
        }
    }
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
            },
            computed_styles: Vec::new(),
        }
    }
}

/// Extension trait for Page to add snapshot functionality.
pub trait SnapshotExt {
    /// Capture a snapshot of the current page state.
    async fn snapshot(&self, options: SnapshotOptions) -> Result<Snapshot>;
}

impl SnapshotExt for Page {
    async fn snapshot(&self, options: SnapshotOptions) -> Result<Snapshot> {
        let url = self.url().to_string();
        let title = self.title().await?;
        
        let mut snapshot = Snapshot::new(url, title);
        
        // Capture HTML if requested
        if options.include_html {
            snapshot.html = self.html().await?;
        }
        
        // Capture accessibility tree if requested
        if options.include_accessibility_tree {
            snapshot.accessibility_tree = capture_accessibility_tree(self).await?;
        }
        
        // Capture performance timing if requested
        if options.include_performance_timing {
            snapshot.performance_timing = capture_performance_timing(self).await?;
        }
        
        // Capture computed styles if requested
        if options.include_computed_styles {
            snapshot.computed_styles = capture_computed_styles(self).await?;
        }
        
        Ok(snapshot)
    }
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
    
    if let Some(cdp_page) = page.cdp_page() {
        match cdp_page.evaluate(js).await {
            Ok(result) => {
                if let Ok(json_str) = result.into_value::<String>() {
                    if let Ok(nodes) = serde_json::from_str::<Vec<AccessibilityNode>>(&json_str) {
                        return Ok(nodes);
                    }
                }
            }
            Err(_) => {
                // Fallback to empty tree on error
            }
        }
    }
    
    Ok(Vec::new())
}

/// Capture performance timing metrics.
async fn capture_performance_timing(page: &Page) -> Result<PerformanceTiming> {
    let js = r#"
        (function() {
            const timing = performance.timing;
            return JSON.stringify({
                navigation_start: timing.navigationStart,
                dom_interactive: timing.domInteractive || null,
                dom_content_loaded: timing.domContentLoadedEventEnd || null,
                load_complete: timing.loadEventEnd || null,
                response_start: timing.responseStart || null
            });
        })()
    "#;
    
    if let Some(cdp_page) = page.cdp_page() {
        match cdp_page.evaluate(js).await {
            Ok(result) => {
                if let Ok(json_str) = result.into_value::<String>() {
                    if let Ok(timing) = serde_json::from_str::<serde_json::Value>(&json_str) {
                        return Ok(PerformanceTiming {
                            navigation_start: timing["navigation_start"].as_u64().unwrap_or(0),
                            dom_interactive: timing["dom_interactive"].as_u64(),
                            dom_content_loaded: timing["dom_content_loaded"].as_u64(),
                            load_complete: timing["load_complete"].as_u64(),
                            response_start: timing["response_start"].as_u64(),
                        });
                    }
                }
            }
            Err(_) => {}
        }
    }
    
    Ok(PerformanceTiming {
        navigation_start: 0,
        dom_interactive: None,
        dom_content_loaded: None,
        load_complete: None,
        response_start: None,
    })
}

/// Capture computed styles for visible elements.
async fn capture_computed_styles(page: &Page) -> Result<Vec<ComputedStyle>> {
    let js = r#"
        (function() {
            const styles = [];
            const elements = document.querySelectorAll('p, h1, h2, h3, h4, h5, h6, a, button, span, div');
            
            elements.forEach((el, index) => {
                const computed = window.getComputedStyle(el);
                const rect = el.getBoundingClientRect();
                
                // Only include visible elements
                if (rect.width > 0 && rect.height > 0) {
                    styles.push({
                        selector: el.tagName.toLowerCase() + (el.id ? '#' + el.id : '') + 
                                  (el.className ? '.' + el.className.split(' ').join('.') : ''),
                        color: computed.color || null,
                        background_color: computed.backgroundColor || null,
                        font_size: computed.fontSize || null,
                        font_weight: computed.fontWeight || null
                    });
                }
            });
            
            return JSON.stringify(styles.slice(0, 100)); // Limit to 100 elements
        })()
    "#;
    
    if let Some(cdp_page) = page.cdp_page() {
        match cdp_page.evaluate(js).await {
            Ok(result) => {
                if let Ok(json_str) = result.into_value::<String>() {
                    if let Ok(styles) = serde_json::from_str::<Vec<ComputedStyle>>(&json_str) {
                        return Ok(styles);
                    }
                }
            }
            Err(_) => {}
        }
    }
    
    Ok(Vec::new())
}

/// Internal extension to access the underlying CDP page
impl Page {
    /// Get access to the underlying chromiumoxide page (internal use only).
    pub(crate) fn cdp_page(&self) -> Option<&chromiumoxide::Page> {
        self.cdp_page.as_ref()
    }
}
