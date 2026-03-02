#![allow(clippy::needless_pass_by_value)]
// IPC command handlers for the PageLens desktop application.

use pagelens_core::{
    Browser, CrawlOptions, Crawler, SeoAnalyzer, SiteFilesAnalyzer, SnapshotExt, SnapshotOptions,
};
use pagelens_db::{
    export_to_file, import_from_file, AnalysisPageResult, AnalysisRun, AnalysisRunStatus,
    AnalysisSummary, AnalysisType, CreateAnalysisPageResult, CreateAnalysisRun, Database,
    HistoryListItem, UpdateAnalysisRun,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use specta_typescript::Typescript;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{App, AppHandle, Builder, State, Wry};
use tauri_specta::{collect_commands, collect_events, Builder as SpectaBuilder, Event};

/// Register all IPC handlers and generate TypeScript bindings.
pub fn register_ipc_handlers(app: Builder<Wry>) -> Builder<Wry> {
    let builder = specta_builder();

    // Generate the IPC types
    #[cfg(debug_assertions)]
    specta_builder()
        .export(Typescript::default(), "../src/lib/ipc.ts")
        .expect("Failed to export typescript bindings");

    app.invoke_handler(builder.invoke_handler())
}

pub fn mount_ipc_events(app: &mut App<Wry>) {
    specta_builder().mount_events(app);
}

fn specta_builder() -> SpectaBuilder<tauri::Wry> {
    SpectaBuilder::<tauri::Wry>::new()
        .commands(collect_commands![
            // Analysis commands
            analyze_url,
            crawl_url,
            analyze_site_files,
            list_analysis_page_results,
            // History commands
            list_history,
            get_history_item,
            update_history_item,
            delete_history_item,
            delete_all_history,
            export_history,
            export_history_item,
            import_history,
        ])
        .events(collect_events![AnalysisProgressEvent, AnalysisPageEvent])
}

// ============================================================================
// Analysis Commands
// ============================================================================

/// Event emitted during analysis progress.
#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct AnalysisProgressEvent {
    pub run_id: String,
    pub stage: String,
    pub message: String,
    pub progress: Option<f64>,
    pub page_count: Option<u32>,
    pub success_count: Option<u32>,
    pub failed_count: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct AnalysisPageEvent {
    pub run_id: String,
    pub page: AnalysisPageResult,
}

fn emit_progress(
    app: &AppHandle,
    run_id: &str,
    stage: &str,
    message: String,
    progress: Option<f64>,
    page_count: Option<u32>,
    success_count: Option<u32>,
    failed_count: Option<u32>,
) {
    AnalysisProgressEvent {
        run_id: run_id.to_string(),
        stage: stage.to_string(),
        message,
        progress,
        page_count,
        success_count,
        failed_count,
    }
    .emit(app)
    .ok();
}

/// Input for single page analysis.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AnalyzeUrlInput {
    pub run_id: Option<String>,
    pub url: String,
    pub name: Option<String>,
    pub options: Option<SnapshotOptionsInput>,
}

/// Snapshot options input.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SnapshotOptionsInput {
    pub include_html: bool,
    pub include_accessibility_tree: bool,
    pub include_performance_timing: bool,
    pub include_computed_styles: bool,
    #[serde(default = "default_include_network_metadata")]
    pub include_network_metadata: bool,
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
        SnapshotOptions {
            include_html: input.include_html,
            include_accessibility_tree: input.include_accessibility_tree,
            include_performance_timing: input.include_performance_timing,
            include_computed_styles: input.include_computed_styles,
            include_network_metadata: input.include_network_metadata,
        }
    }
}

/// Result of single page analysis.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AnalysisResult {
    pub run: AnalysisRun,
}

/// Analyze a single URL and store the result.
#[tauri::command]
#[specta::specta]
pub async fn analyze_url(
    input: AnalyzeUrlInput,
    db: State<'_, Mutex<Database>>,
    app: AppHandle,
) -> Result<AnalysisResult, String> {
    let start_time = std::time::Instant::now();
    let run_id = input
        .run_id
        .clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    {
        let db_lock = db.lock().map_err(|e| e.to_string())?;
        let repo = db_lock.analysis_repository();
        repo.create(CreateAnalysisRun {
            id: Some(run_id.clone()),
            url: input.url.clone(),
            name: input.name.clone(),
            analysis_type: AnalysisType::Single,
            payload_json: "{}".to_string(),
            summary: AnalysisSummary::default(),
            status: AnalysisRunStatus::Running,
            current_stage: Some("browser".to_string()),
            current_message: Some("Launching browser...".to_string()),
            progress: Some(0.05),
        })
        .map_err(|e| e.to_string())?;
    }

    emit_progress(
        &app,
        &run_id,
        "browser",
        "Launching browser...".to_string(),
        Some(0.1),
        Some(0),
        Some(0),
        Some(0),
    );

    let execute: std::result::Result<AnalysisResult, String> = async {
        let browser = Browser::launch().await.map_err(|e| e.to_string())?;

        {
            let db_lock = db.lock().map_err(|e| e.to_string())?;
            let repo = db_lock.analysis_repository();
            repo.update_run_state(
                &run_id,
                AnalysisRunStatus::Running,
                Some("navigation"),
                Some("Navigating to target URL..."),
                Some(0.3),
            )
            .map_err(|e| e.to_string())?;
        }
        emit_progress(
            &app,
            &run_id,
            "navigation",
            format!("Navigating to {}...", input.url),
            Some(0.3),
            Some(0),
            Some(0),
            Some(0),
        );

        let page = browser
            .navigate(&input.url)
            .await
            .map_err(|e| e.to_string())?;

        {
            let db_lock = db.lock().map_err(|e| e.to_string())?;
            let repo = db_lock.analysis_repository();
            repo.update_run_state(
                &run_id,
                AnalysisRunStatus::Running,
                Some("snapshot"),
                Some("Capturing page snapshot..."),
                Some(0.5),
            )
            .map_err(|e| e.to_string())?;
        }
        emit_progress(
            &app,
            &run_id,
            "snapshot",
            "Capturing page snapshot...".to_string(),
            Some(0.5),
            Some(0),
            Some(0),
            Some(0),
        );

        let options: SnapshotOptions = input.options.unwrap_or_default().into();
        let snapshot = page
            .snapshot(options)
            .await
            .map_err(|e| e.to_string())?;

        {
            let db_lock = db.lock().map_err(|e| e.to_string())?;
            let repo = db_lock.analysis_repository();
            repo.update_run_state(
                &run_id,
                AnalysisRunStatus::Running,
                Some("analysis"),
                Some("Analyzing SEO..."),
                Some(0.8),
            )
            .map_err(|e| e.to_string())?;
        }
        emit_progress(
            &app,
            &run_id,
            "analysis",
            "Analyzing SEO...".to_string(),
            Some(0.8),
            Some(1),
            Some(0),
            Some(0),
        );

        let seo_report = SeoAnalyzer::analyze(&snapshot);
        let error_count = seo_report
            .issues
            .iter()
            .filter(|i| matches!(i.severity, pagelens_core::Severity::Error))
            .count() as u32;
        let warning_count = seo_report
            .issues
            .iter()
            .filter(|i| matches!(i.severity, pagelens_core::Severity::Warning))
            .count() as u32;

        {
            let db_lock = db.lock().map_err(|e| e.to_string())?;
            let repo = db_lock.analysis_repository();
            let page_row = repo
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
                })
                .map_err(|e| e.to_string())?;

            AnalysisPageEvent {
                run_id: run_id.clone(),
                page: page_row,
            }
            .emit(&app)
            .ok();
        }

        let payload_json = serde_json::to_string(&serde_json::json!({
            "snapshot": snapshot,
            "seo_report": seo_report,
        }))
        .map_err(|e| e.to_string())?;

        let summary = AnalysisSummary {
            seo_score: Some(seo_report.score),
            page_count: 1,
            total_issues: seo_report.issues.len() as u32,
            error_count,
            warning_count,
            duration_ms: start_time.elapsed().as_millis() as u32,
        };

        {
            let db_lock = db.lock().map_err(|e| e.to_string())?;
            let repo = db_lock.analysis_repository();
            repo.complete_run(&run_id, &payload_json, &summary)
                .map_err(|e| e.to_string())?;
        }

        emit_progress(
            &app,
            &run_id,
            "complete",
            "Analysis complete!".to_string(),
            Some(1.0),
            Some(1),
            Some(1),
            Some(0),
        );

        let db_lock = db.lock().map_err(|e| e.to_string())?;
        let run = db_lock
            .analysis_repository()
            .get(&run_id)
            .map_err(|e| e.to_string())?;
        Ok(AnalysisResult { run })
    }
    .await;

    match execute {
        Ok(result) => Ok(result),
        Err(error) => {
            if let Ok(db_lock) = db.lock() {
                let repo = db_lock.analysis_repository();
                let _ = repo.update_run_state(
                    &run_id,
                    AnalysisRunStatus::Failed,
                    Some("failed"),
                    Some(&error),
                    None,
                );

                let _ = repo.insert_page_result(CreateAnalysisPageResult {
                    run_id: run_id.clone(),
                    url: input.url.clone(),
                    depth: 0,
                    success: false,
                    seo_score: None,
                    total_issues: 0,
                    error_count: 0,
                    warning_count: 0,
                    links_found_count: 0,
                    error_message: Some(error.clone()),
                });
            }

            emit_progress(
                &app,
                &run_id,
                "failed",
                error.clone(),
                None,
                Some(1),
                Some(0),
                Some(1),
            );

            Err(error)
        }
    }
}

/// Input for crawl analysis.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CrawlUrlInput {
    pub run_id: Option<String>,
    pub url: String,
    pub name: Option<String>,
    pub options: Option<CrawlOptionsInput>,
}

/// Crawl options input.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CrawlOptionsInput {
    pub max_pages: u32,
    pub max_depth: u32,
    pub follow_external_links: bool,
    pub same_subdomain_only: bool,
    pub page_timeout_ms: u32,
    pub delay_ms: u32,
    pub max_concurrency: u32,
}

impl Default for CrawlOptionsInput {
    fn default() -> Self {
        Self {
            max_pages: 50,
            max_depth: 3,
            follow_external_links: false,
            same_subdomain_only: true,
            page_timeout_ms: 30000,
            delay_ms: 100,
            max_concurrency: 4,
        }
    }
}

impl From<CrawlOptionsInput> for CrawlOptions {
    fn from(input: CrawlOptionsInput) -> Self {
        CrawlOptions {
            max_pages: input.max_pages as usize,
            max_depth: input.max_depth as usize,
            follow_external_links: input.follow_external_links,
            same_subdomain_only: input.same_subdomain_only,
            page_timeout_ms: input.page_timeout_ms as u64,
            delay_ms: input.delay_ms as u64,
            max_concurrency: input.max_concurrency as usize,
        }
    }
}

/// Crawl a URL and store the result.
#[tauri::command]
#[specta::specta]
pub async fn crawl_url(
    input: CrawlUrlInput,
    db: State<'_, Mutex<Database>>,
    app: AppHandle,
) -> Result<AnalysisResult, String> {
    let start_time = std::time::Instant::now();
    let run_id = input
        .run_id
        .clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    {
        let db_lock = db.lock().map_err(|e| e.to_string())?;
        let repo = db_lock.analysis_repository();
        repo.create(CreateAnalysisRun {
            id: Some(run_id.clone()),
            url: input.url.clone(),
            name: input.name.clone(),
            analysis_type: AnalysisType::Crawl,
            payload_json: "{}".to_string(),
            summary: AnalysisSummary::default(),
            status: AnalysisRunStatus::Running,
            current_stage: Some("browser".to_string()),
            current_message: Some("Launching browser...".to_string()),
            progress: Some(0.05),
        })
        .map_err(|e| e.to_string())?;
    }

    emit_progress(
        &app,
        &run_id,
        "browser",
        "Launching browser...".to_string(),
        Some(0.1),
        Some(0),
        Some(0),
        Some(0),
    );

    let execute: std::result::Result<AnalysisResult, String> = async {
        let browser = Browser::launch().await.map_err(|e| e.to_string())?;

        {
            let db_lock = db.lock().map_err(|e| e.to_string())?;
            let repo = db_lock.analysis_repository();
            repo.update_run_state(
                &run_id,
                AnalysisRunStatus::Running,
                Some("crawl"),
                Some("Starting crawl..."),
                Some(0.2),
            )
            .map_err(|e| e.to_string())?;
        }

        emit_progress(
            &app,
            &run_id,
            "crawl",
            format!("Starting crawl from {}...", input.url),
            Some(0.2),
            Some(0),
            Some(0),
            Some(0),
        );

        let options: CrawlOptions = input.options.unwrap_or_default().into();
        let crawler = Crawler::new(&browser, options);
        let crawl_result = crawler
            .crawl_with_callback(&input.url, |page, stats| {
                if let Ok(db_lock) = db.lock() {
                    let repo = db_lock.analysis_repository();
                    let error_count = page
                        .seo_report
                        .issues
                        .iter()
                        .filter(|i| matches!(i.severity, pagelens_core::Severity::Error))
                        .count() as u32;
                    let warning_count = page
                        .seo_report
                        .issues
                        .iter()
                        .filter(|i| matches!(i.severity, pagelens_core::Severity::Warning))
                        .count() as u32;

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
                        AnalysisPageEvent {
                            run_id: run_id.clone(),
                            page: page_row,
                        }
                        .emit(&app)
                        .ok();
                    }

                    let scanned = (stats.crawled_pages + stats.failed_pages) as u32;
                    let total = stats.total_pages as u32;
                    let progress = if total > 0 {
                        0.2 + ((scanned as f64 / total as f64) * 0.6)
                    } else {
                        0.2
                    };

                    let _ = repo.update_run_state(
                        &run_id,
                        AnalysisRunStatus::Running,
                        Some("crawl"),
                        Some(&format!(
                            "Scanned {} pages ({} successful, {} failed)",
                            scanned, stats.crawled_pages, stats.failed_pages
                        )),
                        Some(progress.min(0.8)),
                    );

                    emit_progress(
                        &app,
                        &run_id,
                        "crawl",
                        format!(
                            "Scanned {} pages ({} successful, {} failed)",
                            scanned, stats.crawled_pages, stats.failed_pages
                        ),
                        Some(progress.min(0.8)),
                        Some(total),
                        Some(stats.crawled_pages as u32),
                        Some(stats.failed_pages as u32),
                    );
                }
            })
            .await
            .map_err(|e| e.to_string())?;

        {
            let db_lock = db.lock().map_err(|e| e.to_string())?;
            let repo = db_lock.analysis_repository();
            repo.update_run_state(
                &run_id,
                AnalysisRunStatus::Running,
                Some("analysis"),
                Some("Computing aggregate metrics..."),
                Some(0.9),
            )
            .map_err(|e| e.to_string())?;
        }

        emit_progress(
            &app,
            &run_id,
            "analysis",
            "Computing aggregate metrics...".to_string(),
            Some(0.9),
            Some(crawl_result.stats.total_pages as u32),
            Some(crawl_result.stats.crawled_pages as u32),
            Some(crawl_result.stats.failed_pages as u32),
        );

        let payload_json = serde_json::to_string(&crawl_result).map_err(|e| e.to_string())?;
        let summary = AnalysisSummary {
            seo_score: Some(crawl_result.aggregate.avg_seo_score),
            page_count: crawl_result.stats.crawled_pages as u32,
            total_issues: crawl_result.aggregate.total_issues as u32,
            error_count: crawl_result.aggregate.total_errors as u32,
            warning_count: crawl_result.aggregate.total_warnings as u32,
            duration_ms: start_time.elapsed().as_millis() as u32,
        };

        {
            let db_lock = db.lock().map_err(|e| e.to_string())?;
            let repo = db_lock.analysis_repository();
            repo.complete_run(&run_id, &payload_json, &summary)
                .map_err(|e| e.to_string())?;
        }

        emit_progress(
            &app,
            &run_id,
            "complete",
            "Crawl complete!".to_string(),
            Some(1.0),
            Some(crawl_result.stats.total_pages as u32),
            Some(crawl_result.stats.crawled_pages as u32),
            Some(crawl_result.stats.failed_pages as u32),
        );

        let db_lock = db.lock().map_err(|e| e.to_string())?;
        let run = db_lock
            .analysis_repository()
            .get(&run_id)
            .map_err(|e| e.to_string())?;

        Ok(AnalysisResult { run })
    }
    .await;

    match execute {
        Ok(result) => Ok(result),
        Err(error) => {
            if let Ok(db_lock) = db.lock() {
                let repo = db_lock.analysis_repository();
                let _ = repo.update_run_state(
                    &run_id,
                    AnalysisRunStatus::Failed,
                    Some("failed"),
                    Some(&error),
                    None,
                );
            }

            emit_progress(
                &app,
                &run_id,
                "failed",
                error.clone(),
                None,
                None,
                None,
                None,
            );

            Err(error)
        }
    }
}

// ============================================================================
// Site Files Commands
// ============================================================================

/// Input for site files analysis.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SiteFilesInput {
    pub run_id: Option<String>,
    pub url: String,
    pub crawled_urls: Option<Vec<String>>,
}

/// Analyze site files (robots.txt, sitemaps, etc.) for a given URL.
#[tauri::command]
#[specta::specta]
pub async fn analyze_site_files(
    input: SiteFilesInput,
    db: State<'_, Mutex<Database>>,
    app: AppHandle,
) -> Result<AnalysisResult, String> {
    let run_id = input
        .run_id
        .clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    {
        let db_lock = db.lock().map_err(|e| e.to_string())?;
        let repo = db_lock.analysis_repository();
        repo.create(CreateAnalysisRun {
            id: Some(run_id.clone()),
            url: input.url.clone(),
            name: Some("Site files analysis".to_string()),
            analysis_type: AnalysisType::Single,
            payload_json: "{}".to_string(),
            summary: AnalysisSummary::default(),
            status: AnalysisRunStatus::Running,
            current_stage: Some("site_files".to_string()),
            current_message: Some("Analyzing site files...".to_string()),
            progress: Some(0.1),
        })
        .map_err(|e| e.to_string())?;
    }

    emit_progress(
        &app,
        &run_id,
        "site_files",
        "Analyzing site files...".to_string(),
        Some(0.2),
        Some(1),
        Some(0),
        Some(0),
    );

    let crawled_urls = input.crawled_urls.unwrap_or_default();
    let report = SiteFilesAnalyzer::analyze_with_crawled_urls(&input.url, &crawled_urls)
        .await
        .map_err(|e| e.to_string())?;

    let payload_json = serde_json::to_string(&serde_json::json!({
        "site_files_report": report,
    }))
    .map_err(|e| e.to_string())?;
    let summary = AnalysisSummary {
        seo_score: None,
        page_count: 1,
        total_issues: 0,
        error_count: 0,
        warning_count: 0,
        duration_ms: 0,
    };

    {
        let db_lock = db.lock().map_err(|e| e.to_string())?;
        let repo = db_lock.analysis_repository();
        repo.insert_page_result(CreateAnalysisPageResult {
            run_id: run_id.clone(),
            url: input.url.clone(),
            depth: 0,
            success: true,
            seo_score: None,
            total_issues: 0,
            error_count: 0,
            warning_count: 0,
            links_found_count: 0,
            error_message: None,
        })
        .map_err(|e| e.to_string())?;
        repo.complete_run(&run_id, &payload_json, &summary)
            .map_err(|e| e.to_string())?;
    }

    emit_progress(
        &app,
        &run_id,
        "complete",
        "Site files analysis complete!".to_string(),
        Some(1.0),
        Some(1),
        Some(1),
        Some(0),
    );

    let db_lock = db.lock().map_err(|e| e.to_string())?;
    let run = db_lock
        .analysis_repository()
        .get(&run_id)
        .map_err(|e| e.to_string())?;

    Ok(AnalysisResult { run })
}

// ============================================================================
// History Commands
// ============================================================================

/// List all analysis history items.
#[tauri::command]
#[specta::specta]
pub fn list_history(
    db: State<Mutex<Database>>,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<HistoryListItem>, String> {
    let db_lock = db.lock().map_err(|e| e.to_string())?;
    let repo = db_lock.analysis_repository();
    let items = repo
        .list(limit.unwrap_or(100) as i64, offset.unwrap_or(0) as i64)
        .map_err(|e| e.to_string())?;
    Ok(items)
}

/// Get a single history item by ID.
#[tauri::command]
#[specta::specta]
pub fn get_history_item(
    id: String,
    db: State<Mutex<Database>>,
) -> Result<AnalysisRun, String> {
    let db_lock = db.lock().map_err(|e| e.to_string())?;
    let repo = db_lock.analysis_repository();
    let run = repo.get(&id).map_err(|e| e.to_string())?;
    Ok(run)
}

#[tauri::command]
#[specta::specta]
pub fn list_analysis_page_results(
    id: String,
    db: State<Mutex<Database>>,
) -> Result<Vec<AnalysisPageResult>, String> {
    let db_lock = db.lock().map_err(|e| e.to_string())?;
    let repo = db_lock.analysis_repository();
    let rows = repo.list_page_results(&id).map_err(|e| e.to_string())?;
    Ok(rows)
}

/// Update a history item (e.g., rename).
#[tauri::command]
#[specta::specta]
pub fn update_history_item(
    id: String,
    input: UpdateAnalysisRun,
    db: State<Mutex<Database>>,
) -> Result<AnalysisRun, String> {
    let db_lock = db.lock().map_err(|e| e.to_string())?;
    let repo = db_lock.analysis_repository();
    let run = repo.update(&id, input).map_err(|e| e.to_string())?;
    Ok(run)
}

/// Delete a single history item.
#[tauri::command]
#[specta::specta]
pub fn delete_history_item(
    id: String,
    db: State<Mutex<Database>>,
) -> Result<(), String> {
    let db_lock = db.lock().map_err(|e| e.to_string())?;
    let repo = db_lock.analysis_repository();
    repo.delete(&id).map_err(|e| e.to_string())?;
    Ok(())
}

/// Delete all history items.
#[tauri::command]
#[specta::specta]
pub fn delete_all_history(db: State<Mutex<Database>>) -> Result<u32, String> {
    let db_lock = db.lock().map_err(|e| e.to_string())?;
    let repo = db_lock.analysis_repository();
    let count = repo.delete_all().map_err(|e| e.to_string())?;
    Ok(count as u32)
}

/// Export history to a JSON file.
#[tauri::command]
#[specta::specta]
pub fn export_history(
    path: String,
    db: State<Mutex<Database>>,
) -> Result<(), String> {
    let db_lock = db.lock().map_err(|e| e.to_string())?;
    let path = PathBuf::from(path);
    export_to_file(&db_lock, &path).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn export_history_item(
    id: String,
    path: String,
    db: State<Mutex<Database>>,
) -> Result<(), String> {
    let db_lock = db.lock().map_err(|e| e.to_string())?;
    let repo = db_lock.analysis_repository();

    let run = repo.get(&id).map_err(|e| e.to_string())?;
    let pages = repo.list_page_results(&id).map_err(|e| e.to_string())?;
    let exported_at_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();

    let payload = serde_json::json!({
        "version": "1.0.0",
        "exported_at_unix": exported_at_unix,
        "run": run,
        "pages": pages,
    });
    let json = serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?;
    std::fs::write(PathBuf::from(path), json).map_err(|e| e.to_string())?;

    Ok(())
}

/// Import history from a JSON file.
#[tauri::command]
#[specta::specta]
pub fn import_history(
    path: String,
    db: State<Mutex<Database>>,
) -> Result<u32, String> {
    let db_lock = db.lock().map_err(|e| e.to_string())?;
    let path = PathBuf::from(path);
    let count = import_from_file(&db_lock, &path).map_err(|e| e.to_string())?;
    Ok(count as u32)
}
