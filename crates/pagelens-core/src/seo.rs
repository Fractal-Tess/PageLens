//! SEO analysis — Meta tags, headings, images, structured data, etc.

use crate::snapshot::Snapshot;
use scraper::{Html, Selector};
use std::sync::LazyLock;

// Only keep regex for JSON parsing (schema type extraction from JSON-LD)
static SCHEMA_TYPE_REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#""@type"\s*:\s*"([^"]+)""#).unwrap());

// Pre-compiled CSS selectors for HTML parsing
static IMG_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("img").expect("valid img selector"));
static HEADING_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("h1, h2, h3, h4, h5, h6").expect("valid heading selector"));
static META_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("meta").expect("valid meta selector"));
static JSONLD_SELECTOR: LazyLock<Selector> = LazyLock::new(|| {
    Selector::parse(r#"script[type="application/ld+json"]"#).expect("valid json-ld selector")
});
static TITLE_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("title").expect("valid title selector"));
static LINK_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("link").expect("valid link selector"));
static HTML_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("html").expect("valid html selector"));

/// Severity level for SEO issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Severity {
    /// Critical issue that significantly impacts SEO.
    Error,
    /// Warning for best practice violations.
    Warning,
    /// Informational suggestion.
    Info,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScoreConfig {
    pub error_penalty: f64,
    pub warning_penalty: f64,
    pub info_penalty: f64,
    pub title_bonus: f64,
    pub description_bonus: f64,
    pub canonical_bonus: f64,
    pub structured_data_bonus: f64,
}

impl Default for ScoreConfig {
    fn default() -> Self {
        Self {
            error_penalty: 15.0,
            warning_penalty: 5.0,
            info_penalty: 1.0,
            title_bonus: 5.0,
            description_bonus: 5.0,
            canonical_bonus: 2.0,
            structured_data_bonus: 5.0,
        }
    }
}

/// A single SEO issue.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Issue {
    /// The severity of this issue.
    pub severity: Severity,
    /// Human-readable description of the issue.
    pub message: String,
    /// Category this issue belongs to.
    pub category: String,
}

/// Meta tag information.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct MetaInfo {
    /// Whether charset is specified.
    pub charset: bool,
    /// Whether viewport meta tag is present.
    pub viewport: bool,
    /// The page title (from <title> tag).
    pub title: Option<String>,
    /// The meta description.
    pub description: Option<String>,
    /// The language attribute from <html> tag.
    pub language: Option<String>,
    /// Robots meta directive.
    pub robots: Option<String>,
}

/// Open Graph tag information.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct OpenGraphInfo {
    /// OG title.
    pub title: Option<String>,
    /// OG description.
    pub description: Option<String>,
    /// OG type (article, website, etc.).
    pub og_type: Option<String>,
    /// OG URL.
    pub url: Option<String>,
    /// OG image URL.
    pub image: Option<String>,
}

/// Twitter Card information.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TwitterCardInfo {
    /// Card type (summary, summary_large_image, etc.).
    pub card: Option<String>,
    /// Twitter title.
    pub title: Option<String>,
    /// Twitter description.
    pub description: Option<String>,
    /// Twitter image.
    pub image: Option<String>,
}

/// Heading structure information.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct HeadingsInfo {
    /// Number of H1 tags.
    pub h1_count: usize,
    /// Number of H2 tags.
    pub h2_count: usize,
    /// Number of H3 tags.
    pub h3_count: usize,
    /// Number of H4 tags.
    pub h4_count: usize,
    /// Number of H5 tags.
    pub h5_count: usize,
    /// Number of H6 tags.
    pub h6_count: usize,
    /// The order of headings as they appear (e.g., ["h1", "h2", "h3"]).
    pub structure: Vec<String>,
}

/// Image information.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageInfo {
    /// Source URL.
    pub src: String,
    /// Alt text (if present).
    pub alt: Option<String>,
    /// Whether the image has alt text.
    pub has_alt: bool,
}

/// Structured data entry (JSON-LD).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StructuredData {
    /// The @type field from the JSON-LD.
    pub schema_type: String,
    /// Raw JSON content.
    pub raw: String,
}

/// Complete SEO analysis report.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SeoReport {
    /// Overall SEO score (0-100).
    pub score: f64,
    /// Meta tag information.
    pub meta: MetaInfo,
    /// Open Graph information.
    pub open_graph: OpenGraphInfo,
    /// Twitter Card information.
    pub twitter_card: TwitterCardInfo,
    /// Canonical URL (if present).
    pub canonical_url: Option<String>,
    /// Heading structure.
    pub headings: HeadingsInfo,
    /// Images found on the page.
    pub images: Vec<ImageInfo>,
    /// Structured data (JSON-LD).
    pub structured_data: Vec<StructuredData>,
    /// List of SEO issues found.
    pub issues: Vec<Issue>,
}

impl SeoReport {
    /// Create a new empty SEO report.
    pub fn new() -> Self {
        Self {
            score: 0.0,
            meta: MetaInfo::default(),
            open_graph: OpenGraphInfo::default(),
            twitter_card: TwitterCardInfo::default(),
            canonical_url: None,
            headings: HeadingsInfo::default(),
            images: Vec::new(),
            structured_data: Vec::new(),
            issues: Vec::new(),
        }
    }

    /// Add an issue to the report.
    fn add_issue(&mut self, severity: Severity, category: &str, message: &str) {
        self.issues.push(Issue {
            severity,
            message: message.to_string(),
            category: category.to_string(),
        });
    }
}

impl Default for SeoReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Analyzer for SEO checks.
pub struct SeoAnalyzer;

impl SeoAnalyzer {
    /// Analyze a page snapshot and produce an SEO report.
    pub fn analyze(snapshot: &Snapshot) -> SeoReport {
        Self::analyze_with_config(snapshot, &ScoreConfig::default())
    }

    pub fn analyze_with_config(snapshot: &Snapshot, score_config: &ScoreConfig) -> SeoReport {
        let mut report = SeoReport::new();
        let html = &snapshot.html;
        let document = Html::parse_document(html);

        // Analyze meta tags
        Self::analyze_meta(&mut report, &document);

        // Analyze Open Graph
        Self::analyze_open_graph(&mut report, &document);

        // Analyze Twitter Cards
        Self::analyze_twitter_cards(&mut report, &document);

        // Analyze canonical URL
        Self::analyze_canonical(&mut report, &document);

        // Analyze headings
        Self::analyze_headings(&mut report, &document);

        // Analyze images
        Self::analyze_images(&mut report, &document);

        // Analyze structured data
        Self::analyze_structured_data(&mut report, &document);

        // Calculate final score
        report.score = Self::calculate_score(&report, score_config);

        report
    }

    /// Analyze meta tags.
    fn analyze_meta(report: &mut SeoReport, document: &Html) {
        // Check charset - look for meta[charset] or meta[http-equiv="Content-Type"]
        report.meta.charset = document.select(&META_SELECTOR).any(|el| {
            el.value().attr("charset").is_some()
                || el
                    .value()
                    .attr("http-equiv")
                    .map(|v| v.eq_ignore_ascii_case("content-type"))
                    .unwrap_or(false)
        });
        if !report.meta.charset {
            report.add_issue(Severity::Error, "meta", "Missing charset declaration");
        }

        // Check viewport
        report.meta.viewport = Self::get_meta_content(document, "viewport").is_some();
        if !report.meta.viewport {
            report.add_issue(
                Severity::Warning,
                "meta",
                "Missing viewport meta tag (important for mobile SEO)",
            );
        }

        // Extract title
        if let Some(title_elem) = document.select(&TITLE_SELECTOR).next() {
            let title = title_elem.text().collect::<String>().trim().to_string();
            let title_len = title.len();
            report.meta.title = Some(title.clone());

            if title_len == 0 {
                report.add_issue(Severity::Error, "meta", "Title tag is empty");
            } else if title_len < 10 {
                report.add_issue(
                    Severity::Warning,
                    "meta",
                    &format!("Title is too short ({} chars)", title_len),
                );
            } else if title_len > 60 {
                report.add_issue(
                    Severity::Warning,
                    "meta",
                    &format!(
                        "Title may be truncated in search results ({} chars)",
                        title_len
                    ),
                );
            }
        } else {
            report.add_issue(Severity::Error, "meta", "Missing title tag");
        }

        // Extract description
        if let Some(desc) = Self::get_meta_content(document, "description") {
            let desc_len = desc.len();
            report.meta.description = Some(desc.clone());

            if desc_len == 0 {
                report.add_issue(Severity::Warning, "meta", "Meta description is empty");
            } else if desc_len < 50 {
                report.add_issue(
                    Severity::Info,
                    "meta",
                    &format!("Description is short ({} chars)", desc_len),
                );
            } else if desc_len > 160 {
                report.add_issue(
                    Severity::Info,
                    "meta",
                    &format!("Description may be truncated ({} chars)", desc_len),
                );
            }
        } else {
            report.add_issue(Severity::Warning, "meta", "Missing meta description");
        }

        // Extract language
        if let Some(html_elem) = document.select(&HTML_SELECTOR).next() {
            if let Some(lang) = html_elem.value().attr("lang") {
                report.meta.language = Some(lang.to_string());
            } else {
                report.add_issue(
                    Severity::Info,
                    "meta",
                    "Missing lang attribute on <html> tag",
                );
            }
        } else {
            report.add_issue(
                Severity::Info,
                "meta",
                "Missing lang attribute on <html> tag",
            );
        }
    }

    /// Analyze Open Graph tags.
    fn analyze_open_graph(report: &mut SeoReport, document: &Html) {
        report.open_graph.title = Self::get_meta_property(document, "og:title");
        report.open_graph.description = Self::get_meta_property(document, "og:description");
        report.open_graph.og_type = Self::get_meta_property(document, "og:type");
        report.open_graph.url = Self::get_meta_property(document, "og:url");
        report.open_graph.image = Self::get_meta_property(document, "og:image");

        // Check for missing OG tags
        if report.open_graph.title.is_none() {
            report.add_issue(Severity::Info, "open-graph", "Missing og:title");
        }
        if report.open_graph.description.is_none() {
            report.add_issue(Severity::Info, "open-graph", "Missing og:description");
        }
        if report.open_graph.image.is_none() {
            report.add_issue(
                Severity::Info,
                "open-graph",
                "Missing og:image (important for social sharing)",
            );
        }
    }

    /// Analyze Twitter Card tags.
    fn analyze_twitter_cards(report: &mut SeoReport, document: &Html) {
        report.twitter_card.card = Self::get_meta_name_or_property(document, "twitter:card");
        report.twitter_card.title = Self::get_meta_name_or_property(document, "twitter:title");
        report.twitter_card.description =
            Self::get_meta_name_or_property(document, "twitter:description");
        report.twitter_card.image = Self::get_meta_name_or_property(document, "twitter:image");
    }

    /// Analyze canonical URL.
    fn analyze_canonical(report: &mut SeoReport, document: &Html) {
        // Look for <link rel="canonical" href="...">
        if let Some(href) = Self::get_link_rel(document, "canonical") {
            report.canonical_url = Some(href);
        } else {
            report.add_issue(
                Severity::Info,
                "canonical",
                "Missing canonical URL (recommended for SEO)",
            );
        }
    }

    /// Analyze heading structure.
    fn analyze_headings(report: &mut SeoReport, document: &Html) {
        // Collect all headings in order
        let headings: Vec<_> = document.select(&HEADING_SELECTOR).collect();

        // Count headings by level
        for heading in &headings {
            let tag_name = heading.value().name.local.as_ref();
            match tag_name {
                "h1" => report.headings.h1_count += 1,
                "h2" => report.headings.h2_count += 1,
                "h3" => report.headings.h3_count += 1,
                "h4" => report.headings.h4_count += 1,
                "h5" => report.headings.h5_count += 1,
                "h6" => report.headings.h6_count += 1,
                _ => {}
            }
            report.headings.structure.push(tag_name.to_string());
        }

        // Check for multiple H1s
        if report.headings.h1_count > 1 {
            report.add_issue(Severity::Warning, "headings", 
                &format!("Multiple H1 tags found ({}). Best practice is to have exactly one H1 per page.", 
                    report.headings.h1_count));
        }

        if report.headings.h1_count == 0 {
            report.add_issue(Severity::Error, "headings", "No H1 tag found");
        }

        // Check for skipped heading levels
        let structure_clone = report.headings.structure.clone();
        if !structure_clone.is_empty() {
            let mut prev_level = 0u8;
            for heading in &structure_clone {
                if let Ok(level) = heading[1..].parse::<u8>() {
                    if level > prev_level + 1 && prev_level > 0 {
                        report.add_issue(
                            Severity::Info,
                            "headings",
                            &format!("Heading level skipped: H{} follows H{}", level, prev_level),
                        );
                    }
                    prev_level = level;
                }
            }
        }
    }

    /// Analyze images.
    fn analyze_images(report: &mut SeoReport, document: &Html) {
        for img in document.select(&IMG_SELECTOR) {
            let src = img.value().attr("src").unwrap_or("").to_string();
            let alt = img.value().attr("alt").map(|s| s.to_string());
            let has_alt = alt.is_some();

            if src.is_empty() {
                report.add_issue(Severity::Error, "images", "Image without src attribute");
            } else {
                report.images.push(ImageInfo {
                    src: src.clone(),
                    alt: alt.clone(),
                    has_alt,
                });

                if !has_alt {
                    report.add_issue(
                        Severity::Warning,
                        "images",
                        &format!("Image missing alt text: {}", &src[..src.len().min(50)]),
                    );
                } else if alt.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                    report.add_issue(
                        Severity::Warning,
                        "images",
                        &format!("Image has empty alt text: {}", &src[..src.len().min(50)]),
                    );
                }
            }
        }
    }

    /// Analyze structured data (JSON-LD).
    fn analyze_structured_data(report: &mut SeoReport, document: &Html) {
        for script in document.select(&JSONLD_SELECTOR) {
            let json_content = script.text().collect::<String>();

            // Try to extract @type using regex (JSON parsing)
            let schema_type = SCHEMA_TYPE_REGEX
                .captures(&json_content)
                .map(|c| c[1].to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            report.structured_data.push(StructuredData {
                schema_type,
                raw: json_content.trim().to_string(),
            });
        }
    }

    /// Calculate overall SEO score.
    fn calculate_score(report: &SeoReport, score_config: &ScoreConfig) -> f64 {
        let mut score = 100.0;

        // Deduct points for errors
        let error_count = report
            .issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .count();
        score -= error_count as f64 * score_config.error_penalty;

        // Deduct points for warnings
        let warning_count = report
            .issues
            .iter()
            .filter(|i| i.severity == Severity::Warning)
            .count();
        score -= warning_count as f64 * score_config.warning_penalty;

        // Deduct points for info (minor)
        let info_count = report
            .issues
            .iter()
            .filter(|i| i.severity == Severity::Info)
            .count();
        score -= info_count as f64 * score_config.info_penalty;

        // Bonus for good practices
        if report.meta.title.is_some() {
            score += score_config.title_bonus;
        }
        if report.meta.description.is_some() {
            score += score_config.description_bonus;
        }
        if report.canonical_url.is_some() {
            score += score_config.canonical_bonus;
        }
        if !report.structured_data.is_empty() {
            score += score_config.structured_data_bonus;
        }

        score.clamp(0.0, 100.0)
    }

    // =========================================================================
    // Helper functions using scraper
    // =========================================================================

    /// Get meta tag content by name attribute.
    fn get_meta_content(document: &Html, name: &str) -> Option<String> {
        document
            .select(&META_SELECTOR)
            .find(|el| {
                el.value()
                    .attr("name")
                    .map(|n| n.eq_ignore_ascii_case(name))
                    .unwrap_or(false)
            })
            .and_then(|el| el.value().attr("content"))
            .map(|s| s.to_string())
    }

    /// Get meta tag content by property attribute (for Open Graph).
    fn get_meta_property(document: &Html, property: &str) -> Option<String> {
        document
            .select(&META_SELECTOR)
            .find(|el| {
                el.value()
                    .attr("property")
                    .map(|p| p.eq_ignore_ascii_case(property))
                    .unwrap_or(false)
            })
            .and_then(|el| el.value().attr("content"))
            .map(|s| s.to_string())
    }

    /// Get meta tag content by name attribute (alias for get_meta_content).
    fn get_meta_name(document: &Html, name: &str) -> Option<String> {
        Self::get_meta_content(document, name)
    }

    fn get_meta_name_or_property(document: &Html, key: &str) -> Option<String> {
        Self::get_meta_name(document, key).or_else(|| Self::get_meta_property(document, key))
    }

    /// Get href from link tag with specific rel attribute.
    fn get_link_rel(document: &Html, rel: &str) -> Option<String> {
        document
            .select(&LINK_SELECTOR)
            .find(|el| {
                el.value()
                    .attr("rel")
                    .map(|r| r.eq_ignore_ascii_case(rel))
                    .unwrap_or(false)
            })
            .and_then(|el| el.value().attr("href"))
            .map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_meta_content() {
        let html = r#"<meta name="description" content="Test description">"#;
        let document = Html::parse_fragment(html);
        assert_eq!(
            SeoAnalyzer::get_meta_content(&document, "description"),
            Some("Test description".to_string())
        );
    }

    #[test]
    fn test_get_meta_content_case_insensitive() {
        let html = r#"<meta name="DESCRIPTION" content="Test description">"#;
        let document = Html::parse_fragment(html);
        assert_eq!(
            SeoAnalyzer::get_meta_content(&document, "description"),
            Some("Test description".to_string())
        );
    }

    #[test]
    fn test_get_meta_name_or_property_fallback() {
        let html = r#"<meta property="twitter:card" content="summary_large_image">"#;
        let document = Html::parse_fragment(html);
        assert_eq!(
            SeoAnalyzer::get_meta_name_or_property(&document, "twitter:card"),
            Some("summary_large_image".to_string())
        );
    }

    // =====================================================================
    // Full analyze() unit tests with crafted HTML
    // =====================================================================

    fn snapshot_with_html(html: &str) -> crate::snapshot::Snapshot {
        crate::snapshot::Snapshot {
            url: "http://test.local".to_string(),
            title: String::new(),
            html: html.to_string(),
            accessibility_tree: Vec::new(),
            performance_timing: crate::snapshot::PerformanceTiming {
                navigation_start: 0,
                dom_interactive: None,
                dom_content_loaded: None,
                load_complete: None,
                response_start: None,
                first_paint: None,
                first_contentful_paint: None,
            },
            computed_styles: Vec::new(),
            referenced_assets: crate::snapshot::ReferencedAssets::default(),
            capture_issues: Vec::new(),
        }
    }

    #[test]
    fn analyze_empty_html() {
        let snapshot = snapshot_with_html("");
        let report = SeoAnalyzer::analyze(&snapshot);

        assert!(report.meta.title.is_none());
        assert!(!report.meta.charset);
        assert!(!report.meta.viewport);
        assert_eq!(report.headings.h1_count, 0);
        assert!(report.score < 50.0, "Empty page should score poorly");

        // Should have errors for missing title, charset, h1
        let has_missing_title = report
            .issues
            .iter()
            .any(|i| i.message.contains("Missing title"));
        let has_missing_charset = report
            .issues
            .iter()
            .any(|i| i.message.contains("Missing charset"));
        let has_no_h1 = report.issues.iter().any(|i| i.message.contains("No H1"));
        assert!(has_missing_title);
        assert!(has_missing_charset);
        assert!(has_no_h1);
    }

    #[test]
    fn analyze_well_formed_page() {
        let html = r#"
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>My Great Website - Home Page</title>
                <meta name="description" content="A well-optimized website with proper meta tags for good SEO performance.">
                <meta property="og:title" content="My Great Website">
                <meta property="og:description" content="A well-optimized website">
                <meta property="og:image" content="https://example.com/image.png">
                <link rel="canonical" href="https://example.com/">
                <script type="application/ld+json">{"@type": "WebSite", "name": "My Great Website"}</script>
            </head>
            <body>
                <h1>Welcome to My Website</h1>
                <h2>About Us</h2>
                <p>Some content here.</p>
                <h2>Services</h2>
                <img src="photo.jpg" alt="A descriptive alt text">
            </body>
            </html>
        "#;

        let snapshot = snapshot_with_html(html);
        let report = SeoAnalyzer::analyze(&snapshot);

        assert!(report.meta.charset);
        assert!(report.meta.viewport);
        assert_eq!(
            report.meta.title.as_deref(),
            Some("My Great Website - Home Page")
        );
        assert!(report.meta.description.is_some());
        assert_eq!(report.meta.language.as_deref(), Some("en"));
        assert_eq!(report.headings.h1_count, 1);
        assert_eq!(report.headings.h2_count, 2);
        assert!(report.canonical_url.is_some());
        assert!(report.open_graph.title.is_some());
        assert!(!report.structured_data.is_empty());
        assert_eq!(report.images.len(), 1);
        assert!(report.images[0].has_alt);
        assert!(
            report.score >= 90.0,
            "Well-formed page should score high, got {}",
            report.score
        );
    }

    #[test]
    fn analyze_multiple_h1s() {
        let html = r#"
            <html><head><meta charset="utf-8"><title>Test Page Title</title></head>
            <body>
                <h1>First Heading</h1>
                <h1>Second Heading</h1>
                <h1>Third Heading</h1>
            </body></html>
        "#;

        let snapshot = snapshot_with_html(html);
        let report = SeoAnalyzer::analyze(&snapshot);

        assert_eq!(report.headings.h1_count, 3);
        let h1_issue = report
            .issues
            .iter()
            .find(|i| i.message.contains("Multiple H1"));
        assert!(h1_issue.is_some());
        assert_eq!(h1_issue.unwrap().severity, Severity::Warning);
    }

    #[test]
    fn analyze_images_without_alt() {
        let html = r#"
            <html><head><meta charset="utf-8"><title>Image Test Page</title></head>
            <body>
                <h1>Images</h1>
                <img src="good.jpg" alt="A good image">
                <img src="bad.jpg">
                <img src="empty-alt.jpg" alt="">
            </body></html>
        "#;

        let snapshot = snapshot_with_html(html);
        let report = SeoAnalyzer::analyze(&snapshot);

        assert_eq!(report.images.len(), 3);
        assert!(report.images[0].has_alt);
        assert!(!report.images[1].has_alt);
        assert!(report.images[2].has_alt); // has alt attr, just empty

        let missing_alt = report
            .issues
            .iter()
            .filter(|i| i.message.contains("missing alt"))
            .count();
        let empty_alt = report
            .issues
            .iter()
            .filter(|i| i.message.contains("empty alt"))
            .count();
        assert_eq!(missing_alt, 1);
        assert_eq!(empty_alt, 1);
    }

    #[test]
    fn analyze_skipped_heading_levels() {
        let html = r#"
            <html><head><meta charset="utf-8"><title>Heading Skip Test</title></head>
            <body>
                <h1>Title</h1>
                <h3>Jumped from H1 to H3</h3>
            </body></html>
        "#;

        let snapshot = snapshot_with_html(html);
        let report = SeoAnalyzer::analyze(&snapshot);

        let skipped = report
            .issues
            .iter()
            .any(|i| i.message.contains("Heading level skipped"));
        assert!(skipped, "Should detect skipped heading level");
    }

    #[test]
    fn analyze_structured_data_extraction() {
        let html = r#"
            <html><head>
                <meta charset="utf-8"><title>Blog Post About Rust</title>
                <script type="application/ld+json">{"@type": "Article", "headline": "Rust is great"}</script>
                <script type="application/ld+json">{"@type": "BreadcrumbList"}</script>
            </head>
            <body><h1>Blog Post</h1></body></html>
        "#;

        let snapshot = snapshot_with_html(html);
        let report = SeoAnalyzer::analyze(&snapshot);

        assert_eq!(report.structured_data.len(), 2);
        assert!(report
            .structured_data
            .iter()
            .any(|s| s.schema_type == "Article"));
        assert!(report
            .structured_data
            .iter()
            .any(|s| s.schema_type == "BreadcrumbList"));
    }

    #[test]
    fn analyze_title_length_boundaries() {
        // Too short
        let short = snapshot_with_html(
            r#"<html><head><meta charset="utf-8"><title>Hi</title></head><body><h1>X</h1></body></html>"#,
        );
        let report = SeoAnalyzer::analyze(&short);
        assert!(report
            .issues
            .iter()
            .any(|i| i.message.contains("too short")));

        // Too long (>60 chars)
        let long_title = "A".repeat(65);
        let long_html = format!(
            r#"<html><head><meta charset="utf-8"><title>{}</title></head><body><h1>X</h1></body></html>"#,
            long_title
        );
        let long = snapshot_with_html(&long_html);
        let report = SeoAnalyzer::analyze(&long);
        assert!(report
            .issues
            .iter()
            .any(|i| i.message.contains("truncated")));
    }

    #[test]
    fn score_clamped_to_bounds() {
        // Even a terrible page shouldn't go below 0
        let snapshot = snapshot_with_html("");
        let report = SeoAnalyzer::analyze(&snapshot);
        assert!(report.score >= 0.0);
        assert!(report.score <= 100.0);
    }

    #[test]
    fn analyze_with_custom_score_config_changes_score() {
        let snapshot = snapshot_with_html(
            r#"<html><head><meta charset="utf-8"><title>Custom Score Page</title></head><body><h1>Test</h1></body></html>"#,
        );

        let default_report = SeoAnalyzer::analyze(&snapshot);
        let custom = ScoreConfig {
            warning_penalty: 20.0,
            ..ScoreConfig::default()
        };
        let custom_report = SeoAnalyzer::analyze_with_config(&snapshot, &custom);

        assert!(custom_report.score < default_report.score);
    }

    #[test]
    fn analyze_meta_attribute_order() {
        // content before name (reversed order)
        let html = r#"<html><head>
            <meta charset="utf-8">
            <title>Attribute Order Test Page</title>
            <meta content="Reversed order description" name="description">
        </head><body><h1>Test</h1></body></html>"#;

        let snapshot = snapshot_with_html(html);
        let report = SeoAnalyzer::analyze(&snapshot);
        assert_eq!(
            report.meta.description.as_deref(),
            Some("Reversed order description")
        );
    }
}
