#![allow(clippy::needless_pass_by_value)]
// IPC command handlers for the PageLens desktop application.

use crate::state::Store;
use pagelens_core::{
    Browser, CrawlOptions, Crawler, SeoAnalyzer, SnapshotExt, SnapshotOptions,
};
use pagelens_db::{
    export_to_file, import_from_file, AnalysisRun, AnalysisSummary, AnalysisType,
    CreateAnalysisRun, Database, HistoryListItem, UpdateAnalysisRun,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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

    ExampleEvent::listen(app, |event| {
        println!("{:?}", event.payload);
    });

    ExampleEvent("Test".into()).emit(app).unwrap();
}

fn specta_builder() -> SpectaBuilder<tauri::Wry> {
    SpectaBuilder::<tauri::Wry>::new()
        .commands(collect_commands![
            // Example commands
            hello_tauri,
            hash256sum,
            store_set_key,
            store_read_key,
            // Analysis commands
            analyze_url,
            crawl_url,
            // History commands
            list_history,
            get_history_item,
            update_history_item,
            delete_history_item,
            delete_all_history,
            export_history,
            import_history,
        ])
        .events(collect_events![AnalysisProgressEvent, ExampleEvent])
}

// ============================================================================
// Example Commands (kept for reference)
// ============================================================================

#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct ExampleEvent(String);

#[tauri::command]
#[specta::specta]
fn hello_tauri() -> String {
    "Hi from Tauri".to_owned()
}

#[tauri::command]
#[specta::specta]
fn hash256sum(hash_input: String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(hash_input.as_bytes());
    let result = hasher.finalize();
    format!("{result:X}")
}

#[tauri::command]
#[specta::specta]
fn store_set_key(key: String, value: String, store: State<Store>) {
    store.add_key_val(key, value);
}

#[tauri::command]
#[specta::specta]
fn store_read_key(key: String, store: State<Store>) -> Option<String> {
    store.read_key(&key)
}

// ============================================================================
// Analysis Commands
// ============================================================================

/// Event emitted during analysis progress.
#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct AnalysisProgressEvent {
    pub stage: String,
    pub message: String,
    pub progress: Option<f64>,
}

/// Input for single page analysis.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AnalyzeUrlInput {
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
}

impl Default for SnapshotOptionsInput {
    fn default() -> Self {
        Self {
            include_html: true,
            include_accessibility_tree: true,
            include_performance_timing: true,
            include_computed_styles: false,
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

    // Emit progress event
    AnalysisProgressEvent {
        stage: "browser".to_string(),
        message: "Launching browser...".to_string(),
        progress: Some(0.1),
    }
    .emit(&app)
    .ok();

    // Launch browser and analyze
    let browser = Browser::launch().await.map_err(|e| e.to_string())?;

    AnalysisProgressEvent {
        stage: "navigation".to_string(),
        message: format!("Navigating to {}...", input.url),
        progress: Some(0.3),
    }
    .emit(&app)
    .ok();

    let page = browser
        .navigate(&input.url)
        .await
        .map_err(|e| e.to_string())?;

    AnalysisProgressEvent {
        stage: "snapshot".to_string(),
        message: "Capturing page snapshot...".to_string(),
        progress: Some(0.5),
    }
    .emit(&app)
    .ok();

    let options: SnapshotOptions = input.options.unwrap_or_default().into();
    let snapshot = page
        .snapshot(options)
        .await
        .map_err(|e| e.to_string())?;

    AnalysisProgressEvent {
        stage: "analysis".to_string(),
        message: "Analyzing SEO...".to_string(),
        progress: Some(0.8),
    }
    .emit(&app)
    .ok();

    let seo_report = SeoAnalyzer::analyze(&snapshot);

    // Create payload JSON string
    let payload_json = serde_json::to_string(&serde_json::json!({
        "snapshot": snapshot,
        "seo_report": seo_report,
    })).map_err(|e| e.to_string())?;

    let summary = AnalysisSummary {
        seo_score: Some(seo_report.score),
        page_count: 1,
        total_issues: seo_report.issues.len() as u32,
        error_count: seo_report
            .issues
            .iter()
            .filter(|i| matches!(i.severity, pagelens_core::Severity::Error))
            .count() as u32,
        warning_count: seo_report
            .issues
            .iter()
            .filter(|i| matches!(i.severity, pagelens_core::Severity::Warning))
            .count() as u32,
        duration_ms: start_time.elapsed().as_millis() as u32,
    };

    AnalysisProgressEvent {
        stage: "storage".to_string(),
        message: "Saving results...".to_string(),
        progress: Some(0.95),
    }
    .emit(&app)
    .ok();

    // Store in database
    let create_input = CreateAnalysisRun {
        url: input.url.clone(),
        name: input.name,
        analysis_type: AnalysisType::Single,
        payload_json,
        summary,
    };

    let db_lock = db.lock().map_err(|e| e.to_string())?;
    let repo = db_lock.analysis_repository();
    let run = repo.create(create_input).map_err(|e| e.to_string())?;

    AnalysisProgressEvent {
        stage: "complete".to_string(),
        message: "Analysis complete!".to_string(),
        progress: Some(1.0),
    }
    .emit(&app)
    .ok();

    Ok(AnalysisResult { run })
}

/// Input for crawl analysis.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CrawlUrlInput {
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

    AnalysisProgressEvent {
        stage: "browser".to_string(),
        message: "Launching browser...".to_string(),
        progress: Some(0.1),
    }
    .emit(&app)
    .ok();

    let browser = Browser::launch().await.map_err(|e| e.to_string())?;

    AnalysisProgressEvent {
        stage: "crawl".to_string(),
        message: format!("Starting crawl from {}...", input.url),
        progress: Some(0.2),
    }
    .emit(&app)
    .ok();

    let options: CrawlOptions = input.options.unwrap_or_default().into();
    let crawler = Crawler::new(&browser, options);
    let crawl_result = crawler
        .crawl(&input.url)
        .await
        .map_err(|e| e.to_string())?;

    AnalysisProgressEvent {
        stage: "analysis".to_string(),
        message: format!(
            "Crawled {} pages, analyzing...",
            crawl_result.stats.crawled_pages
        ),
        progress: Some(0.8),
    }
    .emit(&app)
    .ok();

    // Create payload JSON string
    let payload_json = serde_json::to_string(&crawl_result).map_err(|e| e.to_string())?;

    let summary = AnalysisSummary {
        seo_score: Some(crawl_result.aggregate.avg_seo_score),
        page_count: crawl_result.stats.crawled_pages as u32,
        total_issues: crawl_result.aggregate.total_issues as u32,
        error_count: crawl_result.aggregate.total_errors as u32,
        warning_count: crawl_result.aggregate.total_warnings as u32,
        duration_ms: start_time.elapsed().as_millis() as u32,
    };

    AnalysisProgressEvent {
        stage: "storage".to_string(),
        message: "Saving results...".to_string(),
        progress: Some(0.95),
    }
    .emit(&app)
    .ok();

    let create_input = CreateAnalysisRun {
        url: input.url.clone(),
        name: input.name,
        analysis_type: AnalysisType::Crawl,
        payload_json,
        summary,
    };

    let db_lock = db.lock().map_err(|e| e.to_string())?;
    let repo = db_lock.analysis_repository();
    let run = repo.create(create_input).map_err(|e| e.to_string())?;

    AnalysisProgressEvent {
        stage: "complete".to_string(),
        message: "Crawl complete!".to_string(),
        progress: Some(1.0),
    }
    .emit(&app)
    .ok();

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
