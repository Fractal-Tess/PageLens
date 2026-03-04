use pagelens_core::snapshot::{MainResourceNetwork, PerformanceTiming, ReferencedAssets, Snapshot};
use pagelens_core::{ScoreConfig, SeoAnalyzer, Severity};

fn snapshot_with_html(html: &str) -> Snapshot {
    Snapshot {
        url: "http://test.local".to_string(),
        title: String::new(),
        html: html.to_string(),
        accessibility_tree: Vec::new(),
        performance_timing: PerformanceTiming {
            navigation_start: 0,
            dom_interactive: None,
            dom_content_loaded: None,
            load_complete: None,
            response_start: None,
            first_paint: None,
            first_contentful_paint: None,
            largest_contentful_paint: None,
            cumulative_layout_shift: None,
            interaction_to_next_paint: None,
        },
        computed_styles: Vec::new(),
        referenced_assets: ReferencedAssets::default(),
        favicon_url: None,
        main_resource_network: MainResourceNetwork::default(),
        network_requests: Vec::new(),
        capture_issues: Vec::new(),
    }
}

#[test]
fn analyze_empty_html() {
    let snapshot = snapshot_with_html("");
    let report = SeoAnalyzer::analyze(&snapshot);

    assert!(report.meta.title.is_none());
    assert!(!report.meta.charset);
    assert!(!report.meta.viewport);
    assert_eq!(report.headings.h1_count, 0);
    assert!(report.score < 50.0, "Empty page should score poorly");

    let has_missing_title = report
        .issues
        .iter()
        .any(|i| i.message.contains("Missing title"));
    let has_missing_charset = report
        .issues
        .iter()
        .any(|i| i.message.contains("Missing charset"));
    let has_no_h1 = report.issues.iter().any(|i| i.message.contains("No H1"));
    assert!(has_missing_title);
    assert!(has_missing_charset);
    assert!(has_no_h1);
}

#[test]
fn analyze_well_formed_page() {
    let html = r#"
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>My Great Website - Home Page</title>
            <meta name="description" content="A well-optimized website with proper meta tags for good SEO performance.">
            <meta property="og:title" content="My Great Website">
            <meta property="og:description" content="A well-optimized website">
            <meta property="og:image" content="https://example.com/image.png">
            <link rel="canonical" href="https://example.com/">
            <script type="application/ld+json">{"@type": "WebSite", "name": "My Great Website"}</script>
        </head>
        <body>
            <h1>Welcome to My Website</h1>
            <h2>About Us</h2>
            <p>Some content here.</p>
            <h2>Services</h2>
            <img src="photo.jpg" alt="A descriptive alt text">
        </body>
        </html>
    "#;

    let snapshot = snapshot_with_html(html);
    let report = SeoAnalyzer::analyze(&snapshot);

    assert!(report.meta.charset);
    assert!(report.meta.viewport);
    assert_eq!(
        report.meta.title.as_deref(),
        Some("My Great Website - Home Page")
    );
    assert!(report.meta.description.is_some());
    assert_eq!(report.meta.language.as_deref(), Some("en"));
    assert_eq!(report.headings.h1_count, 1);
    assert_eq!(report.headings.h2_count, 2);
    assert!(report.canonical_url.is_some());
    assert!(report.open_graph.title.is_some());
    assert!(!report.structured_data.is_empty());
    assert_eq!(report.images.len(), 1);
    assert!(report.images[0].has_alt);
    assert!(
        report.score >= 90.0,
        "Well-formed page should score high, got {}",
        report.score
    );
}

#[test]
fn analyze_multiple_h1s() {
    let html = r#"
        <html><head><meta charset="utf-8"><title>Test Page Title</title></head>
        <body>
            <h1>First Heading</h1>
            <h1>Second Heading</h1>
            <h1>Third Heading</h1>
        </body></html>
    "#;

    let snapshot = snapshot_with_html(html);
    let report = SeoAnalyzer::analyze(&snapshot);

    assert_eq!(report.headings.h1_count, 3);
    let h1_issue = report
        .issues
        .iter()
        .find(|i| i.message.contains("Multiple H1"));
    assert!(h1_issue.is_some());
    assert_eq!(
        h1_issue.expect("issue expected").severity,
        Severity::Warning
    );
}

#[test]
fn analyze_images_without_alt() {
    let html = r#"
        <html><head><meta charset="utf-8"><title>Image Test Page</title></head>
        <body>
            <h1>Images</h1>
            <img src="good.jpg" alt="A good image">
            <img src="bad.jpg">
            <img src="empty-alt.jpg" alt="">
        </body></html>
    "#;

    let snapshot = snapshot_with_html(html);
    let report = SeoAnalyzer::analyze(&snapshot);

    assert_eq!(report.images.len(), 3);
    assert!(report.images[0].has_alt);
    assert!(!report.images[1].has_alt);
    assert!(report.images[2].has_alt);

    let missing_alt = report
        .issues
        .iter()
        .filter(|i| i.message.contains("missing alt"))
        .count();
    let empty_alt = report
        .issues
        .iter()
        .filter(|i| i.message.contains("empty alt"))
        .count();
    assert_eq!(missing_alt, 1);
    assert_eq!(empty_alt, 1);
}

#[test]
fn analyze_skipped_heading_levels() {
    let html = r#"
        <html><head><meta charset="utf-8"><title>Heading Skip Test</title></head>
        <body>
            <h1>Title</h1>
            <h3>Jumped from H1 to H3</h3>
        </body></html>
    "#;

    let snapshot = snapshot_with_html(html);
    let report = SeoAnalyzer::analyze(&snapshot);

    let skipped = report
        .issues
        .iter()
        .any(|i| i.message.contains("Heading level skipped"));
    assert!(skipped, "Should detect skipped heading level");
}

#[test]
fn analyze_structured_data_extraction() {
    let html = r#"
        <html><head>
            <meta charset="utf-8"><title>Blog Post About Rust</title>
            <script type="application/ld+json">{"@type": "Article", "headline": "Rust is great"}</script>
            <script type="application/ld+json">{"@type": "BreadcrumbList"}</script>
        </head>
        <body><h1>Blog Post</h1></body></html>
    "#;

    let snapshot = snapshot_with_html(html);
    let report = SeoAnalyzer::analyze(&snapshot);

    assert_eq!(report.structured_data.len(), 2);
    assert!(report
        .structured_data
        .iter()
        .any(|s| s.schema_type == "Article"));
    assert!(report
        .structured_data
        .iter()
        .any(|s| s.schema_type == "BreadcrumbList"));
}

#[test]
fn analyze_title_length_boundaries() {
    let short = snapshot_with_html(
        r#"<html><head><meta charset="utf-8"><title>Hi</title></head><body><h1>X</h1></body></html>"#,
    );
    let report = SeoAnalyzer::analyze(&short);
    assert!(report
        .issues
        .iter()
        .any(|i| i.message.contains("too short")));

    let long_title = "A".repeat(65);
    let long_html = format!(
        r#"<html><head><meta charset="utf-8"><title>{}</title></head><body><h1>X</h1></body></html>"#,
        long_title
    );
    let long = snapshot_with_html(&long_html);
    let report = SeoAnalyzer::analyze(&long);
    assert!(report
        .issues
        .iter()
        .any(|i| i.message.contains("truncated")));
}

#[test]
fn score_clamped_to_bounds() {
    let snapshot = snapshot_with_html("");
    let report = SeoAnalyzer::analyze(&snapshot);
    assert!(report.score >= 0.0);
    assert!(report.score <= 100.0);
}

#[test]
fn analyze_with_custom_score_config_changes_score() {
    let snapshot = snapshot_with_html(
        r#"<html><head><meta charset="utf-8"><title>Custom Score Page</title></head><body><h1>Test</h1></body></html>"#,
    );
    let default_report = SeoAnalyzer::analyze(&snapshot);

    let custom_config = ScoreConfig {
        title_bonus: 20.0,
        ..Default::default()
    };
    let custom_report = SeoAnalyzer::analyze_with_config(&snapshot, &custom_config);

    assert!(
        custom_report.score > default_report.score,
        "Custom config with higher title bonus should produce higher score"
    );
}
