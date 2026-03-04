use pagelens_db::error::Error;
use pagelens_db::{AnalysisRunStatus, AnalysisSummary, AnalysisType, CreateAnalysisRun, Database};
use tempfile::TempDir;

fn create_test_db() -> (TempDir, Database) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("pagelens-test.db");
    let db = Database::open(&db_path).expect("open test db");
    (temp_dir, db)
}

fn sample_run_input(id: &str, status: AnalysisRunStatus) -> CreateAnalysisRun {
    CreateAnalysisRun {
        id: Some(id.to_string()),
        url: "https://example.com".to_string(),
        name: Some("benchmark".to_string()),
        analysis_type: AnalysisType::HttpBenchmark,
        payload_json: "{}".to_string(),
        summary: AnalysisSummary::default(),
        status,
        current_stage: Some("queued".to_string()),
        current_message: Some("waiting".to_string()),
        progress: Some(0.0),
    }
}

#[test]
fn persists_live_snapshot_for_active_run() {
    let (_temp_dir, db) = create_test_db();
    let repo = db.analysis_repository();
    let run = repo
        .create(sample_run_input("run-active", AnalysisRunStatus::Running))
        .expect("create run");

    let payload = r#"{"stats":{"requests":42}}"#;
    let summary = AnalysisSummary {
        seo_score: None,
        page_count: 42,
        total_issues: 0,
        error_count: 2,
        warning_count: 0,
        duration_ms: 1_337,
    };

    let updated = repo
        .update_benchmark_live_snapshot_if_active(
            &run.id,
            payload,
            &summary,
            Some("benchmarking"),
            Some("streaming live stats"),
            Some(0.42),
        )
        .expect("persist live snapshot");

    assert!(updated);

    let refreshed = repo.get(&run.id).expect("fetch run");
    assert_eq!(refreshed.payload_json, payload);
    assert_eq!(refreshed.summary.page_count, 42);
    assert_eq!(refreshed.summary.error_count, 2);
    assert_eq!(refreshed.summary.duration_ms, 1_337);
    assert_eq!(refreshed.current_stage.as_deref(), Some("benchmarking"));
    assert_eq!(
        refreshed.current_message.as_deref(),
        Some("streaming live stats")
    );
    assert_eq!(refreshed.progress, Some(0.42));
}

#[test]
fn skips_snapshot_for_inactive_run() {
    let (_temp_dir, db) = create_test_db();
    let repo = db.analysis_repository();
    let run = repo
        .create(sample_run_input("run-done", AnalysisRunStatus::Completed))
        .expect("create run");

    let summary = AnalysisSummary {
        seo_score: None,
        page_count: 99,
        total_issues: 0,
        error_count: 0,
        warning_count: 0,
        duration_ms: 500,
    };

    let updated = repo
        .update_benchmark_live_snapshot_if_active(
            &run.id,
            r#"{"stats":{"requests":99}}"#,
            &summary,
            Some("benchmarking"),
            Some("should not persist"),
            Some(0.99),
        )
        .expect("attempt live snapshot");

    assert!(!updated);

    let refreshed = repo.get(&run.id).expect("fetch run");
    assert_eq!(refreshed.payload_json, "{}");
    assert_eq!(refreshed.summary.page_count, 0);
    assert_eq!(refreshed.current_stage.as_deref(), Some("queued"));
    assert_eq!(refreshed.current_message.as_deref(), Some("waiting"));
    assert_eq!(refreshed.progress, Some(0.0));
}

#[test]
fn returns_not_found_for_missing_run() {
    let (_temp_dir, db) = create_test_db();
    let repo = db.analysis_repository();

    let err = repo
        .update_benchmark_live_snapshot_if_active(
            "missing-run-id",
            "{}",
            &AnalysisSummary::default(),
            Some("benchmarking"),
            Some("not found"),
            Some(0.1),
        )
        .expect_err("missing id should error");

    assert!(matches!(err, Error::AnalysisRunNotFound { .. }));
}
