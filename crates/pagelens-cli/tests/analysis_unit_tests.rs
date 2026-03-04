use clap::ValueEnum;
use pagelens_cli::cli::{
    analyze_contrast, build_performance_report, parse_headers, resolve_effective_toggles,
    AnalysisMode,
};

#[test]
fn parse_headers_accepts_multiple_entries() {
    let parsed = parse_headers(&[
        "Authorization: Bearer token:abc".to_string(),
        "X-Org: pagelens".to_string(),
    ])
    .expect("headers should parse");

    assert_eq!(
        parsed.get("Authorization"),
        Some(&"Bearer token:abc".to_string())
    );
    assert_eq!(parsed.get("X-Org"), Some(&"pagelens".to_string()));
}

#[test]
fn parse_headers_rejects_invalid_format() {
    let err = parse_headers(&["InvalidHeaderWithoutColon".to_string()]).expect_err("should fail");
    assert!(err.contains("Expected KEY:VALUE"));
}

#[test]
fn performance_report_computes_relative_timings() {
    let timing = pagelens_core::snapshot::PerformanceTiming {
        navigation_start: 1000,
        dom_interactive: Some(1300),
        dom_content_loaded: Some(1400),
        load_complete: Some(2000),
        response_start: Some(1100),
        first_paint: Some(1250),
        first_contentful_paint: Some(1350),
        largest_contentful_paint: Some(1600),
        cumulative_layout_shift: Some(0.08),
        interaction_to_next_paint: Some(190),
    };

    let report = build_performance_report(&timing);
    assert_eq!(report.ttfb_ms, Some(100));
    assert_eq!(report.first_paint_ms, Some(250));
    assert_eq!(report.first_contentful_paint_ms, Some(350));
}

#[test]
fn contrast_report_flags_low_ratio_pair() {
    let styles = vec![
        pagelens_core::snapshot::ComputedStyle {
            selector: "p.low".to_string(),
            color: Some("rgb(120, 120, 120)".to_string()),
            background_color: Some("rgb(140, 140, 140)".to_string()),
            font_size: None,
            font_weight: None,
        },
        pagelens_core::snapshot::ComputedStyle {
            selector: "p.high".to_string(),
            color: Some("rgb(0, 0, 0)".to_string()),
            background_color: Some("rgb(255, 255, 255)".to_string()),
            font_size: None,
            font_weight: None,
        },
    ];

    let report = analyze_contrast(&styles);
    assert_eq!(report.analyzed_elements, 2);
    assert!(report.failing_elements >= 1);
    assert!(report.issues.iter().any(|issue| issue.selector == "p.low"));
}

#[test]
fn full_mode_enables_site_files_checks() {
    let toggles = resolve_effective_toggles(AnalysisMode::Full, false, false, false, false);
    assert!(toggles.assets);
    assert!(toggles.perf);
    assert!(toggles.contrast);
    assert!(toggles.site_files);
}

#[test]
fn seo_only_disables_optional_checks() {
    let toggles = resolve_effective_toggles(AnalysisMode::SeoOnly, true, true, true, true);
    assert!(!toggles.assets);
    assert!(!toggles.perf);
    assert!(!toggles.contrast);
    assert!(!toggles.site_files);
}

#[test]
fn analysis_mode_defaults_include_full() {
    let variants = AnalysisMode::value_variants();
    assert!(variants.contains(&AnalysisMode::Full));
}
