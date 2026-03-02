//! SEO analysis — Meta tags, headings, images, structured data, etc.

use crate::snapshot::Snapshot;
use scraper::{Html, Selector};
use std::sync::LazyLock;
use url::Url;

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
static ANCHOR_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("a").expect("valid anchor selector"));
static FORM_CONTROL_SELECTOR: LazyLock<Selector> = LazyLock::new(|| {
    Selector::parse("input, select, textarea").expect("valid form control selector")
});
static LABEL_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("label").expect("valid label selector"));
static SCRIPT_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("script[src]").expect("valid script selector"));
static TARGET_BLANK_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("a[target=\"_blank\"]").expect("valid target blank selector"));
static ROBOTS_USER_AGENT_NAMES: &[&str] = &["robots", "googlebot", "bingbot", "duckduckbot"];
static ROBOTS_BLOCKLIST: &[&str] = &["noindex", "none"];

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

        Self::analyze_document_basics(&mut report, snapshot);

        // Analyze Open Graph
        Self::analyze_open_graph(&mut report, &document);

        // Analyze Twitter Cards
        Self::analyze_twitter_cards(&mut report, &document);

        // Analyze canonical URL
        Self::analyze_canonical(&mut report, &document, &snapshot.url);

        Self::analyze_hreflang(&mut report, &document);

        // Analyze headings
        Self::analyze_headings(&mut report, &document);

        // Analyze images
        Self::analyze_images(&mut report, &document);

        Self::analyze_links(&mut report, &document);

        Self::analyze_form_controls(&mut report, &document);

        Self::analyze_crawlability(&mut report, &document);

        Self::analyze_network_artifacts(&mut report, snapshot);

        Self::analyze_network_request_records(&mut report, snapshot);

        Self::analyze_best_practices(&mut report, &document, snapshot);

        Self::analyze_performance(&mut report, snapshot);

        Self::analyze_accessibility_styles(&mut report, snapshot);

        Self::analyze_media_optimization(&mut report, &document);

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
    fn analyze_canonical(report: &mut SeoReport, document: &Html, page_url: &str) {
        // Look for <link rel="canonical" href="...">
        let canonical_links = Self::get_link_rels(document, "canonical");

        if canonical_links.is_empty() {
            report.add_issue(
                Severity::Info,
                "canonical",
                "Missing canonical URL (recommended for SEO)",
            );
            return;
        }

        let mut unique = std::collections::HashSet::new();
        for href in &canonical_links {
            unique.insert(href.clone());
        }

        if unique.len() > 1 {
            report.add_issue(
                Severity::Warning,
                "canonical",
                "Multiple conflicting canonical URLs found",
            );
        }

        if let Some(primary) = canonical_links.first() {
            report.canonical_url = Some(primary.clone());

            if Url::parse(primary).is_err() {
                report.add_issue(
                    Severity::Warning,
                    "canonical",
                    "Canonical URL is not absolute",
                );
            }

            if let (Ok(page), Ok(canonical)) = (Url::parse(page_url), Url::parse(primary)) {
                if page.origin() == canonical.origin()
                    && page.path() != "/"
                    && canonical.path() == "/"
                {
                    report.add_issue(
                        Severity::Info,
                        "canonical",
                        "Canonical URL points to site root from a non-root page",
                    );
                }
            }
        }
    }

    fn analyze_hreflang(report: &mut SeoReport, document: &Html) {
        let mut invalid_count = 0usize;

        for link in document.select(&LINK_SELECTOR) {
            let rel = link.value().attr("rel").unwrap_or("");
            let is_alternate = rel
                .split_ascii_whitespace()
                .any(|token| token.eq_ignore_ascii_case("alternate"));
            if !is_alternate {
                continue;
            }

            let Some(hreflang) = link.value().attr("hreflang") else {
                continue;
            };

            let href = link.value().attr("href").unwrap_or("");
            if href.is_empty() || Url::parse(href).is_err() {
                invalid_count += 1;
                continue;
            }

            if !Self::is_expected_hreflang_code(hreflang) {
                invalid_count += 1;
            }
        }

        if invalid_count > 0 {
            report.add_issue(
                Severity::Warning,
                "hreflang",
                &format!("Found {} invalid hreflang link(s)", invalid_count),
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

    fn analyze_links(report: &mut SeoReport, document: &Html) {
        for anchor in document.select(&ANCHOR_SELECTOR) {
            let href = anchor.value().attr("href").unwrap_or("").trim();
            if href.is_empty() || href.eq_ignore_ascii_case("javascript:void(0)") {
                report.add_issue(
                    Severity::Info,
                    "links",
                    "Found anchor with non-crawlable href",
                );
            }

            let text = anchor.text().collect::<String>().trim().to_string();
            let has_accessible_label = anchor
                .value()
                .attr("aria-label")
                .map(|v| !v.trim().is_empty())
                .unwrap_or(false)
                || anchor
                    .value()
                    .attr("title")
                    .map(|v| !v.trim().is_empty())
                    .unwrap_or(false);

            if text.is_empty() && !has_accessible_label {
                report.add_issue(
                    Severity::Warning,
                    "links",
                    "Found link without descriptive text or accessible label",
                );
            }
        }
    }

    fn analyze_form_controls(report: &mut SeoReport, document: &Html) {
        let mut labels_for: std::collections::HashSet<String> = std::collections::HashSet::new();
        for label in document.select(&LABEL_SELECTOR) {
            if let Some(for_attr) = label.value().attr("for") {
                if !for_attr.trim().is_empty() {
                    labels_for.insert(for_attr.to_string());
                }
            }
        }

        for control in document.select(&FORM_CONTROL_SELECTOR) {
            let control_name = control.value().name.local.to_string();
            let id = control.value().attr("id").unwrap_or("").trim().to_string();
            let has_label_for = !id.is_empty() && labels_for.contains(&id);
            let has_aria = control
                .value()
                .attr("aria-label")
                .map(|v| !v.trim().is_empty())
                .unwrap_or(false)
                || control
                    .value()
                    .attr("aria-labelledby")
                    .map(|v| !v.trim().is_empty())
                    .unwrap_or(false);

            if !has_label_for && !has_aria {
                report.add_issue(
                    Severity::Warning,
                    "accessibility",
                    &format!("Found unlabeled form control: {}", control_name),
                );
            }
        }
    }

    fn analyze_document_basics(report: &mut SeoReport, snapshot: &Snapshot) {
        let trimmed = snapshot.html.trim_start();
        if !trimmed.to_ascii_lowercase().starts_with("<!doctype html") {
            report.add_issue(
                Severity::Warning,
                "best-practices",
                "Document is missing an HTML5 doctype",
            );
        }

        if let Some(charset_index) = Self::find_case_insensitive(&snapshot.html, "<meta charset") {
            if charset_index > 1024 {
                report.add_issue(
                    Severity::Warning,
                    "meta",
                    "Charset declaration appears after the first 1024 bytes",
                );
            }
        }
    }

    fn analyze_crawlability(report: &mut SeoReport, document: &Html) {
        for meta in document.select(&META_SELECTOR) {
            let name = meta
                .value()
                .attr("name")
                .unwrap_or("")
                .trim()
                .to_ascii_lowercase();
            if !ROBOTS_USER_AGENT_NAMES.contains(&name.as_str()) {
                continue;
            }

            let content = meta.value().attr("content").unwrap_or("");
            let has_blocking_directive = content
                .split(',')
                .map(|d| d.trim().to_ascii_lowercase())
                .any(|directive| ROBOTS_BLOCKLIST.contains(&directive.as_str()));

            if has_blocking_directive {
                let severity = if name == "robots" {
                    Severity::Error
                } else {
                    Severity::Warning
                };
                report.add_issue(
                    severity,
                    "seo",
                    &format!("Found blocking indexing directive in {} meta tag", name),
                );
            }
        }
    }

    fn analyze_network_artifacts(report: &mut SeoReport, snapshot: &Snapshot) {
        let network = &snapshot.main_resource_network;

        if let Some(status) = network.status_code {
            if status >= 400 {
                report.add_issue(
                    Severity::Error,
                    "seo",
                    &format!("Main resource returned unsuccessful status code {}", status),
                );
            }
        } else if network.fetch_error.is_some() {
            report.add_issue(
                Severity::Info,
                "best-practices",
                "Unable to collect main resource status and headers",
            );
        }

        if Self::header_contains_token(&network.headers, "x-robots-tag", "noindex")
            || Self::header_contains_token(&network.headers, "x-robots-tag", "none")
        {
            report.add_issue(
                Severity::Error,
                "seo",
                "X-Robots-Tag blocks indexing for this page",
            );
        }

        if let Some(content_type) = network.headers.get("content-type") {
            if content_type.contains("text/html") {
                let cache_control = network
                    .headers
                    .get("cache-control")
                    .map(|v| v.to_ascii_lowercase())
                    .unwrap_or_default();

                if cache_control.is_empty() {
                    report.add_issue(
                        Severity::Info,
                        "performance",
                        "Main resource is missing cache-control header",
                    );
                } else if cache_control.contains("no-store") {
                    report.add_issue(
                        Severity::Info,
                        "performance",
                        "Main resource disables caching via cache-control: no-store",
                    );
                }
            }
        }

        if network.headers.get("content-encoding").is_none() {
            report.add_issue(
                Severity::Info,
                "performance",
                "Main resource is missing content-encoding (gzip/brotli)",
            );
        }

        if snapshot.url.starts_with("https://") {
            let missing_security_headers = [
                ("content-security-policy", "CSP"),
                ("strict-transport-security", "HSTS"),
                ("x-content-type-options", "X-Content-Type-Options"),
                ("x-frame-options", "X-Frame-Options"),
                ("referrer-policy", "Referrer-Policy"),
            ];

            for (header, label) in missing_security_headers {
                if network
                    .headers
                    .get(header)
                    .map(|v| v.trim().is_empty())
                    .unwrap_or(true)
                {
                    report.add_issue(
                        Severity::Info,
                        "best-practices",
                        &format!("Missing security header: {}", label),
                    );
                }
            }
        }
    }

    fn analyze_network_request_records(report: &mut SeoReport, snapshot: &Snapshot) {
        if snapshot.network_requests.is_empty() {
            return;
        }

        if snapshot.url.starts_with("https://")
            && snapshot
                .network_requests
                .iter()
                .any(|request| request.url.starts_with("http://"))
        {
            report.add_issue(
                Severity::Warning,
                "best-practices",
                "Detected mixed content request in network activity",
            );
        }

        let redirect_count = snapshot
            .network_requests
            .iter()
            .filter(|request| request.was_redirect.unwrap_or(false))
            .count();
        if redirect_count > 0 {
            report.add_issue(
                Severity::Info,
                "performance",
                &format!(
                    "Detected {} redirect network request(s); reduce redirects for faster loads",
                    redirect_count
                ),
            );
        }

        let uncompressed_count = snapshot
            .network_requests
            .iter()
            .filter(|request| {
                let Some(mime) = request.mime_type.as_deref() else {
                    return false;
                };

                let text_like = mime.starts_with("text/")
                    || mime.contains("javascript")
                    || mime.contains("json")
                    || mime.contains("xml");

                text_like
                    && request.encoded_data_length.unwrap_or(0.0) > 1024.0
                    && request
                        .content_encoding
                        .as_deref()
                        .map(|v| v.trim().is_empty())
                        .unwrap_or(true)
            })
            .count();

        if uncompressed_count > 0 {
            report.add_issue(
                Severity::Info,
                "performance",
                &format!(
                    "Found {} text request(s) without compression",
                    uncompressed_count
                ),
            );
        }

        let failed_count = snapshot
            .network_requests
            .iter()
            .filter(|request| request.failed.unwrap_or(false))
            .count();
        if failed_count > 0 {
            report.add_issue(
                Severity::Warning,
                "best-practices",
                &format!("Detected {} failed network request(s)", failed_count),
            );
        }
    }

    fn analyze_best_practices(report: &mut SeoReport, document: &Html, snapshot: &Snapshot) {
        if Self::should_check_secure_transport(&snapshot.url) && snapshot.url.starts_with("http://")
        {
            report.add_issue(
                Severity::Warning,
                "best-practices",
                "Page is served over HTTP instead of HTTPS",
            );
        }

        if snapshot.url.starts_with("https://") {
            let mixed_content = snapshot
                .referenced_assets
                .javascript
                .iter()
                .chain(snapshot.referenced_assets.stylesheets.iter())
                .chain(snapshot.referenced_assets.media.iter())
                .any(|url| url.starts_with("http://"));

            if mixed_content {
                report.add_issue(
                    Severity::Error,
                    "best-practices",
                    "Found mixed content: HTTPS page loads HTTP assets",
                );
            }
        }

        for link in document.select(&TARGET_BLANK_SELECTOR) {
            let rel = link.value().attr("rel").unwrap_or("").to_ascii_lowercase();
            let has_protection = rel
                .split_ascii_whitespace()
                .any(|token| token == "noopener" || token == "noreferrer");

            if !has_protection {
                report.add_issue(
                    Severity::Warning,
                    "best-practices",
                    "Found target=\"_blank\" link without rel=noopener/noreferrer",
                );
            }
        }

        let blocking_scripts = document
            .select(&SCRIPT_SELECTOR)
            .filter(|script| {
                script.value().attr("async").is_none() && script.value().attr("defer").is_none()
            })
            .count();

        if blocking_scripts > 0 {
            report.add_issue(
                Severity::Info,
                "best-practices",
                &format!(
                    "Found {} external script(s) without async/defer",
                    blocking_scripts
                ),
            );
        }

        let has_meta_refresh = document.select(&META_SELECTOR).any(|meta| {
            let http_equiv = meta.value().attr("http-equiv").unwrap_or("");
            let content = meta.value().attr("content").unwrap_or("").trim();
            http_equiv.eq_ignore_ascii_case("refresh") && !content.is_empty()
        });

        if has_meta_refresh {
            report.add_issue(
                Severity::Info,
                "best-practices",
                "Found meta refresh redirect; prefer HTTP redirects",
            );
        }
    }

    fn analyze_performance(report: &mut SeoReport, snapshot: &Snapshot) {
        let nav_start = snapshot.performance_timing.navigation_start;
        if nav_start == 0 {
            return;
        }

        if let Some(ttfb) =
            Self::timing_delta(snapshot.performance_timing.response_start, nav_start)
        {
            if ttfb > 1800 {
                report.add_issue(
                    Severity::Error,
                    "performance",
                    &format!("High TTFB detected ({}ms)", ttfb),
                );
            } else if ttfb > 800 {
                report.add_issue(
                    Severity::Warning,
                    "performance",
                    &format!("Slow TTFB detected ({}ms)", ttfb),
                );
            }
        }

        if let Some(fcp) = Self::timing_delta(
            snapshot.performance_timing.first_contentful_paint,
            nav_start,
        ) {
            if fcp > 5000 {
                report.add_issue(
                    Severity::Error,
                    "performance",
                    &format!("Very slow first contentful paint ({}ms)", fcp),
                );
            } else if fcp > 3000 {
                report.add_issue(
                    Severity::Warning,
                    "performance",
                    &format!("Slow first contentful paint ({}ms)", fcp),
                );
            }
        }

        if let Some(load) = Self::timing_delta(snapshot.performance_timing.load_complete, nav_start)
        {
            if load > 10000 {
                report.add_issue(
                    Severity::Warning,
                    "performance",
                    &format!("Page load completed very late ({}ms)", load),
                );
            } else if load > 5000 {
                report.add_issue(
                    Severity::Info,
                    "performance",
                    &format!("Page load completed after {}ms", load),
                );
            }
        }
    }

    fn analyze_media_optimization(report: &mut SeoReport, document: &Html) {
        let mut total_images = 0usize;
        let mut lazy_images = 0usize;
        let mut unsized_images = 0usize;

        for img in document.select(&IMG_SELECTOR) {
            if img.value().attr("src").unwrap_or("").trim().is_empty() {
                continue;
            }

            total_images += 1;

            if img
                .value()
                .attr("loading")
                .map(|v| v.eq_ignore_ascii_case("lazy"))
                .unwrap_or(false)
            {
                lazy_images += 1;
            }

            let has_width = img.value().attr("width").is_some();
            let has_height = img.value().attr("height").is_some();
            if !has_width || !has_height {
                unsized_images += 1;
            }
        }

        if total_images >= 4 && lazy_images == 0 {
            report.add_issue(
                Severity::Info,
                "performance",
                "Page has multiple images but none use loading=lazy",
            );
        }

        if unsized_images > 0 {
            report.add_issue(
                Severity::Warning,
                "performance",
                &format!(
                    "Found {} image(s) without explicit width/height attributes",
                    unsized_images
                ),
            );
        }
    }

    fn analyze_accessibility_styles(report: &mut SeoReport, snapshot: &Snapshot) {
        if snapshot.computed_styles.is_empty() {
            return;
        }

        let mut low_contrast_count = 0usize;

        for style in &snapshot.computed_styles {
            let Some(fg) = style.color.as_deref().and_then(Self::parse_color) else {
                continue;
            };
            let Some(bg) = style
                .background_color
                .as_deref()
                .and_then(Self::parse_color)
            else {
                continue;
            };

            let ratio = Self::contrast_ratio(fg, bg);
            let min_ratio =
                if Self::is_large_text(style.font_size.as_deref(), style.font_weight.as_deref()) {
                    3.0
                } else {
                    4.5
                };

            if ratio < min_ratio {
                low_contrast_count += 1;
            }
        }

        if low_contrast_count > 0 {
            report.add_issue(
                Severity::Warning,
                "accessibility",
                &format!(
                    "Found {} element(s) with insufficient text contrast",
                    low_contrast_count
                ),
            );
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

    fn get_link_rels(document: &Html, rel: &str) -> Vec<String> {
        document
            .select(&LINK_SELECTOR)
            .filter(|el| {
                el.value()
                    .attr("rel")
                    .map(|rels| {
                        rels.split_ascii_whitespace()
                            .any(|token| token.eq_ignore_ascii_case(rel))
                    })
                    .unwrap_or(false)
            })
            .filter_map(|el| el.value().attr("href"))
            .map(std::string::ToString::to_string)
            .collect()
    }

    fn is_expected_hreflang_code(hreflang: &str) -> bool {
        if hreflang.eq_ignore_ascii_case("x-default") {
            return true;
        }

        let mut parts = hreflang.split('-');
        let Some(lang) = parts.next() else {
            return false;
        };

        if lang.len() < 2 || lang.len() > 3 || !lang.chars().all(|c| c.is_ascii_alphabetic()) {
            return false;
        }

        parts.all(|part| {
            let len = part.len();
            (2..=8).contains(&len) && part.chars().all(|c| c.is_ascii_alphanumeric())
        })
    }

    fn timing_delta(value: Option<u64>, navigation_start: u64) -> Option<u64> {
        value.and_then(|v| v.checked_sub(navigation_start))
    }

    fn find_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
        haystack
            .to_ascii_lowercase()
            .find(&needle.to_ascii_lowercase())
    }

    fn parse_color(value: &str) -> Option<(u8, u8, u8)> {
        let v = value.trim();
        if let Some(hex) = v.strip_prefix('#') {
            return match hex.len() {
                3 => {
                    let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                    let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                    let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                    Some((r, g, b))
                }
                6 => {
                    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                    Some((r, g, b))
                }
                _ => None,
            };
        }

        let open = v.find('(')?;
        let close = v.rfind(')')?;
        if close <= open {
            return None;
        }

        let body = &v[open + 1..close];
        let channels: Vec<&str> = body.split(',').collect();
        if channels.len() < 3 {
            return None;
        }

        let r = channels[0].trim().parse::<u8>().ok()?;
        let g = channels[1].trim().parse::<u8>().ok()?;
        let b = channels[2].trim().parse::<u8>().ok()?;
        Some((r, g, b))
    }

    fn is_large_text(font_size: Option<&str>, font_weight: Option<&str>) -> bool {
        let size = font_size
            .and_then(|v| v.trim_end_matches("px").trim().parse::<f64>().ok())
            .unwrap_or(0.0);
        let weight = font_weight
            .and_then(|v| v.trim().parse::<u16>().ok())
            .unwrap_or(400);

        size >= 18.0 || (size >= 14.0 && weight >= 700)
    }

    fn relative_luminance((r, g, b): (u8, u8, u8)) -> f64 {
        fn to_linear(channel: u8) -> f64 {
            let c = channel as f64 / 255.0;
            if c <= 0.039_28 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }

        let r = to_linear(r);
        let g = to_linear(g);
        let b = to_linear(b);
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    fn contrast_ratio(fg: (u8, u8, u8), bg: (u8, u8, u8)) -> f64 {
        let l1 = Self::relative_luminance(fg);
        let l2 = Self::relative_luminance(bg);
        let (lighter, darker) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
        (lighter + 0.05) / (darker + 0.05)
    }

    fn header_contains_token(
        headers: &std::collections::HashMap<String, String>,
        header_name: &str,
        token: &str,
    ) -> bool {
        headers
            .get(header_name)
            .map(|value| {
                value
                    .split(',')
                    .map(|part| part.trim().to_ascii_lowercase())
                    .any(|part| part == token)
            })
            .unwrap_or(false)
    }

    fn should_check_secure_transport(page_url: &str) -> bool {
        let Ok(url) = Url::parse(page_url) else {
            return false;
        };

        let Some(host) = url.host_str() else {
            return false;
        };

        !host.eq_ignore_ascii_case("localhost")
            && host != "127.0.0.1"
            && host != "::1"
            && !host.ends_with(".local")
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
            main_resource_network: crate::snapshot::MainResourceNetwork::default(),
            network_requests: Vec::new(),
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
    fn analyze_invalid_hreflang_and_unlabeled_controls() {
        let html = r#"
            <html><head>
                <meta charset="utf-8">
                <title>Accessibility and Hreflang Test</title>
                <link rel="alternate" hreflang="english" href="/relative">
            </head>
            <body>
                <h1>Title</h1>
                <form>
                    <input id="email" type="email">
                </form>
            </body></html>
        "#;

        let snapshot = snapshot_with_html(html);
        let report = SeoAnalyzer::analyze(&snapshot);

        assert!(report
            .issues
            .iter()
            .any(|i| i.category == "hreflang" && i.message.contains("invalid hreflang")));
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.category == "accessibility"
                    && i.message.contains("unlabeled form control"))
        );
    }

    #[test]
    fn analyze_best_practices_and_performance_heuristics() {
        let html = r#"
            <html><head>
                <meta charset="utf-8">
                <title>Best Practices Test</title>
                <script src="/app.js"></script>
            </head>
            <body>
                <h1>Title</h1>
                <a href="https://example.com" target="_blank">external</a>
            </body></html>
        "#;

        let mut snapshot = snapshot_with_html(html);
        snapshot.url = "https://example.com/page".to_string();
        snapshot.performance_timing.navigation_start = 100;
        snapshot.performance_timing.response_start = Some(2300);
        snapshot.performance_timing.first_contentful_paint = Some(4200);
        snapshot.performance_timing.load_complete = Some(12050);
        snapshot
            .referenced_assets
            .stylesheets
            .push("http://cdn.example.com/site.css".to_string());

        let report = SeoAnalyzer::analyze(&snapshot);

        assert!(report
            .issues
            .iter()
            .any(|i| i.category == "best-practices" && i.message.contains("mixed content")));
        assert!(report.issues.iter().any(|i| {
            i.category == "best-practices" && i.message.contains("target=\"_blank\"")
        }));
        assert!(report
            .issues
            .iter()
            .any(|i| i.category == "performance" && i.message.contains("TTFB")));
    }

    #[test]
    fn analyze_crawlability_and_document_basics_checks() {
        let mut html = String::new();
        html.push_str("<html><head>");
        html.push_str(&" ".repeat(1100));
        html.push_str("<meta charset=\"utf-8\">");
        html.push_str("<meta name=\"robots\" content=\"noindex, nofollow\">");
        html.push_str("<title>Crawlability Test</title>");
        html.push_str("</head><body><h1>Title</h1></body></html>");

        let snapshot = snapshot_with_html(&html);
        let report = SeoAnalyzer::analyze(&snapshot);

        assert!(report.issues.iter().any(|i| {
            i.category == "best-practices" && i.message.contains("missing an HTML5 doctype")
        }));
        assert!(report
            .issues
            .iter()
            .any(|i| i.category == "meta" && i.message.contains("first 1024 bytes")));
        assert!(report
            .issues
            .iter()
            .any(|i| { i.category == "seo" && i.message.contains("blocking indexing directive") }));
    }

    #[test]
    fn analyze_media_optimization_checks() {
        let html = r#"
            <!doctype html>
            <html><head><meta charset="utf-8"><title>Media Test</title></head>
            <body>
                <h1>Media</h1>
                <img src="1.jpg">
                <img src="2.jpg">
                <img src="3.jpg" width="400">
                <img src="4.jpg" height="300">
            </body></html>
        "#;

        let snapshot = snapshot_with_html(html);
        let report = SeoAnalyzer::analyze(&snapshot);

        assert!(report.issues.iter().any(|i| {
            i.category == "performance" && i.message.contains("none use loading=lazy")
        }));
        assert!(report.issues.iter().any(|i| {
            i.category == "performance" && i.message.contains("without explicit width/height")
        }));
    }

    #[test]
    fn analyze_contrast_and_meta_refresh_checks() {
        let html = r#"
            <!doctype html>
            <html><head>
                <meta charset="utf-8">
                <meta http-equiv="refresh" content="0;url=/target">
                <title>Contrast Test</title>
            </head>
            <body><h1>Title</h1></body></html>
        "#;

        let mut snapshot = snapshot_with_html(html);
        snapshot
            .computed_styles
            .push(crate::snapshot::ComputedStyle {
                selector: "p".to_string(),
                color: Some("rgb(120, 120, 120)".to_string()),
                background_color: Some("rgb(130, 130, 130)".to_string()),
                font_size: Some("12px".to_string()),
                font_weight: Some("400".to_string()),
            });

        let report = SeoAnalyzer::analyze(&snapshot);

        assert!(report.issues.iter().any(|i| {
            i.category == "accessibility" && i.message.contains("insufficient text contrast")
        }));
        assert!(report.issues.iter().any(|i| {
            i.category == "best-practices" && i.message.contains("meta refresh redirect")
        }));
    }

    #[test]
    fn analyze_network_header_based_checks() {
        let html = r#"
            <!doctype html>
            <html><head><meta charset="utf-8"><title>Network Test</title></head>
            <body><h1>Title</h1></body></html>
        "#;

        let mut snapshot = snapshot_with_html(html);
        snapshot.url = "https://example.com/".to_string();
        snapshot.main_resource_network.status_code = Some(404);
        snapshot
            .main_resource_network
            .headers
            .insert("x-robots-tag".to_string(), "noindex".to_string());
        snapshot.main_resource_network.headers.insert(
            "content-type".to_string(),
            "text/html; charset=utf-8".to_string(),
        );

        let report = SeoAnalyzer::analyze(&snapshot);

        assert!(report.issues.iter().any(|i| {
            i.category == "seo" && i.message.contains("unsuccessful status code 404")
        }));
        assert!(report
            .issues
            .iter()
            .any(|i| i.category == "seo" && i.message.contains("X-Robots-Tag blocks indexing")));
        assert!(report.issues.iter().any(|i| {
            i.category == "performance" && i.message.contains("missing content-encoding")
        }));
        assert!(report.issues.iter().any(|i| {
            i.category == "best-practices" && i.message.contains("Missing security header")
        }));
    }

    #[test]
    fn analyze_cdp_network_event_based_checks() {
        let html = r#"
            <!doctype html>
            <html><head><meta charset="utf-8"><title>CDP Network Test</title></head>
            <body><h1>Title</h1></body></html>
        "#;

        let mut snapshot = snapshot_with_html(html);
        snapshot.url = "https://example.com/".to_string();
        snapshot
            .network_requests
            .push(crate::snapshot::NetworkRequestRecord {
                url: "http://cdn.example.com/app.js".to_string(),
                resource_type: Some("Script".to_string()),
                status_code: Some(200),
                encoded_data_length: Some(12_345.0),
                from_cache: Some(false),
                was_redirect: Some(false),
                failed: Some(false),
                failure_text: None,
                content_encoding: None,
                mime_type: Some("application/javascript".to_string()),
            });
        snapshot
            .network_requests
            .push(crate::snapshot::NetworkRequestRecord {
                url: "https://example.com/redirected".to_string(),
                resource_type: Some("Document".to_string()),
                status_code: Some(301),
                encoded_data_length: Some(900.0),
                from_cache: Some(false),
                was_redirect: Some(true),
                failed: Some(false),
                failure_text: None,
                content_encoding: Some("gzip".to_string()),
                mime_type: Some("text/html".to_string()),
            });
        snapshot
            .network_requests
            .push(crate::snapshot::NetworkRequestRecord {
                url: "https://example.com/fail.css".to_string(),
                resource_type: Some("Stylesheet".to_string()),
                status_code: None,
                encoded_data_length: None,
                from_cache: Some(false),
                was_redirect: Some(false),
                failed: Some(true),
                failure_text: Some("net::ERR_CONNECTION_RESET".to_string()),
                content_encoding: None,
                mime_type: Some("text/css".to_string()),
            });

        let report = SeoAnalyzer::analyze(&snapshot);

        assert!(report.issues.iter().any(|i| {
            i.category == "best-practices" && i.message.contains("mixed content request")
        }));
        assert!(report
            .issues
            .iter()
            .any(|i| i.category == "performance" && i.message.contains("redirect")));
        assert!(report
            .issues
            .iter()
            .any(|i| i.category == "performance" && i.message.contains("without compression")));
        assert!(report.issues.iter().any(
            |i| i.category == "best-practices" && i.message.contains("failed network request")
        ));
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
