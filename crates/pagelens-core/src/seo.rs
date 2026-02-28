//! SEO analysis — Meta tags, headings, images, structured data, etc.

use crate::prelude::*;
use crate::snapshot::Snapshot;

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
        let mut report = SeoReport::new();
        let html = &snapshot.html;

        // Analyze meta tags
        Self::analyze_meta(&mut report, html);

        // Analyze Open Graph
        Self::analyze_open_graph(&mut report, html);

        // Analyze Twitter Cards
        Self::analyze_twitter_cards(&mut report, html);

        // Analyze canonical URL
        Self::analyze_canonical(&mut report, html);

        // Analyze headings
        Self::analyze_headings(&mut report, html);

        // Analyze images
        Self::analyze_images(&mut report, html);

        // Analyze structured data
        Self::analyze_structured_data(&mut report, html);

        // Calculate final score
        report.score = Self::calculate_score(&report);

        report
    }

    /// Analyze meta tags.
    fn analyze_meta(report: &mut SeoReport, html: &str) {
        // Check charset
        report.meta.charset = html.contains(r#"charset="#) || html.contains(r#"charset="#);
        if !report.meta.charset {
            report.add_issue(Severity::Error, "meta", "Missing charset declaration");
        }

        // Check viewport
        report.meta.viewport = html.contains(r#"name="viewport"#) || html.contains(r#"name='viewport'"#);
        if !report.meta.viewport {
            report.add_issue(Severity::Warning, "meta", "Missing viewport meta tag (important for mobile SEO)");
        }

        // Extract title
        if let Some(title) = Self::extract_tag_content(html, "title") {
            let title_len = title.len();
            report.meta.title = Some(title.clone());
            
            if title_len == 0 {
                report.add_issue(Severity::Error, "meta", "Title tag is empty");
            } else if title_len < 10 {
                report.add_issue(Severity::Warning, "meta", &format!("Title is too short ({} chars)", title_len));
            } else if title_len > 60 {
                report.add_issue(Severity::Warning, "meta", &format!("Title may be truncated in search results ({} chars)", title_len));
            }
        } else {
            report.add_issue(Severity::Error, "meta", "Missing title tag");
        }

        // Extract description
        if let Some(desc) = Self::extract_meta_content(html, "description") {
            let desc_len = desc.len();
            report.meta.description = Some(desc.clone());
            
            if desc_len == 0 {
                report.add_issue(Severity::Warning, "meta", "Meta description is empty");
            } else if desc_len < 50 {
                report.add_issue(Severity::Info, "meta", &format!("Description is short ({} chars)", desc_len));
            } else if desc_len > 160 {
                report.add_issue(Severity::Info, "meta", &format!("Description may be truncated ({} chars)", desc_len));
            }
        } else {
            report.add_issue(Severity::Warning, "meta", "Missing meta description");
        }

        // Extract language
        if let Some(lang) = Self::extract_attr(html, "html", "lang") {
            report.meta.language = Some(lang);
        } else {
            report.add_issue(Severity::Info, "meta", "Missing lang attribute on <html> tag");
        }
    }

    /// Analyze Open Graph tags.
    fn analyze_open_graph(report: &mut SeoReport, html: &str) {
        report.open_graph.title = Self::extract_meta_property(html, "og:title");
        report.open_graph.description = Self::extract_meta_property(html, "og:description");
        report.open_graph.og_type = Self::extract_meta_property(html, "og:type");
        report.open_graph.url = Self::extract_meta_property(html, "og:url");
        report.open_graph.image = Self::extract_meta_property(html, "og:image");

        // Check for missing OG tags
        if report.open_graph.title.is_none() {
            report.add_issue(Severity::Info, "open-graph", "Missing og:title");
        }
        if report.open_graph.description.is_none() {
            report.add_issue(Severity::Info, "open-graph", "Missing og:description");
        }
        if report.open_graph.image.is_none() {
            report.add_issue(Severity::Info, "open-graph", "Missing og:image (important for social sharing)");
        }
    }

    /// Analyze Twitter Card tags.
    fn analyze_twitter_cards(report: &mut SeoReport, html: &str) {
        report.twitter_card.card = Self::extract_meta_name(html, "twitter:card");
        report.twitter_card.title = Self::extract_meta_name(html, "twitter:title");
        report.twitter_card.description = Self::extract_meta_name(html, "twitter:description");
        report.twitter_card.image = Self::extract_meta_name(html, "twitter:image");
    }

    /// Analyze canonical URL.
    fn analyze_canonical(report: &mut SeoReport, html: &str) {
        // Look for <link rel="canonical" href="...">
        if let Some(href) = Self::extract_link_rel(html, "canonical") {
            report.canonical_url = Some(href);
        } else {
            report.add_issue(Severity::Info, "canonical", "Missing canonical URL (recommended for SEO)");
        }
    }

    /// Analyze heading structure.
    fn analyze_headings(report: &mut SeoReport, html: &str) {
        // Count headings
        report.headings.h1_count = Self::count_tags(html, "h1");
        report.headings.h2_count = Self::count_tags(html, "h2");
        report.headings.h3_count = Self::count_tags(html, "h3");
        report.headings.h4_count = Self::count_tags(html, "h4");
        report.headings.h5_count = Self::count_tags(html, "h5");
        report.headings.h6_count = Self::count_tags(html, "h6");

        // Check for multiple H1s
        if report.headings.h1_count > 1 {
            report.add_issue(Severity::Warning, "headings", 
                &format!("Multiple H1 tags found ({}). Best practice is to have exactly one H1 per page.", 
                    report.headings.h1_count));
        }

        if report.headings.h1_count == 0 {
            report.add_issue(Severity::Error, "headings", "No H1 tag found");
        }

        // Build heading structure
        report.headings.structure = Self::extract_heading_structure(html);

        // Check for skipped heading levels
        let structure_clone = report.headings.structure.clone();
        if !structure_clone.is_empty() {
            let mut prev_level = 0u8;
            for heading in &structure_clone {
                if let Ok(level) = heading[1..].parse::<u8>() {
                    if level > prev_level + 1 && prev_level > 0 {
                        report.add_issue(Severity::Info, "headings", 
                            &format!("Heading level skipped: H{} follows H{}", level, prev_level));
                    }
                    prev_level = level;
                }
            }
        }
    }

    /// Analyze images.
    fn analyze_images(report: &mut SeoReport, html: &str) {
        // Extract img tags
        let img_regex = regex::Regex::new(r#"<img[^>]*>"#).unwrap();
        let src_regex = regex::Regex::new(r#"src=["']([^"']+)["']"#).unwrap();
        let alt_regex = regex::Regex::new(r#"alt=["']([^"]*)["']"#).unwrap();

        for img_cap in img_regex.captures_iter(html) {
            let img_tag = &img_cap[0];
            
            let src = src_regex.captures(img_tag)
                .map(|c| c[1].to_string())
                .unwrap_or_default();
            
            let alt = alt_regex.captures(img_tag)
                .map(|c| c[1].to_string());
            
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
                    report.add_issue(Severity::Warning, "images", 
                        &format!("Image missing alt text: {}", &src[..src.len().min(50)]));
                } else if alt.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                    report.add_issue(Severity::Warning, "images", 
                        &format!("Image has empty alt text: {}", &src[..src.len().min(50)]));
                }
            }
        }
    }

    /// Analyze structured data (JSON-LD).
    fn analyze_structured_data(report: &mut SeoReport, html: &str) {
        // Find JSON-LD script tags
        let jsonld_regex = regex::Regex::new(r#"<script[^>]*type=["']application/ld\+json["'][^>]*>(.*?)</script>"#).unwrap();
        
        for cap in jsonld_regex.captures_iter(html) {
            let json_content = &cap[1];
            
            // Try to extract @type
            let type_regex = regex::Regex::new(r#""@type"\s*:\s*"([^"]+)""#).unwrap();
            let schema_type = type_regex.captures(json_content)
                .map(|c| c[1].to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            
            report.structured_data.push(StructuredData {
                schema_type,
                raw: json_content.to_string(),
            });
        }
    }

    /// Calculate overall SEO score.
    fn calculate_score(report: &SeoReport) -> f64 {
        let mut score = 100.0;
        
        // Deduct points for errors
        let error_count = report.issues.iter().filter(|i| i.severity == Severity::Error).count();
        score -= error_count as f64 * 15.0;
        
        // Deduct points for warnings
        let warning_count = report.issues.iter().filter(|i| i.severity == Severity::Warning).count();
        score -= warning_count as f64 * 5.0;
        
        // Deduct points for info (minor)
        let info_count = report.issues.iter().filter(|i| i.severity == Severity::Info).count();
        score -= info_count as f64 * 1.0;
        
        // Bonus for good practices
        if report.meta.title.is_some() {
            score += 5.0;
        }
        if report.meta.description.is_some() {
            score += 5.0;
        }
        if report.canonical_url.is_some() {
            score += 2.0;
        }
        if !report.structured_data.is_empty() {
            score += 5.0;
        }
        
        score.clamp(0.0, 100.0)
    }

    // =========================================================================
    // Helper functions
    // =========================================================================

    /// Extract content from a tag (e.g., <title>content</title>).
    fn extract_tag_content(html: &str, tag: &str) -> Option<String> {
        let pattern = format!("<{}[^>]*>([^<]+)</{}>", tag, tag);
        let regex = regex::Regex::new(&pattern).ok()?;
        regex.captures(html).map(|c| c[1].trim().to_string())
    }

    /// Extract content from a meta tag by name.
    fn extract_meta_content(html: &str, name: &str) -> Option<String> {
        // Try name attribute
        let pattern_name = format!(r#"<meta[^>]*name=["']{}["'][^>]*content=["']([^"']*)["']"#, regex::escape(name));
        let regex_name = regex::Regex::new(&pattern_name).ok()?;
        if let Some(cap) = regex_name.captures(html) {
            return Some(cap[1].to_string());
        }
        
        // Try content first, then name (different attribute order)
        let pattern_content = format!(r#"<meta[^>]*content=["']([^"']*)["'][^>]*name=["']{}["']"#, regex::escape(name));
        let regex_content = regex::Regex::new(&pattern_content).ok()?;
        regex_content.captures(html).map(|c| c[1].to_string())
    }

    /// Extract content from a meta tag by property (for Open Graph).
    fn extract_meta_property(html: &str, property: &str) -> Option<String> {
        let pattern = format!(r#"<meta[^>]*property=["']{}["'][^>]*content=["']([^"']*)["']"#, regex::escape(property));
        let regex = regex::Regex::new(&pattern).ok()?;
        if let Some(cap) = regex.captures(html) {
            return Some(cap[1].to_string());
        }
        
        // Try reversed attribute order
        let pattern2 = format!(r#"<meta[^>]*content=["']([^"']*)["'][^>]*property=["']{}["']"#, regex::escape(property));
        let regex2 = regex::Regex::new(&pattern2).ok()?;
        regex2.captures(html).map(|c| c[1].to_string())
    }

    /// Extract content from a meta tag by name.
    fn extract_meta_name(html: &str, name: &str) -> Option<String> {
        Self::extract_meta_content(html, name)
    }

    /// Extract href from a link tag with specific rel.
    fn extract_link_rel(html: &str, rel: &str) -> Option<String> {
        let pattern = format!(r#"<link[^>]*rel=["']{}["'][^>]*href=["']([^"']*)["']"#, regex::escape(rel));
        let regex = regex::Regex::new(&pattern).ok()?;
        if let Some(cap) = regex.captures(html) {
            return Some(cap[1].to_string());
        }
        
        // Try reversed attribute order
        let pattern2 = format!(r#"<link[^>]*href=["']([^"']*)["'][^>]*rel=["']{}["']"#, regex::escape(rel));
        let regex2 = regex::Regex::new(&pattern2).ok()?;
        regex2.captures(html).map(|c| c[1].to_string())
    }

    /// Extract an attribute value from a tag.
    fn extract_attr(html: &str, tag: &str, attr: &str) -> Option<String> {
        let pattern = format!(r#"<{}[^>]*{}=["']([^"']*)["']"#, tag, attr);
        let regex = regex::Regex::new(&pattern).ok()?;
        regex.captures(html).map(|c| c[1].to_string())
    }

    /// Count occurrences of a tag.
    fn count_tags(html: &str, tag: &str) -> usize {
        let pattern = format!(r"<{}[\s>]", tag);
        regex::Regex::new(&pattern).map(|r| r.find_iter(html).count()).unwrap_or(0)
    }

    /// Extract heading structure in order of appearance.
    fn extract_heading_structure(html: &str) -> Vec<String> {
        let mut headings = Vec::new();
        let pattern = r#"<h([1-6])[^>]*>"#;
        
        if let Ok(regex) = regex::Regex::new(pattern) {
            for cap in regex.captures_iter(html) {
                headings.push(format!("h{}", &cap[1]));
            }
        }
        
        headings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tag_content() {
        let html = r#"<title>Test Title</title>"#;
        assert_eq!(SeoAnalyzer::extract_tag_content(html, "title"), Some("Test Title".to_string()));
    }

    #[test]
    fn test_extract_meta_content() {
        let html = r#"<meta name="description" content="Test description">"#;
        assert_eq!(SeoAnalyzer::extract_meta_content(html, "description"), Some("Test description".to_string()));
    }

    #[test]
    fn test_count_tags() {
        let html = r#"<h1>Title 1</h1><h2>Subtitle</h2><h1>Another H1</h1>"#;
        assert_eq!(SeoAnalyzer::count_tags(html, "h1"), 2);
        assert_eq!(SeoAnalyzer::count_tags(html, "h2"), 1);
    }
}
