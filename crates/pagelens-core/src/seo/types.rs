//! SEO analysis data types

use serde::{Deserialize, Serialize};

/// Severity level for SEO issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Critical issue that significantly impacts SEO.
    Error,
    /// Warning for best practice violations.
    Warning,
    /// Informational suggestion.
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreConfig {
    pub error_penalty: f64,
    pub warning_penalty: f64,
    pub info_penalty: f64,
    pub title_bonus: f64,
    pub description_bonus: f64,
    pub canonical_bonus: f64,
    pub structured_data_bonus: f64,
    pub issue_weight: f64,
    pub performance_weight: f64,
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
            issue_weight: 0.65,
            performance_weight: 0.35,
        }
    }
}

/// A single SEO issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// The severity of this issue.
    pub severity: Severity,
    /// Human-readable description of the issue.
    pub message: String,
    /// Category this issue belongs to.
    pub category: String,
}

/// Meta tag information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    /// Source URL.
    pub src: String,
    /// Alt text (if present).
    pub alt: Option<String>,
    /// Whether the image has alt text.
    pub has_alt: bool,
}

/// Structured data entry (JSON-LD).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredData {
    /// The @type field from the JSON-LD.
    pub schema_type: String,
    /// Raw JSON content.
    pub raw: String,
}

/// Complete SEO analysis report.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Favicon URL extracted from the page (if present).
    pub favicon_url: Option<String>,
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
            favicon_url: None,
            headings: HeadingsInfo::default(),
            images: Vec::new(),
            structured_data: Vec::new(),
            issues: Vec::new(),
        }
    }

    /// Add an issue to the report.
    pub(crate) fn add_issue(&mut self, severity: Severity, category: &str, message: &str) {
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
