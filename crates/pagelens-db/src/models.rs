use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

/// Represents a single analysis run stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AnalysisRun {
    /// Unique identifier for this run.
    pub id: String,
    /// The URL that was analyzed.
    pub url: String,
    /// When the analysis was performed.
    pub created_at: DateTime<Utc>,
    /// User-provided name for this run (optional).
    pub name: Option<String>,
    /// Analysis type: "single" for single page, "crawl" for multi-page.
    pub analysis_type: AnalysisType,
    /// The complete analysis payload as JSON string.
    /// For single page: contains snapshot + SEO report.
    /// For crawl: contains CrawlResult.
    pub payload_json: String,
    /// Summary statistics for quick display.
    pub summary: AnalysisSummary,
}

/// Type of analysis performed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisType {
    Single,
    Crawl,
}

/// Summary statistics for quick display in history list.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AnalysisSummary {
    /// Overall SEO score (0-100) if available.
    pub seo_score: Option<f64>,
    /// Number of pages analyzed (1 for single, N for crawl).
    pub page_count: u32,
    /// Total number of issues found.
    pub total_issues: u32,
    /// Number of errors.
    pub error_count: u32,
    /// Number of warnings.
    pub warning_count: u32,
    /// Duration of analysis in milliseconds.
    pub duration_ms: u32,
}

impl Default for AnalysisSummary {
    fn default() -> Self {
        Self {
            seo_score: None,
            page_count: 0,
            total_issues: 0,
            error_count: 0,
            warning_count: 0,
            duration_ms: 0,
        }
    }
}

/// Input for creating a new analysis run.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateAnalysisRun {
    pub url: String,
    pub name: Option<String>,
    pub analysis_type: AnalysisType,
    /// The complete analysis payload as JSON string.
    pub payload_json: String,
    pub summary: AnalysisSummary,
}

/// Input for updating an analysis run's name.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateAnalysisRun {
    pub name: Option<String>,
}

/// Export format for analysis history.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HistoryExport {
    pub version: String,
    pub exported_at: DateTime<Utc>,
    pub runs: Vec<AnalysisRun>,
}

impl HistoryExport {
    pub fn new(runs: Vec<AnalysisRun>) -> Self {
        Self {
            version: "1.0.0".to_string(),
            exported_at: Utc::now(),
            runs,
        }
    }
}

/// List item for history (lighter weight than full AnalysisRun).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HistoryListItem {
    pub id: String,
    pub url: String,
    pub created_at: DateTime<Utc>,
    pub name: Option<String>,
    pub analysis_type: AnalysisType,
    pub summary: AnalysisSummary,
}
