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
    #[serde(default)]
    pub status: AnalysisRunStatus,
    pub current_stage: Option<String>,
    pub current_message: Option<String>,
    pub progress: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisRunStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl Default for AnalysisRunStatus {
    fn default() -> Self {
        Self::Completed
    }
}

/// Type of analysis performed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisType {
    Single,
    Crawl,
    HttpBenchmark,
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
    pub id: Option<String>,
    pub url: String,
    pub name: Option<String>,
    pub analysis_type: AnalysisType,
    /// The complete analysis payload as JSON string.
    pub payload_json: String,
    pub summary: AnalysisSummary,
    pub status: AnalysisRunStatus,
    pub current_stage: Option<String>,
    pub current_message: Option<String>,
    pub progress: Option<f64>,
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
    pub status: AnalysisRunStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AnalysisPageResult {
    pub id: String,
    pub run_id: String,
    pub url: String,
    pub depth: u32,
    pub success: bool,
    pub seo_score: Option<f64>,
    pub total_issues: u32,
    pub error_count: u32,
    pub warning_count: u32,
    pub links_found_count: u32,
    pub error_message: Option<String>,
    pub analyzed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateAnalysisPageResult {
    pub run_id: String,
    pub url: String,
    pub depth: u32,
    pub success: bool,
    pub seo_score: Option<f64>,
    pub total_issues: u32,
    pub error_count: u32,
    pub warning_count: u32,
    pub links_found_count: u32,
    pub error_message: Option<String>,
}

/// A cached/downloaded asset associated with an analysis run.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AnalysisAsset {
    pub id: String,
    pub run_id: String,
    pub original_url: String,
    /// Asset type: 'html', 'favicon', 'og_image', 'javascript', 'stylesheet', 'media', 'font'
    pub asset_type: String,
    pub content_type: Option<String>,
    /// Filename only (e.g. "page.html"); full path = assets_dir/{run_id}/{local_path}
    pub local_path: String,
    pub file_size: i64,
    pub download_error: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Input for inserting a new asset record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAnalysisAsset {
    pub run_id: String,
    pub original_url: String,
    pub asset_type: String,
    pub content_type: Option<String>,
    pub local_path: String,
    pub file_size: i64,
    pub download_error: Option<String>,
}
