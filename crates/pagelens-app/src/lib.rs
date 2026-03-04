mod error;
mod prelude;

use pagelens_core::{
    Browser, CrawlOptions, Crawler, HttpBenchmarkLatencyStats, HttpBenchmarkOptions,
    HttpBenchmarkResult, HttpBenchmarker, SeoAnalyzer, SeoReport, Snapshot, SnapshotExt,
    SnapshotOptions,
};
use pagelens_logging::{error, info, warn};
use pagelens_db::{
    AnalysisAsset, AnalysisPageResult, AnalysisRun, AnalysisRunStatus, AnalysisSummary,
    AnalysisType, CreateAnalysisAsset, CreateAnalysisPageResult, CreateAnalysisRun, Database,
    HistoryListItem, UpdateAnalysisRun,
};
use prelude::*;
use image::DynamicImage;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, Mutex, Semaphore};
use tokio::task::AbortHandle;

pub use error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyseUrlInput {
    pub run_id: Option<String>,
    pub url: String,
    #[serde(default)]
    pub analysis_type: AnalysisMode,
    pub options: Option<SnapshotOptionsInput>,
    pub crawl_options: Option<CrawlOptionsInput>,
    pub benchmark_options: Option<HttpBenchmarkOptionsInput>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisMode {
    Single,
    Crawl,
    HttpBenchmark,
}

impl Default for AnalysisMode {
    fn default() -> Self {
        Self::Single
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditRunInput {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotOptionsInput {
    pub include_html: bool,
    pub include_accessibility_tree: bool,
    pub include_performance_timing: bool,
    pub include_computed_styles: bool,
    #[serde(default = "default_include_network_metadata")]
    pub include_network_metadata: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpBenchmarkOptionsInput {
    pub requests: Option<usize>,
    pub duration_secs: Option<f64>,
    pub connections: Option<usize>,
    pub method: Option<String>,
    pub qps: Option<f64>,
    pub headers: Option<Vec<(String, String)>>,
    pub cookies: Option<Vec<(String, String)>>,
    pub body: Option<String>,
    pub follow_redirects: Option<bool>,
}

impl From<HttpBenchmarkOptionsInput> for HttpBenchmarkOptions {
    fn from(input: HttpBenchmarkOptionsInput) -> Self {
        let defaults = HttpBenchmarkOptions::default();
        Self {
            requests: input.requests.unwrap_or(defaults.requests),
            duration_secs: input.duration_secs,
            connections: input.connections.unwrap_or(defaults.connections),
            method: input.method.unwrap_or(defaults.method),
            timeout_ms: defaults.timeout_ms,
            qps: input.qps,
            headers: input.headers.unwrap_or_default(),
            cookies: input.cookies.unwrap_or_default(),
            body: input.body,
            follow_redirects: input.follow_redirects.unwrap_or(defaults.follow_redirects),
        }
    }
}

fn default_include_network_metadata() -> bool {
    true
}

impl Default for SnapshotOptionsInput {
    fn default() -> Self {
        Self {
            include_html: true,
            include_accessibility_tree: true,
            include_performance_timing: true,
            include_computed_styles: false,
            include_network_metadata: true,
        }
    }
}

impl From<SnapshotOptionsInput> for SnapshotOptions {
    fn from(input: SnapshotOptionsInput) -> Self {
        Self {
            include_html: input.include_html,
            include_accessibility_tree: input.include_accessibility_tree,
            include_performance_timing: input.include_performance_timing,
            include_computed_styles: input.include_computed_styles,
            include_network_metadata: input.include_network_metadata,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlOptionsInput {
    pub max_pages: Option<u32>,
    pub max_depth: Option<u32>,
    pub follow_external_links: Option<bool>,
    pub same_subdomain_only: Option<bool>,
    pub page_timeout_ms: Option<u64>,
    pub delay_ms: Option<u64>,
    pub max_concurrency: Option<u32>,
}

impl Default for CrawlOptionsInput {
    fn default() -> Self {
        Self {
            max_pages: None,
            max_depth: None,
            follow_external_links: None,
            same_subdomain_only: None,
            page_timeout_ms: None,
            delay_ms: None,
            max_concurrency: None,
        }
    }
}

impl From<CrawlOptionsInput> for CrawlOptions {
    fn from(input: CrawlOptionsInput) -> Self {
        let defaults = CrawlOptions::default();
        Self {
            max_pages: input.max_pages.unwrap_or(defaults.max_pages as u32) as usize,
            max_depth: input.max_depth.unwrap_or(defaults.max_depth as u32) as usize,
            follow_external_links: input
                .follow_external_links
                .unwrap_or(defaults.follow_external_links),
            same_subdomain_only: input
                .same_subdomain_only
                .unwrap_or(defaults.same_subdomain_only),
            page_timeout_ms: input.page_timeout_ms.unwrap_or(defaults.page_timeout_ms),
            delay_ms: input.delay_ms.unwrap_or(defaults.delay_ms),
            max_concurrency: input
                .max_concurrency
                .unwrap_or(defaults.max_concurrency as u32)
                .max(1) as usize,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartRunResponse {
    pub run_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunEvent {
    pub kind: String,
    pub run_id: String,
    pub stage: Option<String>,
    pub message: Option<String>,
    pub progress: Option<f64>,
    pub page_count: Option<u32>,
    pub success_count: Option<u32>,
    pub failed_count: Option<u32>,
    pub page: Option<AnalysisPageResult>,
    pub links_found: Option<Vec<String>>,
    pub discovered_count: Option<u32>,
    pub queued_count: Option<u32>,
    pub running_count: Option<u32>,
    pub scanned_count: Option<u32>,
    pub links_found_total: Option<u32>,
    // Live latency stats (only emitted during http_benchmark progress events)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_latency_min_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_latency_max_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_latency_avg_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_latency_p10_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_latency_p25_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_latency_p50_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_latency_p75_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_latency_p90_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_latency_p95_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_latency_p99_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_latency_p99_9_ms: Option<f64>,
    /// Vec of (bucket_upper_ms, count) pairs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_histogram: Option<Vec<(f64, usize)>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_total_data_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_avg_size_per_request_bytes: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_data_per_sec_bytes: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_method: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_connections: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_target_duration_secs: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaviconAnalyzeInput {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaviconCandidate {
    pub url: String,
    pub rel: String,
    pub sizes: Option<String>,
    pub mime_type: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaviconAnalyzeResponse {
    pub input_url: String,
    pub resolved_page_url: String,
    pub default_favicon_url: String,
    pub candidates: Vec<FaviconCandidate>,
    pub warnings: Vec<String>,
    pub has_web_app_manifest: bool,
    pub has_touch_icon: bool,
    pub touch_web_app_title: Option<String>,
    pub has_svg_favicon: bool,
    pub has_desktop_png_favicon: bool,
    pub ico_declared: bool,
    pub ico_found: bool,
    pub ico_sizes: Vec<String>,
    pub ico_extra_sizes: Vec<String>,
    pub ico_missing_sizes: Vec<String>,
    pub audit_messages: Vec<String>,
    pub candidate_reports: Vec<FaviconCandidateReport>,
    pub global_recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaviconCandidateReport {
    pub url: String,
    pub rel: String,
    pub source: String,
    pub mime_type: Option<String>,
    pub declared_sizes: Option<String>,
    pub format: Option<String>,
    pub file_size_bytes: Option<u64>,
    pub detected_dimensions: Vec<String>,
    pub contrast_on_light: Option<f64>,
    pub contrast_on_dark: Option<f64>,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PwaAnalyzeInput {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PwaManifestSummary {
    pub manifest_url: String,
    pub name: Option<String>,
    pub short_name: Option<String>,
    pub start_url: Option<String>,
    pub display: Option<String>,
    pub theme_color: Option<String>,
    pub background_color: Option<String>,
    pub icon_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PwaAnalyzeResponse {
    pub input_url: String,
    pub resolved_page_url: String,
    pub has_manifest: bool,
    pub manifest: Option<PwaManifestSummary>,
    pub has_service_worker_registration: bool,
    pub has_theme_color_meta: bool,
    pub apple_touch_icon_count: usize,
    pub mask_icon_count: usize,
    pub installability_score: u8,
    pub warnings: Vec<String>,
}

impl RunEvent {
    fn progress(run_id: &str, stage: &str, message: String, progress: Option<f64>) -> Self {
        Self {
            kind: "progress".to_string(),
            run_id: run_id.to_string(),
            stage: Some(stage.to_string()),
            message: Some(message),
            progress,
            page_count: None,
            success_count: None,
            failed_count: None,
            page: None,
            links_found: None,
            discovered_count: None,
            queued_count: None,
            running_count: None,
            scanned_count: None,
            links_found_total: None,
            live_latency_min_ms: None,
            live_latency_max_ms: None,
            live_latency_avg_ms: None,
            live_latency_p10_ms: None,
            live_latency_p25_ms: None,
            live_latency_p50_ms: None,
            live_latency_p75_ms: None,
            live_latency_p90_ms: None,
            live_latency_p95_ms: None,
            live_latency_p99_ms: None,
            live_latency_p99_9_ms: None,
            live_histogram: None,
            live_total_data_bytes: None,
            live_avg_size_per_request_bytes: None,
            live_data_per_sec_bytes: None,
            live_method: None,
            live_connections: None,
            live_target_duration_secs: None,
        }
    }

    fn page(run_id: &str, page: AnalysisPageResult, links_found: Vec<String>) -> Self {
        Self {
            kind: "page".to_string(),
            run_id: run_id.to_string(),
            stage: None,
            message: None,
            progress: None,
            page_count: None,
            success_count: None,
            failed_count: None,
            page: Some(page),
            links_found: Some(links_found),
            discovered_count: None,
            queued_count: None,
            running_count: None,
            scanned_count: None,
            links_found_total: None,
            live_latency_min_ms: None,
            live_latency_max_ms: None,
            live_latency_avg_ms: None,
            live_latency_p10_ms: None,
            live_latency_p25_ms: None,
            live_latency_p50_ms: None,
            live_latency_p75_ms: None,
            live_latency_p90_ms: None,
            live_latency_p95_ms: None,
            live_latency_p99_ms: None,
            live_latency_p99_9_ms: None,
            live_histogram: None,
            live_total_data_bytes: None,
            live_avg_size_per_request_bytes: None,
            live_data_per_sec_bytes: None,
            live_method: None,
            live_connections: None,
            live_target_duration_secs: None,
        }
    }

    fn completed(run_id: &str) -> Self {
        Self::completed_with_counts(run_id, 1, 1, 0)
    }

    fn completed_with_counts(
        run_id: &str,
        page_count: u32,
        success_count: u32,
        failed_count: u32,
    ) -> Self {
        Self {
            kind: "complete".to_string(),
            run_id: run_id.to_string(),
            stage: Some("complete".to_string()),
            message: Some("Analysis complete".to_string()),
            progress: Some(1.0),
            page_count: Some(page_count),
            success_count: Some(success_count),
            failed_count: Some(failed_count),
            page: None,
            links_found: None,
            discovered_count: None,
            queued_count: None,
            running_count: None,
            scanned_count: Some(success_count + failed_count),
            links_found_total: None,
            live_latency_min_ms: None,
            live_latency_max_ms: None,
            live_latency_avg_ms: None,
            live_latency_p10_ms: None,
            live_latency_p25_ms: None,
            live_latency_p50_ms: None,
            live_latency_p75_ms: None,
            live_latency_p90_ms: None,
            live_latency_p95_ms: None,
            live_latency_p99_ms: None,
            live_latency_p99_9_ms: None,
            live_histogram: None,
            live_total_data_bytes: None,
            live_avg_size_per_request_bytes: None,
            live_data_per_sec_bytes: None,
            live_method: None,
            live_connections: None,
            live_target_duration_secs: None,
        }
    }

    fn failed(run_id: &str, message: String) -> Self {
        Self {
            kind: "failed".to_string(),
            run_id: run_id.to_string(),
            stage: Some("failed".to_string()),
            message: Some(message),
            progress: None,
            page_count: None,
            success_count: None,
            failed_count: None,
            page: None,
            links_found: None,
            discovered_count: None,
            queued_count: None,
            running_count: None,
            scanned_count: None,
            links_found_total: None,
            live_latency_min_ms: None,
            live_latency_max_ms: None,
            live_latency_avg_ms: None,
            live_latency_p10_ms: None,
            live_latency_p25_ms: None,
            live_latency_p50_ms: None,
            live_latency_p75_ms: None,
            live_latency_p90_ms: None,
            live_latency_p95_ms: None,
            live_latency_p99_ms: None,
            live_latency_p99_9_ms: None,
            live_histogram: None,
            live_total_data_bytes: None,
            live_avg_size_per_request_bytes: None,
            live_data_per_sec_bytes: None,
            live_method: None,
            live_connections: None,
            live_target_duration_secs: None,
        }
    }

    fn cancelled(run_id: &str, message: String) -> Self {
        Self {
            kind: "cancelled".to_string(),
            run_id: run_id.to_string(),
            stage: Some("cancelled".to_string()),
            message: Some(message),
            progress: None,
            page_count: None,
            success_count: None,
            failed_count: None,
            page: None,
            links_found: None,
            discovered_count: None,
            queued_count: None,
            running_count: None,
            scanned_count: None,
            links_found_total: None,
            live_latency_min_ms: None,
            live_latency_max_ms: None,
            live_latency_avg_ms: None,
            live_latency_p10_ms: None,
            live_latency_p25_ms: None,
            live_latency_p50_ms: None,
            live_latency_p75_ms: None,
            live_latency_p90_ms: None,
            live_latency_p95_ms: None,
            live_latency_p99_ms: None,
            live_latency_p99_9_ms: None,
            live_histogram: None,
            live_total_data_bytes: None,
            live_avg_size_per_request_bytes: None,
            live_data_per_sec_bytes: None,
            live_method: None,
            live_connections: None,
            live_target_duration_secs: None,
        }
    }
}

fn run_is_active(run: &AnalysisRun) -> bool {
    matches!(
        run.status,
        AnalysisRunStatus::Pending | AnalysisRunStatus::Running
    )
}

#[derive(Clone)]
pub struct AppService {
    db_path: PathBuf,
    assets_base_dir: PathBuf,
    runs: Arc<Mutex<HashMap<String, broadcast::Sender<RunEvent>>>>,
    run_abort_handles: Arc<Mutex<HashMap<String, AbortHandle>>>,
    run_cancel_flags: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
}

impl AppService {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let db_path = db_path.as_ref().to_path_buf();
        let assets_base_dir = db_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("assets");
        Database::open(&db_path)?;
        info!(db_path = %db_path.display(), assets_dir = %assets_base_dir.display(), "Initialized AppService");
        Ok(Self {
            db_path,
            assets_base_dir,
            runs: Arc::new(Mutex::new(HashMap::new())),
            run_abort_handles: Arc::new(Mutex::new(HashMap::new())),
            run_cancel_flags: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn assets_base_dir(&self) -> &Path {
        &self.assets_base_dir
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    pub async fn start_analyse(&self, input: AnalyseUrlInput) -> Result<StartRunResponse> {
        let analysis_mode = input.analysis_type;
        let run_id = input
            .run_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        info!(run_id = %run_id, url = %input.url, mode = ?analysis_mode, "Starting analysis run");

        {
            let db = Database::open(&self.db_path)?;
            let repo = db.analysis_repository();
            repo.create(CreateAnalysisRun {
                id: Some(run_id.clone()),
                url: input.url.clone(),
                name: None,
                analysis_type: match analysis_mode {
                    AnalysisMode::Single => AnalysisType::Single,
                    AnalysisMode::Crawl => AnalysisType::Crawl,
                    AnalysisMode::HttpBenchmark => AnalysisType::HttpBenchmark,
                },
                payload_json: "{}".to_string(),
                summary: AnalysisSummary::default(),
                status: AnalysisRunStatus::Running,
                current_stage: Some("queued".to_string()),
                current_message: Some("Queued for analysis".to_string()),
                progress: Some(0.0),
            })?;
        }

        let tx = {
            let mut runs = self.runs.lock().await;
            let (tx, _) = broadcast::channel(512);
            runs.insert(run_id.clone(), tx.clone());
            tx
        };

        let service = self.clone();
        let run_id_for_task = run_id.clone();
        let task_handle = tokio::spawn(async move {
            let result = match analysis_mode {
                AnalysisMode::Single => {
                    service
                        .execute_single(run_id_for_task.clone(), input, tx.clone())
                        .await
                }
                AnalysisMode::Crawl => {
                    service
                        .execute_crawl(run_id_for_task.clone(), input, tx.clone())
                        .await
                }
                AnalysisMode::HttpBenchmark => {
                    service
                        .execute_http_benchmark(run_id_for_task.clone(), input, tx.clone())
                        .await
                }
            };
            if let Err(err) = result {
                error!(run_id = %run_id_for_task, error = %err, "Run execution failed");
                let _ = service
                    .mark_run_failed(&run_id_for_task, err.to_string())
                    .await;
                let _ = tx.send(RunEvent::failed(&run_id_for_task, err.to_string()));
            }
            let mut runs = service.runs.lock().await;
            runs.remove(&run_id_for_task);
            let mut handles = service.run_abort_handles.lock().await;
            handles.remove(&run_id_for_task);
            let mut cancel_flags = service.run_cancel_flags.lock().await;
            cancel_flags.remove(&run_id_for_task);
        });

        {
            let mut handles = self.run_abort_handles.lock().await;
            handles.insert(run_id.clone(), task_handle.abort_handle());
        }

        Ok(StartRunResponse { run_id })
    }

    pub async fn subscribe(&self, run_id: &str) -> Option<broadcast::Receiver<RunEvent>> {
        let runs = self.runs.lock().await;
        runs.get(run_id).map(broadcast::Sender::subscribe)
    }

    pub fn terminal_event_for_run(&self, run_id: &str) -> Result<Option<RunEvent>> {
        let run = self.get_run(run_id)?;
        let event = match run.status {
            AnalysisRunStatus::Completed => {
                let db = Database::open(&self.db_path)?;
                let pages = db.analysis_repository().list_page_results(run_id)?;
                let page_count = pages.len() as u32;
                let success_count = pages.iter().filter(|page| page.success).count() as u32;
                let failed_count = page_count.saturating_sub(success_count);
                Some(RunEvent::completed_with_counts(
                    &run.id,
                    page_count,
                    success_count,
                    failed_count,
                ))
            }
            AnalysisRunStatus::Failed => {
                let message = run
                    .current_message
                    .unwrap_or_else(|| "Analysis failed".to_string());
                if run.current_stage.as_deref() == Some("cancelled") {
                    Some(RunEvent::cancelled(&run.id, message))
                } else {
                    Some(RunEvent::failed(&run.id, message))
                }
            }
            _ => None,
        };
        Ok(event)
    }

    pub async fn cancel_run(&self, run_id: &str) -> Result<()> {
        let run = self.get_run(run_id)?;
        if !matches!(
            run.status,
            AnalysisRunStatus::Running | AnalysisRunStatus::Pending
        ) {
            warn!(run_id = %run_id, status = ?run.status, "Cancel requested for non-active run");
            return Err(Error::Message("Run is not active".to_string()));
        }

        let cancel_message = "Analysis cancelled by user".to_string();
        info!(run_id = %run_id, analysis_type = ?run.analysis_type, "Cancelling run");

        if run.analysis_type == AnalysisType::HttpBenchmark {
            {
                let cancel_flags = self.run_cancel_flags.lock().await;
                if let Some(flag) = cancel_flags.get(run_id) {
                    flag.store(true, Ordering::Relaxed);
                }
            }

            {
                let db = Database::open(&self.db_path)?;
                let updated = db.analysis_repository().update_run_state_if_active(
                    run_id,
                    AnalysisRunStatus::Running,
                    Some("cancelling"),
                    Some("Stopping benchmark and finalizing partial results..."),
                    run.progress,
                )?;
                if !updated {
                    return Err(Error::Message("Run is not active".to_string()));
                }
            }

            {
                let runs = self.runs.lock().await;
                if let Some(tx) = runs.get(run_id) {
                    let _ = tx.send(RunEvent::progress(
                        run_id,
                        "cancelling",
                        "Stopping benchmark and finalizing partial results...".to_string(),
                        run.progress,
                    ));
                }
            }

            return Ok(());
        }

        {
            let db = Database::open(&self.db_path)?;
            let updated = db.analysis_repository().update_run_state_if_active(
                run_id,
                AnalysisRunStatus::Failed,
                Some("cancelled"),
                Some(&cancel_message),
                run.progress,
            )?;
            if !updated {
                return Err(Error::Message("Run is not active".to_string()));
            }
        }

        {
            let mut handles = self.run_abort_handles.lock().await;
            if let Some(handle) = handles.remove(run_id) {
                handle.abort();
            }
        }

        {
            let mut runs = self.runs.lock().await;
            if let Some(tx) = runs.get(run_id) {
                let _ = tx.send(RunEvent::cancelled(run_id, cancel_message));
            }
            runs.remove(run_id);
        }

        Ok(())
    }

    pub fn get_run(&self, run_id: &str) -> Result<AnalysisRun> {
        let db = Database::open(&self.db_path)?;
        Ok(db.analysis_repository().get(run_id)?)
    }

    pub fn edit_run(&self, run_id: &str, input: EditRunInput) -> Result<AnalysisRun> {
        let db = Database::open(&self.db_path)?;
        Ok(db
            .analysis_repository()
            .update(run_id, UpdateAnalysisRun { name: input.name })?)
    }

    pub fn list_run_pages(&self, run_id: &str) -> Result<Vec<AnalysisPageResult>> {
        let db = Database::open(&self.db_path)?;
        Ok(db.analysis_repository().list_page_results(run_id)?)
    }

    pub fn list_history(&self, limit: i64, offset: i64) -> Result<Vec<HistoryListItem>> {
        let db = Database::open(&self.db_path)?;
        Ok(db.analysis_repository().list(limit, offset)?)
    }

    pub fn list_run_assets(&self, run_id: &str) -> Result<Vec<AnalysisAsset>> {
        let db = Database::open(&self.db_path)?;
        Ok(db.analysis_repository().list_assets(run_id)?)
    }

    pub async fn analyze_favicon(&self, input: FaviconAnalyzeInput) -> Result<FaviconAnalyzeResponse> {
        let input_url = input.url.trim();
        if input_url.is_empty() {
            return Err(Error::Message("URL is required".to_string()));
        }

        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .map_err(|err| Error::Message(format!("Failed to build HTTP client: {err}")))?;

        let response = client
            .get(input_url)
            .send()
            .await
            .map_err(|err| Error::Message(format!("Failed to fetch URL: {err}")))?
            .error_for_status()
            .map_err(|err| Error::Message(format!("Page request failed: {err}")))?;

        let resolved_page_url = response.url().clone();
        let html = response
            .text()
            .await
            .map_err(|err| Error::Message(format!("Failed to read page body: {err}")))?;

        let default_favicon_url = resolved_page_url
            .join("/favicon.ico")
            .map_err(|err| Error::Message(format!("Failed to resolve default favicon URL: {err}")))?
            .to_string();

        let (mut warnings, mut candidates, has_web_app_manifest, has_touch_icon, ico_declared_url, touch_web_app_title) = {
            let document = Html::parse_document(&html);
            let link_selector = Selector::parse("link")
                .map_err(|err| Error::Message(format!("Failed to parse selector: {err}")))?;
            let touch_title_selector = Selector::parse("meta[name='apple-mobile-web-app-title']")
                .map_err(|err| Error::Message(format!("Failed to parse selector: {err}")))?;

            let mut warnings = Vec::new();
            let mut seen = HashSet::new();
            let mut candidates = Vec::new();
            let mut has_web_app_manifest = false;
            let mut has_touch_icon = false;
            let mut ico_declared_url: Option<String> = None;

            for element in document.select(&link_selector) {
                let Some(rel_raw) = element.value().attr("rel") else {
                    continue;
                };

                let rel_tokens: Vec<String> = rel_raw
                    .split_ascii_whitespace()
                    .map(|token| token.trim().to_ascii_lowercase())
                    .filter(|token| !token.is_empty())
                    .collect();

                if rel_tokens.iter().any(|token| token == "manifest") {
                    has_web_app_manifest = true;
                }

                if rel_tokens.iter().any(|token| token == "apple-touch-icon") {
                    has_touch_icon = true;
                }

                let is_icon = rel_tokens
                    .iter()
                    .any(|token| token == "icon" || token == "shortcut");
                if !is_icon {
                    continue;
                }

                let Some(href) = element.value().attr("href") else {
                    warnings.push("Found icon rel without href".to_string());
                    continue;
                };

                let resolved_icon_url = match resolved_page_url.join(href) {
                    Ok(url) => url,
                    Err(err) => {
                        warnings.push(format!("Could not resolve icon href '{href}': {err}"));
                        continue;
                    }
                };

                let icon_url = resolved_icon_url.to_string();
                if !seen.insert(icon_url.clone()) {
                    continue;
                }

                let mime_type = element.value().attr("type").map(str::to_string);
                if ico_declared_url.is_none()
                    && (icon_url.to_ascii_lowercase().ends_with(".ico")
                        || mime_type
                            .as_deref()
                            .unwrap_or_default()
                            .to_ascii_lowercase()
                            .contains("icon"))
                {
                    ico_declared_url = Some(icon_url.clone());
                }

                candidates.push(FaviconCandidate {
                    url: icon_url,
                    rel: rel_raw.to_string(),
                    sizes: element.value().attr("sizes").map(str::to_string),
                    mime_type,
                    source: "html_link".to_string(),
                });
            }

            let touch_web_app_title = document
                .select(&touch_title_selector)
                .find_map(|node| node.value().attr("content"))
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);

            (
                warnings,
                candidates,
                has_web_app_manifest,
                has_touch_icon,
                ico_declared_url,
                touch_web_app_title,
            )
        };

        if !candidates
            .iter()
            .any(|candidate| candidate.url == default_favicon_url)
        {
            candidates.push(FaviconCandidate {
                url: default_favicon_url.clone(),
                rel: "icon".to_string(),
                sizes: None,
                mime_type: None,
                source: "default_path".to_string(),
            });
        }

        if candidates.is_empty() {
            warnings.push("No favicon candidates discovered".to_string());
        }

        let has_svg_favicon = candidates.iter().any(|candidate| {
            candidate
                .mime_type
                .as_deref()
                .unwrap_or_default()
                .to_ascii_lowercase()
                .contains("svg")
                || candidate.url.to_ascii_lowercase().contains(".svg")
        });

        let has_desktop_png_favicon = candidates.iter().any(|candidate| {
            let is_png = candidate
                .mime_type
                .as_deref()
                .unwrap_or_default()
                .to_ascii_lowercase()
                .contains("png")
                || candidate.url.to_ascii_lowercase().contains(".png");
            let is_touch = candidate.rel.to_ascii_lowercase().contains("apple-touch-icon");
            is_png && !is_touch
        });

        let ico_declared = ico_declared_url.is_some();
        let ico_target_url = ico_declared_url.unwrap_or_else(|| default_favicon_url.clone());
        let mut ico_found = false;
        let mut ico_sizes = Vec::new();

        match client.get(&ico_target_url).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    ico_found = true;
                    match resp.bytes().await {
                        Ok(bytes) => {
                            ico_sizes = parse_ico_sizes(bytes.as_ref())
                                .into_iter()
                                .map(|(w, h)| format!("{w}x{h}"))
                                .collect();
                        }
                        Err(err) => warnings.push(format!("Failed to read ICO bytes: {err}")),
                    }
                }
            }
            Err(err) => warnings.push(format!("Failed to fetch ICO favicon: {err}")),
        }

        let required_ico_sizes = ["48x48", "32x32", "16x16"];
        let ico_missing_sizes: Vec<String> = required_ico_sizes
            .iter()
            .filter(|required| !ico_sizes.iter().any(|size| size == **required))
            .map(|required| (*required).to_string())
            .collect();
        let ico_extra_sizes: Vec<String> = ico_sizes
            .iter()
            .filter(|size| !required_ico_sizes.iter().any(|required| required == size))
            .cloned()
            .collect();

        let mut candidate_reports = Vec::new();
        for candidate in &candidates {
            let mut issues = Vec::new();
            let mut recommendations = Vec::new();
            let detected_format = detect_icon_format(candidate.mime_type.as_deref(), &candidate.url);
            let mut file_size_bytes = None;
            let mut detected_dimensions = Vec::new();
            let mut contrast_on_light = None;
            let mut contrast_on_dark = None;

            match client.get(&candidate.url).send().await {
                Ok(resp) => {
                    if let Ok(ok_resp) = resp.error_for_status() {
                        match ok_resp.bytes().await {
                            Ok(bytes) => {
                                let bytes_vec = bytes.to_vec();
                                file_size_bytes = Some(bytes_vec.len() as u64);
                                if let Some(size) = file_size_bytes {
                                    if size > 200_000 {
                                        issues.push(format!(
                                            "Icon file size is high ({:.1} KB)",
                                            size as f64 / 1024.0
                                        ));
                                        recommendations.push(
                                            "Compress icon assets below ~200 KB for faster first paint"
                                                .to_string(),
                                        );
                                    }
                                }

                                if detected_format.as_deref() == Some("ico") {
                                    let ico_dims = parse_ico_sizes(&bytes_vec);
                                    detected_dimensions = ico_dims
                                        .iter()
                                        .map(|(w, h)| format!("{w}x{h}"))
                                        .collect();
                                } else if detected_format.as_deref() == Some("svg") {
                                    recommendations.push(
                                        "Verify SVG contrast manually for both light and dark surfaces"
                                            .to_string(),
                                    );
                                } else if let Ok(image) = image::load_from_memory(&bytes_vec) {
                                    detected_dimensions = vec![format!(
                                        "{}x{}",
                                        image.width(),
                                        image.height()
                                    )];
                                    let (light_ratio, dark_ratio) = estimate_icon_contrast(&image);
                                    contrast_on_light = Some(light_ratio);
                                    contrast_on_dark = Some(dark_ratio);
                                    if light_ratio < 2.2 {
                                        issues.push(format!(
                                            "Low contrast on light background ({light_ratio:.2}:1)"
                                        ));
                                        recommendations.push(
                                            "Increase icon edge contrast for light themes"
                                                .to_string(),
                                        );
                                    }
                                    if dark_ratio < 2.2 {
                                        issues.push(format!(
                                            "Low contrast on dark background ({dark_ratio:.2}:1)"
                                        ));
                                        recommendations.push(
                                            "Increase icon edge contrast for dark themes"
                                                .to_string(),
                                        );
                                    }
                                }
                            }
                            Err(err) => {
                                issues.push(format!("Failed to read icon bytes: {err}"));
                            }
                        }
                    } else {
                        issues.push("Icon URL returned non-success status".to_string());
                    }
                }
                Err(err) => {
                    issues.push(format!("Icon URL fetch failed: {err}"));
                }
            }

            if detected_format.is_none() {
                recommendations.push("Declare an explicit icon MIME type and extension".to_string());
            }

            if detected_format.as_deref() == Some("svg") {
                recommendations.push(
                    "Provide a PNG fallback for environments with limited SVG favicon support"
                        .to_string(),
                );
            }

            candidate_reports.push(FaviconCandidateReport {
                url: candidate.url.clone(),
                rel: candidate.rel.clone(),
                source: candidate.source.clone(),
                mime_type: candidate.mime_type.clone(),
                declared_sizes: candidate.sizes.clone(),
                format: detected_format,
                file_size_bytes,
                detected_dimensions,
                contrast_on_light,
                contrast_on_dark,
                issues,
                recommendations,
            });
        }

        let mut audit_messages = Vec::new();
        audit_messages.push(if has_svg_favicon {
            "SVG favicon found".to_string()
        } else {
            "There is no SVG favicon".to_string()
        });
        audit_messages.push(if has_desktop_png_favicon {
            "Desktop PNG favicon found".to_string()
        } else {
            "There is no desktop PNG favicon".to_string()
        });
        audit_messages.push(if ico_declared {
            "The ICO favicon is declared".to_string()
        } else {
            "The ICO favicon is not declared".to_string()
        });
        audit_messages.push(if ico_found {
            "ICO favicon found".to_string()
        } else {
            "ICO favicon not found".to_string()
        });

        if ico_extra_sizes.is_empty() {
            audit_messages.push("Extra sizes found in ICO favicon: none".to_string());
        } else {
            audit_messages.push(format!(
                "Extra sizes found in ICO favicon: {}",
                ico_extra_sizes.join(", ")
            ));
        }

        if ico_missing_sizes.is_empty() {
            audit_messages.push("Missing sizes in ICO favicon: none".to_string());
        } else {
            audit_messages.push(format!(
                "Missing sizes in ICO favicon: {}",
                ico_missing_sizes.join(", ")
            ));
        }

        audit_messages.push(if touch_web_app_title.is_some() {
            "Touch web app title declared".to_string()
        } else {
            "No touch web app title declared".to_string()
        });
        audit_messages.push(if has_touch_icon {
            "Touch icon declared".to_string()
        } else {
            "No touch icon declared".to_string()
        });
        audit_messages.push(if has_web_app_manifest {
            "Web app manifest declared".to_string()
        } else {
            "No web app manifest".to_string()
        });

        let mut global_recommendations = Vec::new();
        if !has_svg_favicon {
            global_recommendations
                .push("Add an SVG favicon for sharper rendering on high-density displays".to_string());
        }
        if !has_desktop_png_favicon {
            global_recommendations.push(
                "Add a desktop PNG favicon (at minimum 32x32, ideally 48x48)".to_string(),
            );
        }
        if !has_touch_icon {
            global_recommendations
                .push("Declare an apple-touch-icon for iOS home screen support".to_string());
        }
        if touch_web_app_title.is_none() {
            global_recommendations.push(
                "Add meta name='apple-mobile-web-app-title' for touch web app naming"
                    .to_string(),
            );
        }
        if !has_web_app_manifest {
            global_recommendations.push(
                "Add a web app manifest with at least 192x192 and 512x512 icons".to_string(),
            );
        }
        if !ico_missing_sizes.is_empty() {
            global_recommendations.push(format!(
                "Include missing ICO sizes: {}",
                ico_missing_sizes.join(", ")
            ));
        }

        info!(
            input_url = %input_url,
            resolved_page_url = %resolved_page_url,
            candidate_count = candidates.len(),
            warning_count = warnings.len(),
            "Favicon analysis completed"
        );

        Ok(FaviconAnalyzeResponse {
            input_url: input_url.to_string(),
            resolved_page_url: resolved_page_url.to_string(),
            default_favicon_url,
            candidates,
            warnings,
            has_web_app_manifest,
            has_touch_icon,
            touch_web_app_title,
            has_svg_favicon,
            has_desktop_png_favicon,
            ico_declared,
            ico_found,
            ico_sizes,
            ico_extra_sizes,
            ico_missing_sizes,
            audit_messages,
            candidate_reports,
            global_recommendations,
        })
    }

    pub async fn analyze_pwa(&self, input: PwaAnalyzeInput) -> Result<PwaAnalyzeResponse> {
        let input_url = input.url.trim();
        if input_url.is_empty() {
            return Err(Error::Message("URL is required".to_string()));
        }

        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .map_err(|err| Error::Message(format!("Failed to build HTTP client: {err}")))?;

        let response = client
            .get(input_url)
            .send()
            .await
            .map_err(|err| Error::Message(format!("Failed to fetch URL: {err}")))?
            .error_for_status()
            .map_err(|err| Error::Message(format!("Page request failed: {err}")))?;

        let resolved_page_url = response.url().clone();
        let html = response
            .text()
            .await
            .map_err(|err| Error::Message(format!("Failed to read page body: {err}")))?;

        let (mut warnings, has_theme_color_meta, apple_touch_icon_count, mask_icon_count, manifest_url) = {
            let document = Html::parse_document(&html);
            let link_selector = Selector::parse("link")
                .map_err(|err| Error::Message(format!("Failed to parse selector: {err}")))?;
            let theme_color_selector = Selector::parse("meta[name='theme-color']")
                .map_err(|err| Error::Message(format!("Failed to parse selector: {err}")))?;

            let mut warnings = Vec::new();
            let mut apple_touch_icon_count = 0usize;
            let mut mask_icon_count = 0usize;
            let mut manifest_url: Option<reqwest::Url> = None;

            for element in document.select(&link_selector) {
                let Some(rel_raw) = element.value().attr("rel") else {
                    continue;
                };

                let rel_tokens: Vec<String> = rel_raw
                    .split_ascii_whitespace()
                    .map(|token| token.trim().to_ascii_lowercase())
                    .filter(|token| !token.is_empty())
                    .collect();

                if rel_tokens.iter().any(|token| token == "apple-touch-icon") {
                    apple_touch_icon_count += 1;
                }
                if rel_tokens.iter().any(|token| token == "mask-icon") {
                    mask_icon_count += 1;
                }

                if !rel_tokens.iter().any(|token| token == "manifest") || manifest_url.is_some() {
                    continue;
                }

                let Some(href) = element.value().attr("href") else {
                    warnings.push("Manifest link is missing href".to_string());
                    continue;
                };

                let resolved_manifest_url = match resolved_page_url.join(href) {
                    Ok(url) => url,
                    Err(err) => {
                        warnings.push(format!("Could not resolve manifest href '{href}': {err}"));
                        continue;
                    }
                };

                manifest_url = Some(resolved_manifest_url);
            }

            let has_theme_color_meta = document.select(&theme_color_selector).next().is_some();

            (
                warnings,
                has_theme_color_meta,
                apple_touch_icon_count,
                mask_icon_count,
                manifest_url,
            )
        };

        let mut has_manifest = false;
        let mut manifest: Option<PwaManifestSummary> = None;

        if let Some(manifest_url) = manifest_url {
            has_manifest = true;
            match client.get(manifest_url.clone()).send().await {
                Ok(manifest_response) => match manifest_response.error_for_status() {
                    Ok(ok_response) => match ok_response.text().await {
                        Ok(manifest_body) => match serde_json::from_str::<serde_json::Value>(&manifest_body) {
                            Ok(value) => {
                                let icons = value
                                    .get("icons")
                                    .and_then(|icons| icons.as_array())
                                    .cloned()
                                    .unwrap_or_default();
                                manifest = Some(PwaManifestSummary {
                                    manifest_url: manifest_url.to_string(),
                                    name: value
                                        .get("name")
                                        .and_then(|entry| entry.as_str())
                                        .map(str::to_string),
                                    short_name: value
                                        .get("short_name")
                                        .and_then(|entry| entry.as_str())
                                        .map(str::to_string),
                                    start_url: value
                                        .get("start_url")
                                        .and_then(|entry| entry.as_str())
                                        .map(str::to_string),
                                    display: value
                                        .get("display")
                                        .and_then(|entry| entry.as_str())
                                        .map(str::to_string),
                                    theme_color: value
                                        .get("theme_color")
                                        .and_then(|entry| entry.as_str())
                                        .map(str::to_string),
                                    background_color: value
                                        .get("background_color")
                                        .and_then(|entry| entry.as_str())
                                        .map(str::to_string),
                                    icon_count: icons.len(),
                                });
                            }
                            Err(err) => {
                                warnings.push(format!("Manifest JSON parse failed: {err}"));
                            }
                        },
                        Err(err) => warnings.push(format!("Failed reading manifest body: {err}")),
                    },
                    Err(err) => warnings.push(format!("Manifest request failed: {err}")),
                },
                Err(err) => warnings.push(format!("Manifest fetch failed: {err}")),
            }
        }

        let has_service_worker_registration =
            html.contains("navigator.serviceWorker") || html.contains("serviceWorker.register");

        let mut score = 0u8;
        if has_manifest {
            score += 30;
        }
        if manifest.as_ref().map(|entry| entry.icon_count > 0).unwrap_or(false) {
            score += 20;
        }
        if has_service_worker_registration {
            score += 30;
        }
        if has_theme_color_meta {
            score += 10;
        }
        if apple_touch_icon_count > 0 {
            score += 10;
        }

        info!(
            input_url = %input_url,
            resolved_page_url = %resolved_page_url,
            has_manifest,
            has_service_worker_registration,
            installability_score = score,
            "PWA analysis completed"
        );

        Ok(PwaAnalyzeResponse {
            input_url: input_url.to_string(),
            resolved_page_url: resolved_page_url.to_string(),
            has_manifest,
            manifest,
            has_service_worker_registration,
            has_theme_color_meta,
            apple_touch_icon_count,
            mask_icon_count,
            installability_score: score,
            warnings,
        })
    }

    async fn mark_run_failed(&self, run_id: &str, message: String) -> Result<()> {
        error!(run_id = %run_id, error_message = %message, "Marking run as failed");
        let db = Database::open(&self.db_path)?;
        let repo = db.analysis_repository();
        let updated = repo.update_run_state_if_active(
            run_id,
            AnalysisRunStatus::Failed,
            Some("failed"),
            Some(&message),
            None,
        )?;

        if !updated {
            return Ok(());
        }

        let run = repo.get(run_id)?;
        let _ = repo.insert_page_result(CreateAnalysisPageResult {
            run_id: run_id.to_string(),
            url: run.url,
            depth: 0,
            success: false,
            seo_score: None,
            total_issues: 0,
            error_count: 0,
            warning_count: 0,
            links_found_count: 0,
            error_message: Some(message),
        });

        Ok(())
    }

    async fn execute_single(
        &self,
        run_id: String,
        input: AnalyseUrlInput,
        tx: broadcast::Sender<RunEvent>,
    ) -> Result<()> {
        info!(run_id = %run_id, url = %input.url, "Executing single analysis");
        let start = Instant::now();

        {
            let db = Database::open(&self.db_path)?;
            db.analysis_repository().update_run_state(
                &run_id,
                AnalysisRunStatus::Running,
                Some("browser"),
                Some("Launching browser"),
                Some(0.1),
            )?;
        }
        let _ = tx.send(RunEvent::progress(
            &run_id,
            "browser",
            "Launching browser".to_string(),
            Some(0.1),
        ));

        let browser = Browser::launch().await?;

        {
            let db = Database::open(&self.db_path)?;
            db.analysis_repository().update_run_state(
                &run_id,
                AnalysisRunStatus::Running,
                Some("navigation"),
                Some("Navigating to URL"),
                Some(0.3),
            )?;
        }
        let _ = tx.send(RunEvent::progress(
            &run_id,
            "navigation",
            format!("Navigating to {}", input.url),
            Some(0.3),
        ));

        let page = browser.navigate(&input.url).await?;
        let options: SnapshotOptions = input.options.unwrap_or_default().into();
        let snapshot = page.snapshot(options).await?;
        let seo_report = SeoAnalyzer::analyze(&snapshot);

        let error_count = seo_report
            .issues
            .iter()
            .filter(|issue| matches!(issue.severity, pagelens_core::Severity::Error))
            .count() as u32;
        let warning_count = seo_report
            .issues
            .iter()
            .filter(|issue| matches!(issue.severity, pagelens_core::Severity::Warning))
            .count() as u32;

        let page_row = {
            let db = Database::open(&self.db_path)?;
            db.analysis_repository()
                .insert_page_result(CreateAnalysisPageResult {
                    run_id: run_id.clone(),
                    url: input.url.clone(),
                    depth: 0,
                    success: true,
                    seo_score: Some(seo_report.score),
                    total_issues: seo_report.issues.len() as u32,
                    error_count,
                    warning_count,
                    links_found_count: 0,
                    error_message: None,
                })?
        };
        let _ = tx.send(RunEvent::page(&run_id, page_row, Vec::new()));

        let payload_json = serde_json::to_string(&serde_json::json!({
            "snapshot": snapshot,
            "seo_report": seo_report,
        }))?;
        let summary = AnalysisSummary {
            seo_score: Some(seo_report.score),
            page_count: 1,
            total_issues: seo_report.issues.len() as u32,
            error_count,
            warning_count,
            duration_ms: start.elapsed().as_millis() as u32,
        };

        {
            let db = Database::open(&self.db_path)?;
            if !db
                .analysis_repository()
                .complete_run_if_active(&run_id, &payload_json, &summary)?
            {
                return Ok(());
            }
        }

        let _ = tx.send(RunEvent::completed(&run_id));
        info!(run_id = %run_id, duration_ms = start.elapsed().as_millis() as u64, "Single analysis completed");

        // Fire-and-forget asset download (does not block completion event)
        let db_path = self.db_path.clone();
        let assets_base_dir = self.assets_base_dir.clone();
        let run_id_clone = run_id.clone();
        tokio::spawn(async move {
            let _ = download_and_store_assets(
                &run_id_clone,
                &snapshot,
                &seo_report,
                &assets_base_dir,
                &db_path,
            )
            .await;
        });

        Ok(())
    }

    async fn execute_crawl(
        &self,
        run_id: String,
        input: AnalyseUrlInput,
        tx: broadcast::Sender<RunEvent>,
    ) -> Result<()> {
        info!(run_id = %run_id, seed_url = %input.url, "Executing crawl analysis");
        let start = Instant::now();

        {
            let db = Database::open(&self.db_path)?;
            if !db.analysis_repository().update_run_state_if_active(
                &run_id,
                AnalysisRunStatus::Running,
                Some("browser"),
                Some("Launching browser"),
                Some(0.1),
            )? {
                return Ok(());
            }
        }
        let _ = tx.send(RunEvent::progress(
            &run_id,
            "browser",
            "Launching browser".to_string(),
            Some(0.1),
        ));

        let browser = Browser::launch().await?;

        {
            let db = Database::open(&self.db_path)?;
            if !db.analysis_repository().update_run_state_if_active(
                &run_id,
                AnalysisRunStatus::Running,
                Some("crawl"),
                Some("Starting crawl"),
                Some(0.2),
            )? {
                return Ok(());
            }
        }
        let _ = tx.send(RunEvent::progress(
            &run_id,
            "crawl",
            format!("Starting crawl from {}", input.url),
            Some(0.2),
        ));

        let crawl_options: CrawlOptions = input.crawl_options.unwrap_or_default().into();
        let crawler = Crawler::new(&browser, crawl_options);
        let crawl_result = crawler
            .crawl_with_callback(&input.url, |page, stats| {
                if let Ok(db) = Database::open(&self.db_path) {
                    let repo = db.analysis_repository();

                    let is_active = repo
                        .get(&run_id)
                        .map(|run| run_is_active(&run))
                        .unwrap_or(false);
                    if !is_active {
                        return;
                    }

                    let error_count = page
                        .seo_report
                        .issues
                        .iter()
                        .filter(|issue| matches!(issue.severity, pagelens_core::Severity::Error))
                        .count() as u32;
                    let warning_count = page
                        .seo_report
                        .issues
                        .iter()
                        .filter(|issue| matches!(issue.severity, pagelens_core::Severity::Warning))
                        .count() as u32;

                    let scanned = (stats.crawled_pages + stats.failed_pages) as u32;
                    let total = (stats.total_pages as u32).max(scanned.max(1));
                    let progress = if total > 0 {
                        0.2 + ((scanned as f64 / total as f64) * 0.6)
                    } else {
                        0.2
                    };

                    let message = format!(
                        "Scanned {} pages ({} successful, {} failed)",
                        scanned, stats.crawled_pages, stats.failed_pages
                    );

                    let can_emit = repo
                        .update_run_state_if_active(
                            &run_id,
                            AnalysisRunStatus::Running,
                            Some("crawl"),
                            Some(&message),
                            Some(progress.min(0.8)),
                        )
                        .unwrap_or(false);

                    if !can_emit {
                        return;
                    }

                    if let Ok(page_row) = repo.insert_page_result(CreateAnalysisPageResult {
                        run_id: run_id.clone(),
                        url: page.url.clone(),
                        depth: page.depth as u32,
                        success: page.success,
                        seo_score: if page.success {
                            Some(page.seo_report.score)
                        } else {
                            None
                        },
                        total_issues: page.seo_report.issues.len() as u32,
                        error_count,
                        warning_count,
                        links_found_count: page.links_found.len() as u32,
                        error_message: page.error.clone(),
                    }) {
                        let _ =
                            tx.send(RunEvent::page(&run_id, page_row, page.links_found.clone()));
                    }

                    let _ = tx.send(RunEvent {
                        kind: "progress".to_string(),
                        run_id: run_id.clone(),
                        stage: Some("crawl".to_string()),
                        message: Some(message),
                        progress: Some(progress.min(0.8)),
                        page_count: Some(total),
                        success_count: Some(stats.crawled_pages as u32),
                        failed_count: Some(stats.failed_pages as u32),
                        page: None,
                        links_found: None,
                        discovered_count: Some(stats.total_pages as u32),
                        queued_count: Some(stats.queued_pages as u32),
                        running_count: Some(stats.running_pages as u32),
                        scanned_count: Some(scanned),
                        links_found_total: Some(stats.links_found_total as u32),
                        live_latency_min_ms: None,
                        live_latency_max_ms: None,
                        live_latency_avg_ms: None,
                        live_latency_p10_ms: None,
                        live_latency_p25_ms: None,
                        live_latency_p50_ms: None,
                        live_latency_p75_ms: None,
                        live_latency_p90_ms: None,
                        live_latency_p95_ms: None,
                        live_latency_p99_ms: None,
                        live_latency_p99_9_ms: None,
                        live_histogram: None,
                        live_total_data_bytes: None,
                        live_avg_size_per_request_bytes: None,
                        live_data_per_sec_bytes: None,
                        live_method: None,
                        live_connections: None,
                        live_target_duration_secs: None,
                    });
                }
            })
            .await?;

        {
            let db = Database::open(&self.db_path)?;
            if !db.analysis_repository().update_run_state_if_active(
                &run_id,
                AnalysisRunStatus::Running,
                Some("analysis"),
                Some("Computing aggregate metrics"),
                Some(0.9),
            )? {
                return Ok(());
            }
        }

        let payload_json = serde_json::to_string(&crawl_result)?;
        let summary = AnalysisSummary {
            seo_score: Some(crawl_result.aggregate.avg_seo_score),
            page_count: crawl_result.stats.crawled_pages as u32,
            total_issues: crawl_result.aggregate.total_issues as u32,
            error_count: crawl_result.aggregate.total_errors as u32,
            warning_count: crawl_result.aggregate.total_warnings as u32,
            duration_ms: start.elapsed().as_millis() as u32,
        };

        {
            let db = Database::open(&self.db_path)?;
            if !db
                .analysis_repository()
                .complete_run_if_active(&run_id, &payload_json, &summary)?
            {
                return Ok(());
            }
        }

        let _ = tx.send(RunEvent::completed_with_counts(
            &run_id,
            crawl_result.stats.total_pages as u32,
            crawl_result.stats.crawled_pages as u32,
            crawl_result.stats.failed_pages as u32,
        ));

        info!(
            run_id = %run_id,
            duration_ms = start.elapsed().as_millis() as u64,
            crawled_pages = crawl_result.stats.crawled_pages,
            failed_pages = crawl_result.stats.failed_pages,
            "Crawl analysis completed"
        );

        Ok(())
    }

    async fn execute_http_benchmark(
        &self,
        run_id: String,
        input: AnalyseUrlInput,
        tx: broadcast::Sender<RunEvent>,
    ) -> Result<()> {
        info!(run_id = %run_id, url = %input.url, "Executing HTTP benchmark analysis");
        let start = Instant::now();

        {
            let db = Database::open(&self.db_path)?;
            if !db.analysis_repository().update_run_state_if_active(
                &run_id,
                AnalysisRunStatus::Running,
                Some("benchmark"),
                Some("Preparing HTTP benchmark"),
                Some(0.05),
            )? {
                return Ok(());
            }
        }
        let _ = tx.send(RunEvent::progress(
            &run_id,
            "benchmark",
            "Preparing HTTP benchmark".to_string(),
            Some(0.05),
        ));

        let mut benchmark_input = input.benchmark_options.unwrap_or(HttpBenchmarkOptionsInput {
            requests: None,
            duration_secs: Some(10.0),
            connections: None,
            method: None,
            qps: None,
            headers: None,
            cookies: None,
            body: None,
            follow_redirects: None,
        });
        if benchmark_input.duration_secs.unwrap_or(0.0) <= 0.0 {
            benchmark_input.duration_secs = Some(10.0);
        }
        let benchmark_url = input.url.clone();
        let benchmark_options: HttpBenchmarkOptions = benchmark_input.into();
        let benchmarker = HttpBenchmarker::new(&benchmark_options)?;

        let cancel_flag = {
            let mut cancel_flags = self.run_cancel_flags.lock().await;
            cancel_flags
                .entry(run_id.clone())
                .or_insert_with(|| Arc::new(AtomicBool::new(false)))
                .clone()
        };

        let db_path = self.db_path.clone();
        let run_id_for_progress = run_id.clone();
        let tx_for_progress = tx.clone();
        let benchmark_url_for_live = benchmark_url.clone();
        let benchmark_start = Instant::now();
        let duration_target_secs = benchmark_options.duration_secs.filter(|&d| d > 0.0);
        // Initialise far in the past so the very first event is always emitted.
        let mut last_event_at = benchmark_start
            .checked_sub(Duration::from_secs(1))
            .unwrap_or(benchmark_start);
        // Sorted accumulator for live latency computation.
        let mut live_latencies: Vec<f64> = Vec::new();
        let mut live_total_data_bytes: u64 = 0;
        let mut live_sized_response_count: usize = 0;
        let mut live_status_code_distribution: BTreeMap<String, usize> = BTreeMap::new();
        let mut live_error_distribution: BTreeMap<String, usize> = BTreeMap::new();
        let mut last_payload_persist_at = benchmark_start
            .checked_sub(Duration::from_secs(1))
            .unwrap_or(benchmark_start);

        let result = benchmarker
            .benchmark_with_callback(
                &benchmark_url,
                benchmark_options.clone(),
                Arc::clone(&cancel_flag),
                move |successful, failed, total, latest_sample| {
                    let latency_ms = latest_sample.latency_ms;
                    // Insert latency into sorted vec (binary search insertion).
                    let pos = live_latencies.partition_point(|&x| x < latency_ms);
                    live_latencies.insert(pos, latency_ms);
                    if latest_sample.success {
                        if let Some(code) = latest_sample.status_code {
                            *live_status_code_distribution
                                .entry(code.to_string())
                                .or_insert(0) += 1;
                        }
                        if let Some(bytes) = latest_sample.response_size_bytes {
                            live_total_data_bytes = live_total_data_bytes.saturating_add(bytes);
                            live_sized_response_count += 1;
                        }
                    } else if let Some(err) = latest_sample.error.as_ref() {
                        let normalized = if err.contains("timed out") || err.contains("timeout") {
                            "timeout".to_string()
                        } else if err.contains("connection refused") {
                            "connection refused".to_string()
                        } else if err.contains("connection reset") {
                            "connection reset".to_string()
                        } else if err.to_lowercase().contains("dns") {
                            "dns error".to_string()
                        } else if err.contains("certificate")
                            || err.contains("tls")
                            || err.contains("ssl")
                        {
                            "tls/certificate error".to_string()
                        } else if err.to_lowercase().contains("redirect") {
                            "redirection limit reached".to_string()
                        } else {
                            err.chars().take(60).collect()
                        };
                        *live_error_distribution.entry(normalized).or_insert(0) += 1;
                    }

                    let now = Instant::now();
                    let done = successful + failed;
                    if now.duration_since(last_event_at).as_millis() < 50 {
                        return;
                    }
                    last_event_at = now;

                    let elapsed_secs =
                        now.duration_since(benchmark_start).as_secs_f64();
                    let live_rps = if elapsed_secs > 0.0 {
                        done as f64 / elapsed_secs
                    } else {
                        0.0
                    };
                    let progress_ratio = if total == 0 {
                        if let Some(target) = duration_target_secs {
                            (elapsed_secs / target).clamp(0.0, 1.0)
                        } else {
                            0.0_f64
                        }
                    } else {
                        done as f64 / total as f64
                    };
                    let progress = (0.1_f64 + progress_ratio * 0.9_f64).min(1.0_f64);
                    let message = if total == 0 {
                        if let Some(target) = duration_target_secs {
                            format!("{done} requests in {elapsed_secs:.1}s/{target:.1}s ({live_rps:.1} req/s)")
                        } else {
                            format!("{done} requests ({live_rps:.1} req/s)")
                        }
                    } else {
                        format!("{done}/{total} ({live_rps:.1} req/s)")
                    };


                    // Compute live latency stats from sorted accumulator.
                    let (lat_min, lat_max, lat_avg, lat_p10, lat_p25, lat_p50, lat_p75, lat_p90, lat_p95, lat_p99, lat_p99_9, hist) =
                        if live_latencies.is_empty() {
                            (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, Vec::new())
                        } else {
                            let n = live_latencies.len();
                            let min = live_latencies[0];
                            let max = live_latencies[n - 1];
                            let avg = live_latencies.iter().sum::<f64>() / n as f64;
                            let pct = |p: f64| -> f64 {
                                let rank = ((n as f64 - 1.0) * p).round() as usize;
                                live_latencies.get(rank).copied().unwrap_or(0.0)
                            };
                            let bins: usize = 11;
                            let hist = if (max - min).abs() < f64::EPSILON {
                                vec![(max, n)]
                            } else {
                                let step = (max - min) / bins as f64;
                                let mut counts = vec![0usize; bins];
                                for &v in &live_latencies {
                                    let mut idx = ((v - min) / step).floor() as usize;
                                    if idx >= bins { idx = bins - 1; }
                                    counts[idx] += 1;
                                }
                                counts.into_iter().enumerate()
                                    .map(|(i, c)| (min + step * (i as f64 + 1.0), c))
                                    .collect()
                            };
                            (min, max, avg, pct(0.10), pct(0.25), pct(0.50), pct(0.75), pct(0.90), pct(0.95), pct(0.99), pct(0.999), hist)
                        };
                    let live_avg_size_per_request_bytes =
                        if live_sized_response_count > 0 {
                            live_total_data_bytes as f64 / live_sized_response_count as f64
                        } else {
                            0.0
                        };
                    let live_data_per_sec_bytes = if elapsed_secs > 0.0 {
                        live_total_data_bytes as f64 / elapsed_secs
                    } else {
                        0.0
                    };

                    if let Ok(db) = Database::open(&db_path) {
                        let success_rate = if done == 0 {
                            0.0
                        } else {
                            successful as f64 / done as f64
                        };
                        let live_payload = HttpBenchmarkResult {
                            url: benchmark_url_for_live.clone(),
                            method: benchmark_options.method.clone(),
                            requests: done,
                            connections: benchmark_options.connections,
                            duration_ms: (elapsed_secs * 1000.0) as u64,
                            requests_per_sec: live_rps,
                            successful_requests: successful,
                            failed_requests: failed,
                            success_rate,
                            status_code_distribution: live_status_code_distribution.clone(),
                            error_distribution: live_error_distribution.clone(),
                            latency: HttpBenchmarkLatencyStats {
                                min_ms: lat_min,
                                max_ms: lat_max,
                                avg_ms: lat_avg,
                                p10_ms: lat_p10,
                                p25_ms: lat_p25,
                                p50_ms: lat_p50,
                                p75_ms: lat_p75,
                                p90_ms: lat_p90,
                                p95_ms: lat_p95,
                                p99_ms: lat_p99,
                                p99_9_ms: lat_p99_9,
                            },
                            latency_histogram: hist.clone(),
                            samples: Vec::new(),
                            total_data_bytes: live_total_data_bytes,
                            avg_size_per_request_bytes: live_avg_size_per_request_bytes,
                            data_per_sec_bytes: live_data_per_sec_bytes,
                            is_duration_mode: total == 0,
                            target_duration_secs: duration_target_secs,
                        };
                        let summary = AnalysisSummary {
                            seo_score: None,
                            page_count: done as u32,
                            total_issues: failed as u32,
                            error_count: failed as u32,
                            warning_count: 0,
                            duration_ms: (elapsed_secs * 1000.0) as u32,
                        };

                        let persist_live = now.duration_since(last_payload_persist_at).as_millis() >= 250;

                        let updated = if persist_live {
                            last_payload_persist_at = now;
                            let payload_json = serde_json::to_string(&live_payload)
                                .unwrap_or_else(|_| "{}".to_string());
                            db.analysis_repository().update_benchmark_live_snapshot_if_active(
                                &run_id_for_progress,
                                &payload_json,
                                &summary,
                                Some("benchmark"),
                                Some(&message),
                                Some(progress),
                            )
                        } else {
                            db.analysis_repository().update_run_state_if_active(
                                &run_id_for_progress,
                                AnalysisRunStatus::Running,
                                Some("benchmark"),
                                Some(&message),
                                Some(progress),
                            )
                        };
                        if !matches!(updated, Ok(true)) {
                            return;
                        }
                    }

                    let _ = tx_for_progress.send(RunEvent {
                        kind: "progress".to_string(),
                        run_id: run_id_for_progress.clone(),
                        stage: Some("benchmark".to_string()),
                        message: Some(message),
                        progress: Some(progress),
                        page_count: if total == 0 { None } else { Some(total as u32) },
                        success_count: Some(successful as u32),
                        failed_count: Some(failed as u32),
                        page: None,
                        links_found: None,
                        discovered_count: None,
                        queued_count: None,
                        running_count: None,
                        scanned_count: Some(done as u32),
                        links_found_total: None,
                        live_latency_min_ms: Some(lat_min),
                        live_latency_max_ms: Some(lat_max),
                        live_latency_avg_ms: Some(lat_avg),
                        live_latency_p10_ms: Some(lat_p10),
                        live_latency_p25_ms: Some(lat_p25),
                        live_latency_p50_ms: Some(lat_p50),
                        live_latency_p75_ms: Some(lat_p75),
                        live_latency_p90_ms: Some(lat_p90),
                        live_latency_p95_ms: Some(lat_p95),
                        live_latency_p99_ms: Some(lat_p99),
                        live_latency_p99_9_ms: Some(lat_p99_9),
                        live_histogram: Some(hist),
                        live_total_data_bytes: Some(live_total_data_bytes),
                        live_avg_size_per_request_bytes: Some(live_avg_size_per_request_bytes),
                        live_data_per_sec_bytes: Some(live_data_per_sec_bytes),
                        live_method: Some(benchmark_options.method.clone()),
                        live_connections: Some(benchmark_options.connections as u32),
                        live_target_duration_secs: duration_target_secs,
                    });
                },
            )
            .await?;

        let cancelled_by_user = cancel_flag.load(Ordering::Relaxed);

        let payload_json = serde_json::to_string(&result)?;
        let summary = AnalysisSummary {
            seo_score: None,
            page_count: result.requests as u32,
            total_issues: result.failed_requests as u32,
            error_count: result.failed_requests as u32,
            warning_count: 0,
            duration_ms: start.elapsed().as_millis() as u32,
        };

        {
            let db = Database::open(&self.db_path)?;
            if cancelled_by_user {
                if !db
                    .analysis_repository()
                    .complete_run_as_cancelled_if_active(
                        &run_id,
                        &payload_json,
                        &summary,
                        "Analysis cancelled by user. Showing partial benchmark results.",
                    )?
                {
                    return Ok(());
                }
            } else if !db
                .analysis_repository()
                .complete_run_if_active(&run_id, &payload_json, &summary)?
            {
                return Ok(());
            }
        }

        if cancelled_by_user {
            info!(
                run_id = %run_id,
                requests = result.requests,
                successful_requests = result.successful_requests,
                failed_requests = result.failed_requests,
                "HTTP benchmark cancelled; persisting partial results"
            );
            let _ = tx.send(RunEvent::cancelled(
                &run_id,
                "Analysis cancelled by user. Showing partial benchmark results.".to_string(),
            ));
        } else {
            let _ = tx.send(RunEvent::completed_with_counts(
                &run_id,
                result.requests as u32,
                result.successful_requests as u32,
                result.failed_requests as u32,
            ));
            info!(
                run_id = %run_id,
                duration_ms = start.elapsed().as_millis() as u64,
                requests = result.requests,
                successful_requests = result.successful_requests,
                failed_requests = result.failed_requests,
                "HTTP benchmark completed"
            );
        }

        Ok(())
    }
}

/// Insert an asset record via spawn_blocking (Database is not Send).
async fn db_insert_asset(db_path: PathBuf, input: CreateAnalysisAsset) {
    let _ = tokio::task::spawn_blocking(move || {
        if let Ok(db) = Database::open(&db_path) {
            let _ = db.analysis_repository().insert_asset(input);
        }
    })
    .await;
}

/// Download and cache all assets referenced by a page analysis.
async fn download_and_store_assets(
    run_id: &str,
    snapshot: &Snapshot,
    seo_report: &SeoReport,
    assets_base_dir: &Path,
    db_path: &Path,
) {
    let run_dir = assets_base_dir.join(run_id);
    if tokio::fs::create_dir_all(&run_dir).await.is_err() {
        return;
    }

    // Save page HTML — do the write first, then the DB insert (no non-Send across .await)
    let html_path = run_dir.join("page.html");
    let html_bytes = snapshot.html.clone().into_bytes();
    let (html_size, html_err) = match tokio::fs::write(&html_path, &html_bytes).await {
        Ok(()) => (html_bytes.len() as i64, None),
        Err(e) => (0i64, Some(e.to_string())),
    };
    db_insert_asset(
        db_path.to_path_buf(),
        CreateAnalysisAsset {
            run_id: run_id.to_string(),
            original_url: snapshot.url.clone(),
            asset_type: "html".to_string(),
            content_type: Some("text/html".to_string()),
            local_path: "page.html".to_string(),
            file_size: html_size,
            download_error: html_err,
        },
    )
    .await;

    // Build deduplicated download queue
    let mut queue: Vec<(String, String)> = Vec::new();
    let mut seen_urls: HashSet<String> = HashSet::new();

    // Helper macro to avoid closure borrow conflicts
    macro_rules! enqueue_opt {
        ($opt:expr, $t:expr) => {
            if let Some(u) = $opt {
                if !u.is_empty() && seen_urls.insert(u.clone()) {
                    queue.push((u.clone(), $t.to_string()));
                }
            }
        };
    }
    macro_rules! enqueue {
        ($url:expr, $t:expr) => {
            if seen_urls.insert($url.clone()) {
                queue.push(($url.clone(), $t.to_string()));
            }
        };
    }

    enqueue_opt!(snapshot.favicon_url.as_ref(), "favicon");
    enqueue_opt!(seo_report.open_graph.image.as_ref(), "og_image");
    // Twitter image only if different from OG image
    if seo_report
        .twitter_card
        .image
        .as_ref()
        .map(|t| seo_report.open_graph.image.as_deref() != Some(t.as_str()))
        .unwrap_or(false)
    {
        enqueue_opt!(seo_report.twitter_card.image.as_ref(), "og_image");
    }
    for url in &snapshot.referenced_assets.javascript {
        enqueue!(url, "javascript");
    }
    for url in &snapshot.referenced_assets.stylesheets {
        enqueue!(url, "stylesheet");
    }
    for url in &snapshot.referenced_assets.media {
        enqueue!(url, "media");
    }
    for url in &snapshot.referenced_assets.fonts {
        enqueue!(url, "font");
    }

    let semaphore = Arc::new(Semaphore::new(8));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default();

    let mut handles = Vec::new();
    for (url, asset_type) in queue {
        let permit = semaphore.clone().acquire_owned().await.ok();
        let client = client.clone();
        let run_dir = run_dir.clone();
        let db_path = db_path.to_path_buf();
        let run_id = run_id.to_string();

        handles.push(tokio::spawn(async move {
            let _permit = permit;
            download_single_asset(&url, &asset_type, &run_id, &run_dir, &db_path, &client).await;
        }));
    }
    for h in handles {
        let _ = h.await;
    }
}

async fn download_single_asset(
    url: &str,
    asset_type: &str,
    run_id: &str,
    run_dir: &Path,
    db_path: &Path,
    client: &reqwest::Client,
) {
    // All async work first — no non-Send values held across .await
    let response = client.get(url).send().await;

    let record = match response {
        Err(e) => CreateAnalysisAsset {
            run_id: run_id.to_string(),
            original_url: url.to_string(),
            asset_type: asset_type.to_string(),
            content_type: None,
            local_path: String::new(),
            file_size: 0,
            download_error: Some(e.to_string()),
        },
        Ok(resp) => {
            let content_type = resp
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.split(';').next().unwrap_or(s).trim().to_string());

            let ext = extension_from_content_type_or_url(content_type.as_deref(), url);
            let filename = format!("{}{}", uuid::Uuid::new_v4(), ext);
            let file_path = run_dir.join(&filename);

            match resp.bytes().await {
                Err(e) => CreateAnalysisAsset {
                    run_id: run_id.to_string(),
                    original_url: url.to_string(),
                    asset_type: asset_type.to_string(),
                    content_type,
                    local_path: String::new(),
                    file_size: 0,
                    download_error: Some(e.to_string()),
                },
                Ok(bytes) => {
                    let file_size = bytes.len() as i64;
                    let (local_path, download_error) =
                        match tokio::fs::write(&file_path, &bytes).await {
                            Ok(()) => (filename, None),
                            Err(e) => (String::new(), Some(e.to_string())),
                        };
                    CreateAnalysisAsset {
                        run_id: run_id.to_string(),
                        original_url: url.to_string(),
                        asset_type: asset_type.to_string(),
                        content_type,
                        local_path,
                        file_size,
                        download_error,
                    }
                }
            }
        }
    };

    db_insert_asset(db_path.to_path_buf(), record).await;
}

fn extension_from_content_type_or_url(content_type: Option<&str>, url: &str) -> String {
    if let Some(ct) = content_type {
        let ext = match ct {
            "text/html" => ".html",
            "text/css" => ".css",
            "application/javascript" | "text/javascript" => ".js",
            "image/png" => ".png",
            "image/jpeg" | "image/jpg" => ".jpg",
            "image/gif" => ".gif",
            "image/webp" => ".webp",
            "image/svg+xml" => ".svg",
            "image/x-icon" | "image/vnd.microsoft.icon" => ".ico",
            "font/woff" => ".woff",
            "font/woff2" => ".woff2",
            "font/ttf" | "application/x-font-ttf" => ".ttf",
            "font/otf" | "application/x-font-otf" => ".otf",
            _ => "",
        };
        if !ext.is_empty() {
            return ext.to_string();
        }
    }
    // Fallback: derive from URL path
    url.split('?')
        .next()
        .and_then(|path| path.rsplit('.').next())
        .filter(|ext| ext.len() <= 5 && ext.chars().all(|c| c.is_alphanumeric()))
        .map(|ext| format!(".{}", ext))
        .unwrap_or_default()
}

fn detect_icon_format(mime_type: Option<&str>, url: &str) -> Option<String> {
    let mime = mime_type.unwrap_or_default().to_ascii_lowercase();
    let url_lower = url.to_ascii_lowercase();

    if mime.contains("svg") || url_lower.ends_with(".svg") {
        return Some("svg".to_string());
    }
    if mime.contains("png") || url_lower.ends_with(".png") {
        return Some("png".to_string());
    }
    if mime.contains("icon") || url_lower.ends_with(".ico") {
        return Some("ico".to_string());
    }
    if mime.contains("webp") || url_lower.ends_with(".webp") {
        return Some("webp".to_string());
    }
    if mime.contains("jpeg") || mime.contains("jpg") || url_lower.ends_with(".jpg") || url_lower.ends_with(".jpeg") {
        return Some("jpeg".to_string());
    }

    None
}

fn estimate_icon_contrast(image: &DynamicImage) -> (f64, f64) {
    let rgba = image.to_rgba8();
    let mut weighted_luminance_sum = 0.0f64;
    let mut alpha_sum = 0.0f64;

    for pixel in rgba.pixels() {
        let [r, g, b, a] = pixel.0;
        let alpha = f64::from(a) / 255.0;
        if alpha <= 0.0 {
            continue;
        }
        let r_lin = srgb_to_linear(f64::from(r) / 255.0);
        let g_lin = srgb_to_linear(f64::from(g) / 255.0);
        let b_lin = srgb_to_linear(f64::from(b) / 255.0);
        let luminance = 0.2126 * r_lin + 0.7152 * g_lin + 0.0722 * b_lin;
        weighted_luminance_sum += luminance * alpha;
        alpha_sum += alpha;
    }

    let avg_luminance = if alpha_sum > 0.0 {
        weighted_luminance_sum / alpha_sum
    } else {
        0.5
    };

    let contrast_on_light = (1.0 + 0.05) / (avg_luminance + 0.05);
    let contrast_on_dark = (avg_luminance + 0.05) / 0.05;
    (contrast_on_light, contrast_on_dark)
}

fn srgb_to_linear(channel: f64) -> f64 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

fn parse_ico_sizes(bytes: &[u8]) -> Vec<(u16, u16)> {
    if bytes.len() < 6 {
        return Vec::new();
    }

    let reserved = u16::from_le_bytes([bytes[0], bytes[1]]);
    let icon_type = u16::from_le_bytes([bytes[2], bytes[3]]);
    let count = u16::from_le_bytes([bytes[4], bytes[5]]) as usize;
    if reserved != 0 || icon_type != 1 {
        return Vec::new();
    }

    let mut sizes = Vec::new();
    for i in 0..count {
        let offset = 6 + i * 16;
        if bytes.len() < offset + 16 {
            break;
        }

        let width = if bytes[offset] == 0 {
            256
        } else {
            bytes[offset] as u16
        };
        let height = if bytes[offset + 1] == 0 {
            256
        } else {
            bytes[offset + 1] as u16
        };

        sizes.push((width, height));
    }

    sizes.sort_unstable();
    sizes.dedup();
    sizes
}
