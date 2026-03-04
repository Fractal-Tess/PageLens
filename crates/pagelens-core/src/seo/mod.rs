//! SEO analysis — Meta tags, headings, images, structured data, etc.

use crate::seo::helpers::*;
use crate::seo::scoring::calculate_score;
use crate::snapshot::Snapshot;
use scraper::{Html, Selector};
use std::sync::LazyLock;

mod helpers;
mod scoring;
mod types;

pub use helpers::{get_meta_content, get_meta_name_or_property};
pub use types::{
    HeadingsInfo, ImageInfo, Issue, MetaInfo, OpenGraphInfo, ScoreConfig, SeoReport, Severity,
    StructuredData, TwitterCardInfo,
};

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

        // Favicon
        report.favicon_url = snapshot.favicon_url.clone();
        Self::analyze_favicon(&mut report, snapshot);

        // Calculate final score
        report.score = calculate_score(&report, snapshot, score_config);

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
        report.meta.viewport = get_meta_content(document, "viewport").is_some();
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
        if let Some(desc) = get_meta_content(document, "description") {
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
        report.open_graph.title = get_meta_property(document, "og:title");
        report.open_graph.description = get_meta_property(document, "og:description");
        report.open_graph.og_type = get_meta_property(document, "og:type");
        report.open_graph.url = get_meta_property(document, "og:url");
        report.open_graph.image = get_meta_property(document, "og:image");

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
        report.twitter_card.card = get_meta_name_or_property(document, "twitter:card");
        report.twitter_card.title = get_meta_name_or_property(document, "twitter:title");
        report.twitter_card.description =
            get_meta_name_or_property(document, "twitter:description");
        report.twitter_card.image = get_meta_name_or_property(document, "twitter:image");
    }

    /// Analyze canonical URL.
    fn analyze_canonical(report: &mut SeoReport, document: &Html, page_url: &str) {
        // Look for <link rel="canonical" href="...">
        let canonical_links = get_link_rels(document, "canonical");

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

            if url::Url::parse(primary).is_err() {
                report.add_issue(
                    Severity::Warning,
                    "canonical",
                    "Canonical URL is not absolute",
                );
            }

            if let (Ok(page), Ok(canonical)) = (url::Url::parse(page_url), url::Url::parse(primary))
            {
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
            if href.is_empty() || url::Url::parse(href).is_err() {
                invalid_count += 1;
                continue;
            }

            if !is_expected_hreflang_code(hreflang) {
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
            report.add_issue(
                Severity::Warning,
                "headings",
                &format!(
                    "Multiple H1 tags found ({}). Best practice is to have exactly one H1 per page.",
                    report.headings.h1_count
                ),
            );
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

        if let Some(charset_index) = find_case_insensitive(&snapshot.html, "<meta charset") {
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
        use crate::seo::helpers::header_contains_token;

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

        if header_contains_token(&network.headers, "x-robots-tag", "noindex")
            || header_contains_token(&network.headers, "x-robots-tag", "none")
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

            if let Some(hsts) = network.headers.get("strict-transport-security") {
                let hsts_lower = hsts.to_ascii_lowercase();
                if let Some(max_age) = crate::seo::helpers::extract_hsts_max_age(&hsts_lower) {
                    if max_age < 15_552_000 {
                        report.add_issue(
                            Severity::Warning,
                            "best-practices",
                            "HSTS max-age is too low; prefer at least 15552000 seconds",
                        );
                    }
                }
                if !hsts_lower.contains("includesubdomains") {
                    report.add_issue(
                        Severity::Info,
                        "best-practices",
                        "HSTS header is missing includeSubDomains",
                    );
                }
            }

            if let Some(xcto) = network.headers.get("x-content-type-options") {
                if !xcto.eq_ignore_ascii_case("nosniff") {
                    report.add_issue(
                        Severity::Warning,
                        "best-practices",
                        "X-Content-Type-Options should be set to nosniff",
                    );
                }
            }

            if let Some(xfo) = network.headers.get("x-frame-options") {
                let allowed =
                    xfo.eq_ignore_ascii_case("deny") || xfo.eq_ignore_ascii_case("sameorigin");
                if !allowed {
                    report.add_issue(
                        Severity::Warning,
                        "best-practices",
                        "X-Frame-Options should be DENY or SAMEORIGIN",
                    );
                }
            }

            if let Some(referrer_policy) = network.headers.get("referrer-policy") {
                if referrer_policy.eq_ignore_ascii_case("unsafe-url") {
                    report.add_issue(
                        Severity::Warning,
                        "best-practices",
                        "Referrer-Policy is unsafe-url; prefer stricter policy",
                    );
                }
            }

            if let Some(csp) = network.headers.get("content-security-policy") {
                let csp_lower = csp.to_ascii_lowercase();
                if csp_lower.contains("'unsafe-inline'") || csp_lower.contains("'unsafe-eval'") {
                    report.add_issue(
                        Severity::Warning,
                        "best-practices",
                        "CSP contains unsafe-inline/unsafe-eval directives",
                    );
                }
            }
        }
    }

    fn analyze_network_request_records(report: &mut SeoReport, snapshot: &Snapshot) {
        if snapshot.network_requests.is_empty() {
            return;
        }

        let min_start = snapshot
            .network_requests
            .iter()
            .filter_map(|request| request.request_start_time_s)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let max_end = snapshot
            .network_requests
            .iter()
            .filter_map(|request| request.end_time_s)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        if let (Some(start), Some(end)) = (min_start, max_end) {
            if end >= start {
                report.add_issue(
                    Severity::Info,
                    "performance",
                    &format!(
                        "Network waterfall window spans {}ms across {} request(s)",
                        ((end - start) * 1000.0).round() as u64,
                        snapshot.network_requests.len()
                    ),
                );
            }
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

        let mut redirect_hops: Vec<(String, String, Option<u16>, Option<f64>, Option<f64>)> =
            snapshot
                .network_requests
                .iter()
                .filter_map(|request| {
                    if !request.was_redirect.unwrap_or(false) {
                        return None;
                    }

                    let from = request.redirect_from_url.clone()?;
                    Some((
                        from,
                        request.url.clone(),
                        request.redirect_status_code,
                        request.request_start_time_s,
                        request.end_time_s,
                    ))
                })
                .collect();

        redirect_hops.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal));

        if !redirect_hops.is_empty() {
            let mut chain_urls: Vec<String> = vec![redirect_hops[0].0.clone()];
            for (_, to, _, _, _) in &redirect_hops {
                if chain_urls.last().map(|v| v != to).unwrap_or(true) {
                    chain_urls.push(to.clone());
                }
            }

            let hops = redirect_hops.len();
            let start = redirect_hops.first().and_then(|hop| hop.3);
            let end = redirect_hops.last().and_then(|hop| hop.4);
            let duration_suffix = match (start, end) {
                (Some(s), Some(e)) if e >= s => {
                    format!(" in {}ms", ((e - s) * 1000.0).round() as u64)
                }
                _ => String::new(),
            };
            let status_chain = redirect_hops
                .iter()
                .filter_map(|(_, _, status, _, _)| status.map(|s| s.to_string()))
                .collect::<Vec<_>>()
                .join(" -> ");

            report.add_issue(
                Severity::Info,
                "performance",
                &format!(
                    "Redirect chain ({} hop(s){}): {}{}",
                    hops,
                    duration_suffix,
                    chain_urls.join(" -> "),
                    if status_chain.is_empty() {
                        String::new()
                    } else {
                        format!(" [status: {}]", status_chain)
                    }
                ),
            );

            if hops > 1 {
                report.add_issue(
                    Severity::Warning,
                    "performance",
                    &format!(
                        "Redirect chain has {} hops; consider reducing to a single redirect",
                        hops
                    ),
                );
            }
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

        let is_critical_type = |resource_type: Option<&str>| {
            matches!(
                resource_type,
                Some("Document") | Some("Script") | Some("Stylesheet") | Some("Font")
            )
        };

        let critical_candidate = snapshot
            .network_requests
            .iter()
            .filter(|request| {
                is_critical_type(request.resource_type.as_deref())
                    && !request.from_cache.unwrap_or(false)
                    && !request.failed.unwrap_or(false)
            })
            .max_by(|a, b| {
                let a_ms = a
                    .duration_ms
                    .or_else(|| match (a.request_start_time_s, a.end_time_s) {
                        (Some(start), Some(end)) if end >= start => Some((end - start) * 1000.0),
                        _ => None,
                    });
                let b_ms = b
                    .duration_ms
                    .or_else(|| match (b.request_start_time_s, b.end_time_s) {
                        (Some(start), Some(end)) if end >= start => Some((end - start) * 1000.0),
                        _ => None,
                    });
                a_ms.partial_cmp(&b_ms).unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some(candidate) = critical_candidate {
            let duration_ms = candidate.duration_ms.or_else(|| {
                match (candidate.request_start_time_s, candidate.end_time_s) {
                    (Some(start), Some(end)) if end >= start => Some((end - start) * 1000.0),
                    _ => None,
                }
            });

            if let Some(duration_ms) = duration_ms {
                report.add_issue(
                    Severity::Info,
                    "performance",
                    &format!(
                        "Critical path candidate: {} took {}ms",
                        candidate.url,
                        duration_ms.round() as u64
                    ),
                );
            }
        }
    }

    fn analyze_best_practices(report: &mut SeoReport, document: &Html, snapshot: &Snapshot) {
        if crate::seo::helpers::should_check_secure_transport(&snapshot.url)
            && snapshot.url.starts_with("http://")
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
            crate::seo::helpers::timing_delta(snapshot.performance_timing.response_start, nav_start)
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

        if let Some(fcp) = crate::seo::helpers::timing_delta(
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

        if let Some(load) =
            crate::seo::helpers::timing_delta(snapshot.performance_timing.load_complete, nav_start)
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
        use crate::seo::helpers::{contrast_ratio, is_large_text, parse_color};

        if snapshot.computed_styles.is_empty() {
            return;
        }

        let mut low_contrast_count = 0usize;

        for style in &snapshot.computed_styles {
            let Some(fg) = style.color.as_deref().and_then(parse_color) else {
                continue;
            };
            let Some(bg) = style.background_color.as_deref().and_then(parse_color) else {
                continue;
            };

            let ratio = contrast_ratio(fg, bg);
            let min_ratio =
                if is_large_text(style.font_size.as_deref(), style.font_weight.as_deref()) {
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

    /// Analyze favicon presence and quality.
    fn analyze_favicon(report: &mut SeoReport, snapshot: &Snapshot) {
        if snapshot.favicon_url.is_none() {
            report.add_issue(Severity::Warning, "favicon", "No favicon declared");
        }
    }
}
