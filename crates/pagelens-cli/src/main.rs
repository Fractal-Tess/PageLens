mod error;
mod prelude;

use clap::{ArgAction, Parser, ValueEnum};
use pagelens_core::browser::Browser;
use pagelens_core::seo::SeoAnalyzer;
use pagelens_core::snapshot::{SnapshotExt, SnapshotOptions};
use reqwest::header::{CONTENT_LENGTH, CONTENT_RANGE, HeaderMap, HeaderName, HeaderValue, RANGE};
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "pagelens", about = "Audit web pages for contrast, SEO, accessibility, and performance")]
#[command(version)]
struct Cli {
    /// URL to analyze
    url: String,
    
    /// Output results as JSON
    #[arg(long)]
    json: bool,
    
    /// Run only SEO checks
    #[arg(long)]
    seo: bool,

    #[arg(long, value_enum, default_value_t = AnalysisMode::Full)]
    mode: AnalysisMode,

    #[arg(long, help = "Include referenced JS/CSS/media assets in output")]
    assets: bool,

    #[arg(long, value_enum, default_value_t = AssetSort::SizeDesc)]
    asset_sort: AssetSort,

    #[arg(long, value_enum, default_value_t = AssetGroup::Type)]
    asset_group: AssetGroup,

    #[arg(long = "header", value_name = "KEY:VALUE", action = ArgAction::Append)]
    headers: Vec<String>,

    #[arg(long, help = "Include page paint and timing metrics")]
    perf: bool,

    #[arg(long, help = "Run basic text/background contrast audit")]
    contrast: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum, Serialize)]
enum AssetSort {
    UrlAsc,
    UrlDesc,
    SizeAsc,
    SizeDesc,
}

#[derive(Debug, Clone, Copy, ValueEnum, Serialize)]
enum AssetGroup {
    Type,
    Host,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize)]
enum AnalysisMode {
    Full,
    Custom,
    SeoOnly,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let request_headers = match parse_headers(&cli.headers) {
        Ok(headers) => headers,
        Err(err) => {
            eprintln!("Error: {err}");
            std::process::exit(1);
        }
    };

    // Validate URL
    if !cli.url.starts_with("http://") && !cli.url.starts_with("https://") {
        eprintln!("Error: URL must start with http:// or https://");
        std::process::exit(1);
    }

    if let Err(e) = run_audit(
        &cli.url,
        cli.json,
        cli.seo,
        cli.mode,
        cli.assets,
        cli.asset_sort,
        cli.asset_group,
        &request_headers,
        cli.perf,
        cli.contrast,
    )
    .await
    {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

async fn run_audit(
    url: &str,
    json_output: bool,
    seo_only_flag: bool,
    mode: AnalysisMode,
    include_assets: bool,
    asset_sort: AssetSort,
    asset_group: AssetGroup,
    request_headers: &HashMap<String, String>,
    include_perf: bool,
    include_contrast: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let effective_mode = if seo_only_flag {
        AnalysisMode::SeoOnly
    } else {
        mode
    };
    let (effective_assets, effective_perf, effective_contrast) = match effective_mode {
        AnalysisMode::Full => (true, true, true),
        AnalysisMode::Custom => (include_assets, include_perf, include_contrast),
        AnalysisMode::SeoOnly => (false, false, false),
    };

    if !json_output {
        println!("🔍 PageLens Audit");
        println!("   URL: {}", url);
        println!();
    }

    // Launch browser
    if !json_output {
        println!("⏳ Launching browser...");
    }
    
    let browser = Browser::launch().await
        .map_err(|e| format!("Failed to launch browser: {e}"))?;

    // Navigate to URL
    if !json_output {
        println!("⏳ Navigating to page...");
    }
    
    let page = if request_headers.is_empty() {
        browser
            .navigate(url)
            .await
            .map_err(|e| format!("Failed to navigate: {e}"))?
    } else {
        browser
            .navigate_with_headers(url, request_headers)
            .await
            .map_err(|e| format!("Failed to navigate with headers: {e}"))?
    };

    // Capture snapshot
    if !json_output {
        println!("⏳ Capturing page snapshot...");
    }
    
    let mut snapshot_options = SnapshotOptions::default();
    snapshot_options.include_computed_styles = effective_contrast;

    let snapshot = page.snapshot(snapshot_options).await
        .map_err(|e| format!("Failed to capture snapshot: {e}"))?;

    // Run SEO analysis
    let seo_report = SeoAnalyzer::analyze(&snapshot);

    let asset_sizes = if effective_assets {
        Some(analyze_asset_sizes(&snapshot.referenced_assets, request_headers).await)
    } else {
        None
    };

    let performance_report = if effective_perf {
        Some(build_performance_report(&snapshot.performance_timing))
    } else {
        None
    };

    let contrast_report = if effective_contrast {
        Some(analyze_contrast(&snapshot.computed_styles))
    } else {
        None
    };

    // Output results
    if json_output {
        // JSON output
        let output = serde_json::json!({
            "url": url,
            "title": snapshot.title,
            "seo": {
                "score": seo_report.score,
                "meta": seo_report.meta,
                "open_graph": seo_report.open_graph,
                "twitter_card": seo_report.twitter_card,
                "canonical_url": seo_report.canonical_url,
                "headings": seo_report.headings,
                "images": seo_report.images,
                "structured_data": seo_report.structured_data,
                "issues": seo_report.issues,
            },
            "asset_sizes": asset_sizes,
            "asset_sort": if effective_assets { Some(asset_sort) } else { None },
            "asset_group": if effective_assets { Some(asset_group) } else { None },
            "analysis_mode": effective_mode,
            "performance": performance_report,
            "contrast": contrast_report,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        print_results(&seo_report);
        if let Some(report) = &performance_report {
            print_performance_report(report);
        }
        if let Some(report) = &contrast_report {
            print_contrast_report(report);
        }
        if effective_assets {
            if let Some(sizes) = &asset_sizes {
                print_assets_report(sizes, asset_sort, asset_group);
            }
        }
    }

    // Shutdown browser
    browser.shutdown().await.ok();

    Ok(())
}

#[derive(Debug, Clone, Serialize)]
struct AssetSizeEntry {
    url: String,
    asset_type: AssetType,
    bytes: Option<u64>,
    source: &'static str,
    error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum AssetType {
    JavaScript,
    Stylesheet,
    Media,
}

impl AssetType {
    fn label(self) -> &'static str {
        match self {
            AssetType::JavaScript => "JavaScript",
            AssetType::Stylesheet => "Stylesheets",
            AssetType::Media => "Media",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct AssetSizeSummary {
    entries: Vec<AssetSizeEntry>,
    total_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
struct PerformanceReport {
    navigation_start_ms: u64,
    ttfb_ms: Option<u64>,
    dom_interactive_ms: Option<u64>,
    dom_content_loaded_ms: Option<u64>,
    load_complete_ms: Option<u64>,
    first_paint_ms: Option<u64>,
    first_contentful_paint_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
struct ContrastIssue {
    selector: String,
    ratio: f64,
    foreground: String,
    background: String,
}

#[derive(Debug, Clone, Serialize)]
struct ContrastReport {
    analyzed_elements: usize,
    failing_elements: usize,
    min_ratio: Option<f64>,
    issues: Vec<ContrastIssue>,
}

fn build_performance_report(timing: &pagelens_core::snapshot::PerformanceTiming) -> PerformanceReport {
    PerformanceReport {
        navigation_start_ms: timing.navigation_start,
        ttfb_ms: delta_from_nav(timing.navigation_start, timing.response_start),
        dom_interactive_ms: delta_from_nav(timing.navigation_start, timing.dom_interactive),
        dom_content_loaded_ms: delta_from_nav(timing.navigation_start, timing.dom_content_loaded),
        load_complete_ms: delta_from_nav(timing.navigation_start, timing.load_complete),
        first_paint_ms: delta_from_nav(timing.navigation_start, timing.first_paint),
        first_contentful_paint_ms: delta_from_nav(timing.navigation_start, timing.first_contentful_paint),
    }
}

fn delta_from_nav(navigation_start: u64, absolute: Option<u64>) -> Option<u64> {
    absolute.and_then(|value| value.checked_sub(navigation_start))
}

fn analyze_contrast(styles: &[pagelens_core::snapshot::ComputedStyle]) -> ContrastReport {
    let mut analyzed = 0usize;
    let mut failing = 0usize;
    let mut min_ratio: Option<f64> = None;
    let mut issues = Vec::new();

    for style in styles {
        let (Some(fg), Some(bg)) = (style.color.as_ref(), style.background_color.as_ref()) else {
            continue;
        };

        let (Some(fg_luminance), Some(bg_luminance)) = (color_luminance(fg), color_luminance(bg)) else {
            continue;
        };

        analyzed += 1;
        let ratio = contrast_ratio_from_luminance(fg_luminance, bg_luminance);
        min_ratio = Some(min_ratio.map_or(ratio, |m| m.min(ratio)));

        if ratio < 4.5 {
            failing += 1;
            issues.push(ContrastIssue {
                selector: style.selector.clone(),
                ratio,
                foreground: fg.clone(),
                background: bg.clone(),
            });
        }
    }

    issues.sort_by(|a, b| a.ratio.partial_cmp(&b.ratio).unwrap_or(std::cmp::Ordering::Equal));
    issues.truncate(20);

    ContrastReport {
        analyzed_elements: analyzed,
        failing_elements: failing,
        min_ratio,
        issues,
    }
}

fn parse_rgb_triplet(value: &str) -> Option<(u8, u8, u8)> {
    let trimmed = value.trim();
    if let Some(hex) = trimmed.strip_prefix('#') {
        return parse_hex_color(hex);
    }
    if trimmed.eq_ignore_ascii_case("transparent") {
        return None;
    }
    let body = if trimmed.starts_with("rgb(") {
        trimmed.strip_prefix("rgb(")?.strip_suffix(')')?
    } else if trimmed.starts_with("rgba(") {
        trimmed.strip_prefix("rgba(")?.strip_suffix(')')?
    } else {
        return None;
    };

    let parts = body
        .split(',')
        .map(|p| p.trim())
        .collect::<Vec<_>>();
    if parts.len() < 3 {
        return None;
    }

    let r = parse_rgb_component(parts[0])?;
    let g = parse_rgb_component(parts[1])?;
    let b = parse_rgb_component(parts[2])?;
    Some((r, g, b))
}

fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some((r, g, b))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((r, g, b))
        }
        8 => {
            let alpha = u8::from_str_radix(&hex[6..8], 16).ok()?;
            if alpha == 0 {
                return None;
            }
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((r, g, b))
        }
        _ => None,
    }
}

fn parse_rgb_component(part: &str) -> Option<u8> {
    if let Some(percent) = part.strip_suffix('%') {
        let p = percent.trim().parse::<f64>().ok()?;
        let clamped = p.clamp(0.0, 100.0);
        return Some(((clamped / 100.0) * 255.0).round() as u8);
    }
    let n = part.trim().parse::<f64>().ok()?;
    Some(n.clamp(0.0, 255.0).round() as u8)
}

fn contrast_ratio_from_luminance(la: f64, lb: f64) -> f64 {
    let (light, dark) = if la >= lb { (la, lb) } else { (lb, la) };
    (light + 0.05) / (dark + 0.05)
}

fn color_luminance(value: &str) -> Option<f64> {
    if let Some(rgb) = parse_rgb_triplet(value) {
        return Some(relative_luminance(rgb));
    }
    parse_lab_luminance(value)
        .or_else(|| parse_oklab_luminance(value))
        .or_else(|| parse_oklch_luminance(value))
}

fn parse_lab_luminance(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    if !trimmed.starts_with("lab(") {
        return None;
    }
    let body = trimmed.strip_prefix("lab(")?.strip_suffix(')')?;
    let mut parts = body.split('/');
    let left = parts.next()?.trim();
    if let Some(alpha_raw) = parts.next() {
        let alpha = parse_alpha(alpha_raw.trim())?;
        if alpha <= 0.0 {
            return None;
        }
    }

    let lightness_raw = left.split_whitespace().next()?;
    let lightness = if let Some(percent) = lightness_raw.strip_suffix('%') {
        percent.trim().parse::<f64>().ok()?
    } else {
        lightness_raw.parse::<f64>().ok()?
    };
    let l = lightness.clamp(0.0, 100.0);

    let y = if l > 8.0 {
        ((l + 16.0) / 116.0).powf(3.0)
    } else {
        l / 903.3
    };
    Some(y.clamp(0.0, 1.0))
}

fn parse_oklab_luminance(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    if !trimmed.starts_with("oklab(") {
        return None;
    }
    let body = trimmed.strip_prefix("oklab(")?.strip_suffix(')')?;
    let mut parts = body.split('/');
    let left = parts.next()?.trim();
    if let Some(alpha_raw) = parts.next() {
        let alpha = parse_alpha(alpha_raw.trim())?;
        if alpha <= 0.0 {
            return None;
        }
    }

    let components = left.split_whitespace().collect::<Vec<_>>();
    if components.len() < 3 {
        return None;
    }
    let l = parse_percent_or_number(components[0], 1.0)?;
    let a = parse_percent_or_number(components[1], 0.4)?;
    let b = parse_percent_or_number(components[2], 0.4)?;
    oklab_luminance(l, a, b)
}

fn parse_oklch_luminance(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    if !trimmed.starts_with("oklch(") {
        return None;
    }
    let body = trimmed.strip_prefix("oklch(")?.strip_suffix(')')?;
    let mut parts = body.split('/');
    let left = parts.next()?.trim();
    if let Some(alpha_raw) = parts.next() {
        let alpha = parse_alpha(alpha_raw.trim())?;
        if alpha <= 0.0 {
            return None;
        }
    }

    let components = left.split_whitespace().collect::<Vec<_>>();
    if components.len() < 3 {
        return None;
    }
    let l = parse_percent_or_number(components[0], 1.0)?;
    let c = parse_percent_or_number(components[1], 0.4)?;
    let h = components[2]
        .trim()
        .trim_end_matches("deg")
        .parse::<f64>()
        .ok()?;
    let radians = h.to_radians();
    let a = c * radians.cos();
    let b = c * radians.sin();
    oklab_luminance(l, a, b)
}

fn parse_percent_or_number(raw: &str, percent_scale: f64) -> Option<f64> {
    let trimmed = raw.trim();
    if let Some(percent) = trimmed.strip_suffix('%') {
        let p = percent.trim().parse::<f64>().ok()?;
        return Some((p / 100.0) * percent_scale);
    }
    trimmed.parse::<f64>().ok()
}

fn oklab_luminance(l: f64, a: f64, b: f64) -> Option<f64> {
    let l_ = l + (0.3963377774 * a) + (0.2158037573 * b);
    let m_ = l - (0.1055613458 * a) - (0.0638541728 * b);
    let s_ = l - (0.0894841775 * a) - (1.2914855480 * b);

    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;

    let r = (4.0767416621 * l3) - (3.3077115913 * m3) + (0.2309699292 * s3);
    let g = (-1.2684380046 * l3) + (2.6097574011 * m3) - (0.3413193965 * s3);
    let b = (-0.0041960863 * l3) - (0.7034186147 * m3) + (1.7076147010 * s3);

    if !r.is_finite() || !g.is_finite() || !b.is_finite() {
        return None;
    }

    let y = (0.2126 * r) + (0.7152 * g) + (0.0722 * b);
    Some(y.clamp(0.0, 1.0))
}

fn parse_alpha(alpha_raw: &str) -> Option<f64> {
    let trimmed = alpha_raw.trim();
    if let Some(percent) = trimmed.strip_suffix('%') {
        let p = percent.trim().parse::<f64>().ok()?;
        return Some((p / 100.0).clamp(0.0, 1.0));
    }
    let value = trimmed.parse::<f64>().ok()?;
    Some(value.clamp(0.0, 1.0))
}

fn relative_luminance((r, g, b): (u8, u8, u8)) -> f64 {
    fn channel(v: u8) -> f64 {
        let s = (v as f64) / 255.0;
        if s <= 0.03928 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }
    (0.2126 * channel(r)) + (0.7152 * channel(g)) + (0.0722 * channel(b))
}

async fn analyze_asset_sizes(
    assets: &pagelens_core::ReferencedAssets,
    request_headers: &HashMap<String, String>,
) -> AssetSizeSummary {
    let default_headers = match to_header_map(request_headers) {
        Ok(headers) => headers,
        Err(err) => {
            let mut entries = Vec::new();
            let fallback = |urls: &[String], asset_type: AssetType| {
                urls.iter()
                    .map(|url| AssetSizeEntry {
                        url: url.clone(),
                        asset_type,
                        bytes: None,
                        source: "none",
                        error: Some(err.clone()),
                    })
                    .collect::<Vec<_>>()
            };
            entries.extend(fallback(&assets.javascript, AssetType::JavaScript));
            entries.extend(fallback(&assets.stylesheets, AssetType::Stylesheet));
            entries.extend(fallback(&assets.media, AssetType::Media));
            return AssetSizeSummary {
                entries,
                total_bytes: 0,
            };
        }
    };

    let client = reqwest::Client::builder()
        .default_headers(default_headers)
        .timeout(Duration::from_secs(12))
        .redirect(reqwest::redirect::Policy::limited(10))
        .build();

    if let Ok(client) = client {
        let mut entries = Vec::new();
        entries.extend(analyze_category_sizes(&client, &assets.javascript, AssetType::JavaScript).await);
        entries.extend(analyze_category_sizes(&client, &assets.stylesheets, AssetType::Stylesheet).await);
        entries.extend(analyze_category_sizes(&client, &assets.media, AssetType::Media).await);
        let total_bytes = sum_known_bytes(&entries);

        AssetSizeSummary { entries, total_bytes }
    } else {
        let fallback = |urls: &[String], asset_type: AssetType| {
            urls.iter()
                .map(|url| AssetSizeEntry {
                    url: url.clone(),
                    asset_type,
                    bytes: None,
                    source: "none",
                    error: Some("Failed to build HTTP client".to_string()),
                })
                .collect::<Vec<_>>()
        };
        let mut entries = Vec::new();
        entries.extend(fallback(&assets.javascript, AssetType::JavaScript));
        entries.extend(fallback(&assets.stylesheets, AssetType::Stylesheet));
        entries.extend(fallback(&assets.media, AssetType::Media));
        AssetSizeSummary { entries, total_bytes: 0 }
    }
}

async fn analyze_category_sizes(
    client: &reqwest::Client,
    urls: &[String],
    asset_type: AssetType,
) -> Vec<AssetSizeEntry> {
    let mut out = Vec::with_capacity(urls.len());
    for url in urls {
        out.push(fetch_asset_size(client, url, asset_type).await);
    }
    out
}

async fn fetch_asset_size(client: &reqwest::Client, url: &str, asset_type: AssetType) -> AssetSizeEntry {
    let head_res = client.head(url).send().await;
    if let Ok(resp) = head_res {
        if resp.status().is_success() {
            if let Some(bytes) = extract_content_length(&resp) {
                return AssetSizeEntry {
                    url: url.to_string(),
                    asset_type,
                    bytes: Some(bytes),
                    source: "head",
                    error: None,
                };
            }
        } else {
            return AssetSizeEntry {
                url: url.to_string(),
                asset_type,
                bytes: None,
                source: "head",
                error: Some(format!("HEAD returned {}", resp.status())),
            };
        }
    }

    let range_res = client.get(url).header(RANGE, "bytes=0-0").send().await;
    match range_res {
        Ok(resp) => {
            if !resp.status().is_success() && resp.status() != reqwest::StatusCode::PARTIAL_CONTENT {
                return AssetSizeEntry {
                    url: url.to_string(),
                    asset_type,
                    bytes: None,
                    source: "range",
                    error: Some(format!("Range GET returned {}", resp.status())),
                };
            }

            if let Some(bytes) = extract_content_range_total(&resp) {
                return AssetSizeEntry {
                    url: url.to_string(),
                    asset_type,
                    bytes: Some(bytes),
                    source: "range",
                    error: None,
                };
            }

            if let Some(bytes) = extract_content_length(&resp) {
                return AssetSizeEntry {
                    url: url.to_string(),
                    asset_type,
                    bytes: Some(bytes),
                    source: "range-content-length",
                    error: None,
                };
            }

            AssetSizeEntry {
                url: url.to_string(),
                asset_type,
                bytes: None,
                source: "range",
                error: Some("Size header not available".to_string()),
            }
        }
        Err(err) => AssetSizeEntry {
            url: url.to_string(),
            asset_type,
            bytes: None,
            source: "range",
            error: Some(err.to_string()),
        },
    }
}

fn extract_content_length(resp: &reqwest::Response) -> Option<u64> {
    let value = resp.headers().get(CONTENT_LENGTH)?;
    value.to_str().ok()?.parse::<u64>().ok()
}

fn extract_content_range_total(resp: &reqwest::Response) -> Option<u64> {
    let value = resp.headers().get(CONTENT_RANGE)?;
    let raw = value.to_str().ok()?;
    parse_content_range_total(raw)
}

fn parse_content_range_total(raw: &str) -> Option<u64> {
    let total = raw.split('/').nth(1)?;
    if total == "*" {
        return None;
    }
    total.parse::<u64>().ok()
}

fn sum_known_bytes(entries: &[AssetSizeEntry]) -> u64 {
    entries.iter().filter_map(|e| e.bytes).sum()
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        return format!("{:.2} MB", bytes as f64 / 1024.0 / 1024.0);
    }
    if bytes >= 1024 {
        return format!("{:.2} KB", bytes as f64 / 1024.0);
    }
    format!("{} B", bytes)
}

fn print_performance_report(report: &PerformanceReport) {
    println!("═══════════════════════════════════════════════════════════");
    println!("                 PERFORMANCE METRICS                       ");
    println!("═══════════════════════════════════════════════════════════");
    print_metric("TTFB", report.ttfb_ms);
    print_metric("First Paint", report.first_paint_ms);
    print_metric("First Contentful Paint", report.first_contentful_paint_ms);
    print_metric("DOM Interactive", report.dom_interactive_ms);
    print_metric("DOM Content Loaded", report.dom_content_loaded_ms);
    print_metric("Load Complete", report.load_complete_ms);
    println!();
}

fn print_metric(label: &str, value_ms: Option<u64>) {
    match value_ms {
        Some(value) => println!("  {}: {} ms", label, value),
        None => println!("  {}: n/a", label),
    }
}

fn print_contrast_report(report: &ContrastReport) {
    println!("═══════════════════════════════════════════════════════════");
    println!("                   CONTRAST AUDIT                          ");
    println!("═══════════════════════════════════════════════════════════");
    println!("  Elements analyzed: {}", report.analyzed_elements);
    println!("  Failing elements (< 4.5): {}", report.failing_elements);
    if let Some(min_ratio) = report.min_ratio {
        println!("  Lowest ratio: {:.2}:1", min_ratio);
    } else {
        println!("  Lowest ratio: n/a");
    }
    if !report.issues.is_empty() {
        println!("  Worst offenders:");
        for issue in &report.issues {
            println!(
                "    - {} [ratio {:.2}:1 | fg {} | bg {}]",
                issue.selector,
                issue.ratio,
                issue.foreground,
                issue.background
            );
        }
    }
    println!();
}

fn print_assets_report(summary: &AssetSizeSummary, sort: AssetSort, group: AssetGroup) {
    let mut entries = summary.entries.clone();
    sort_asset_entries(&mut entries, sort);

    println!("═══════════════════════════════════════════════════════════");
    println!("                      ASSETS                                ");
    println!("═══════════════════════════════════════════════════════════");
    println!("  Sort: {}", asset_sort_label(sort));
    println!("  Group: {}", asset_group_label(group));
    println!("  Total Known Size: {}", format_bytes(summary.total_bytes));
    println!();

    match group {
        AssetGroup::Type => {
            print_asset_size_group("JavaScript", AssetType::JavaScript, &entries);
            print_asset_size_group("Stylesheets", AssetType::Stylesheet, &entries);
            print_asset_size_group("Media", AssetType::Media, &entries);
        }
        AssetGroup::Host => {
            let mut by_host: BTreeMap<String, Vec<AssetSizeEntry>> = BTreeMap::new();
            for entry in entries {
                let host = reqwest::Url::parse(&entry.url)
                    .ok()
                    .and_then(|u| u.host_str().map(|h| h.to_string()))
                    .unwrap_or_else(|| "unknown-host".to_string());
                by_host.entry(host).or_default().push(entry);
            }
            for (host, grouped) in by_host {
                print_group_entries(&host, &grouped);
            }
        }
        AssetGroup::None => {
            print_group_entries("All assets", &entries);
        }
    }
}

fn print_asset_size_group(label: &str, asset_type: AssetType, entries: &[AssetSizeEntry]) {
    let typed = entries
        .iter()
        .filter(|entry| entry.asset_type == asset_type)
        .cloned()
        .collect::<Vec<_>>();
    print_group_entries(label, &typed);
}

fn print_group_entries(label: &str, entries: &[AssetSizeEntry]) {
    let known = entries.iter().filter_map(|e| e.bytes).sum::<u64>();
    println!("  {}: {} ({} files)", label, format_bytes(known), entries.len());
    for entry in entries {
        let type_label = entry.asset_type.label();
        if let Some(bytes) = entry.bytes {
            println!(
                "    - {} [{} | {} via {}]",
                entry.url,
                type_label,
                format_bytes(bytes),
                entry.source
            );
        } else if let Some(err) = &entry.error {
            println!("    - {} [{} | unknown: {}]", entry.url, type_label, err);
        } else {
            println!("    - {} [{} | unknown]", entry.url, type_label);
        }
    }
    println!();
}

fn sort_asset_entries(entries: &mut [AssetSizeEntry], sort: AssetSort) {
    match sort {
        AssetSort::UrlAsc => entries.sort_by(|a, b| a.url.cmp(&b.url)),
        AssetSort::UrlDesc => entries.sort_by(|a, b| b.url.cmp(&a.url)),
        AssetSort::SizeAsc => entries.sort_by(|a, b| compare_size_then_url(a, b, true)),
        AssetSort::SizeDesc => entries.sort_by(|a, b| compare_size_then_url(a, b, false)),
    }
}

fn compare_size_then_url(a: &AssetSizeEntry, b: &AssetSizeEntry, asc: bool) -> std::cmp::Ordering {
    let left = a.bytes.unwrap_or(0);
    let right = b.bytes.unwrap_or(0);
    if asc {
        left.cmp(&right).then_with(|| a.url.cmp(&b.url))
    } else {
        right.cmp(&left).then_with(|| a.url.cmp(&b.url))
    }
}

fn asset_sort_label(sort: AssetSort) -> &'static str {
    match sort {
        AssetSort::UrlAsc => "url-asc",
        AssetSort::UrlDesc => "url-desc",
        AssetSort::SizeAsc => "size-asc",
        AssetSort::SizeDesc => "size-desc",
    }
}

fn asset_group_label(group: AssetGroup) -> &'static str {
    match group {
        AssetGroup::Type => "type",
        AssetGroup::Host => "host",
        AssetGroup::None => "none",
    }
}

fn parse_headers(raw_headers: &[String]) -> Result<HashMap<String, String>, String> {
    let mut headers = HashMap::new();
    for raw in raw_headers {
        let (name_raw, value_raw) = raw
            .split_once(':')
            .ok_or_else(|| format!("Invalid header '{raw}'. Expected KEY:VALUE"))?;
        let name = name_raw.trim();
        let value = value_raw.trim();
        if name.is_empty() {
            return Err(format!("Invalid header '{raw}'. Header name is empty"));
        }
        validate_header(name, value).map_err(|e| format!("Invalid header '{raw}': {e}"))?;
        headers.insert(name.to_string(), value.to_string());
    }
    Ok(headers)
}

fn validate_header(name: &str, value: &str) -> Result<(), String> {
    HeaderName::from_bytes(name.as_bytes())
        .map_err(|e| format!("invalid header name ({e})"))?;
    HeaderValue::from_str(value).map_err(|e| format!("invalid header value ({e})"))?;
    Ok(())
}

fn to_header_map(headers: &HashMap<String, String>) -> Result<HeaderMap, String> {
    let mut map = HeaderMap::new();
    for (name, value) in headers {
        let name = HeaderName::from_bytes(name.as_bytes())
            .map_err(|e| format!("invalid header name '{name}': {e}"))?;
        let value = HeaderValue::from_str(value)
            .map_err(|e| format!("invalid header value for '{name}': {e}"))?;
        map.insert(name, value);
    }
    Ok(map)
}

fn print_results(report: &pagelens_core::seo::SeoReport) {
    use pagelens_core::seo::Severity;

    println!();
    println!("═══════════════════════════════════════════════════════════");
    println!("                      SEO REPORT                           ");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    // Score
    let score_color = if report.score >= 80.0 {
        "\x1b[32m" // Green
    } else if report.score >= 60.0 {
        "\x1b[33m" // Yellow
    } else {
        "\x1b[31m" // Red
    };
    let reset = "\x1b[0m";
    
    println!("  Overall Score: {}{:.0}/100{}\n", score_color, report.score, reset);

    // Meta Tags
    println!("  📋 Meta Tags");
    println!("  ─────────────────────────────────────────────────────────");
    
    if let Some(title) = &report.meta.title {
        let title_status = if title.len() <= 60 { "✅" } else { "⚠️ " };
        println!("    {} Title: {} ({} chars)", title_status, title, title.len());
    } else {
        println!("    ❌ Title: Missing");
    }
    
    if let Some(desc) = &report.meta.description {
        let desc_status = if desc.len() >= 50 && desc.len() <= 160 { "✅" } else { "⚠️ " };
        println!("    {} Description: {} ({} chars)", desc_status, desc, desc.len());
    } else {
        println!("    ❌ Description: Missing");
    }
    
    println!("    {} Charset", if report.meta.charset { "✅" } else { "❌" });
    println!("    {} Viewport", if report.meta.viewport { "✅" } else { "❌" });
    
    if let Some(lang) = &report.meta.language {
        println!("    ✅ Language: {}", lang);
    } else {
        println!("    ⚠️  Language: Not specified");
    }
    println!();

    // Open Graph
    println!("  🔗 Open Graph Tags");
    println!("  ─────────────────────────────────────────────────────────");
    println!("    {} og:title", if report.open_graph.title.is_some() { "✅" } else { "❌" });
    println!("    {} og:description", if report.open_graph.description.is_some() { "✅" } else { "❌" });
    println!("    {} og:type", if report.open_graph.og_type.is_some() { "✅" } else { "❌" });
    println!("    {} og:url", if report.open_graph.url.is_some() { "✅" } else { "❌" });
    println!("    {} og:image", if report.open_graph.image.is_some() { "✅" } else { "❌" });
    println!();

    // Twitter Cards
    println!("  🐦 Twitter Cards");
    println!("  ─────────────────────────────────────────────────────────");
    println!("    {} twitter:card", if report.twitter_card.card.is_some() { "✅" } else { "❌" });
    println!("    {} twitter:title", if report.twitter_card.title.is_some() { "✅" } else { "❌" });
    println!("    {} twitter:description", if report.twitter_card.description.is_some() { "✅" } else { "❌" });
    println!();

    // Canonical
    println!("  📎 Canonical URL");
    println!("  ─────────────────────────────────────────────────────────");
    if let Some(canonical) = &report.canonical_url {
        println!("    ✅ {}", canonical);
    } else {
        println!("    ❌ Not specified");
    }
    println!();

    // Headings
    println!("  📝 Headings Structure");
    println!("  ─────────────────────────────────────────────────────────");
    println!("    H1: {}  H2: {}  H3: {}  H4: {}  H5: {}  H6: {}", 
        report.headings.h1_count,
        report.headings.h2_count,
        report.headings.h3_count,
        report.headings.h4_count,
        report.headings.h5_count,
        report.headings.h6_count
    );
    if report.headings.h1_count > 1 {
        println!("    ⚠️  Multiple H1 tags detected");
    }
    println!();

    // Images
    println!("  🖼️  Images ({})", report.images.len());
    println!("  ─────────────────────────────────────────────────────────");
    let missing_alt = report.images.iter().filter(|i| !i.has_alt).count();
    if missing_alt > 0 {
        println!("    ⚠️  {} images missing alt text", missing_alt);
    } else {
        println!("    ✅ All images have alt text");
    }
    println!();

    // Issues
    if !report.issues.is_empty() {
        println!("  ⚠️  Issues Found ({})", report.issues.len());
        println!("  ─────────────────────────────────────────────────────────");
        
        for issue in &report.issues {
            let icon = match issue.severity {
                Severity::Error => "❌",
                Severity::Warning => "⚠️ ",
                Severity::Info => "ℹ️ ",
            };
            println!("    {} [{}] {}", icon, issue.category, issue.message);
        }
        println!();
    }

    println!("═══════════════════════════════════════════════════════════");
}

#[cfg(test)]
mod tests {
    use super::{
        analyze_contrast, build_performance_report, parse_content_range_total, parse_headers,
        parse_rgb_triplet, AnalysisMode, AssetSizeEntry, AssetSort, AssetType,
    };
    use clap::ValueEnum;

    #[test]
    fn parses_content_range_total() {
        assert_eq!(parse_content_range_total("bytes 0-0/12345"), Some(12345));
        assert_eq!(parse_content_range_total("bytes 0-1023/2048"), Some(2048));
        assert_eq!(parse_content_range_total("bytes */*"), None);
        assert_eq!(parse_content_range_total("garbage"), None);
    }

    #[test]
    fn sort_by_size_desc_orders_largest_first() {
        let mut entries = vec![
            AssetSizeEntry {
                url: "https://a.example/small.js".to_string(),
                asset_type: AssetType::JavaScript,
                bytes: Some(10),
                source: "head",
                error: None,
            },
            AssetSizeEntry {
                url: "https://a.example/unknown.js".to_string(),
                asset_type: AssetType::JavaScript,
                bytes: None,
                source: "head",
                error: Some("missing".to_string()),
            },
            AssetSizeEntry {
                url: "https://a.example/large.js".to_string(),
                asset_type: AssetType::JavaScript,
                bytes: Some(100),
                source: "head",
                error: None,
            },
        ];

        super::sort_asset_entries(&mut entries, AssetSort::SizeDesc);

        assert_eq!(entries[0].url, "https://a.example/large.js");
        assert_eq!(entries[1].url, "https://a.example/small.js");
        assert_eq!(entries[2].url, "https://a.example/unknown.js");
    }

    #[test]
    fn parse_headers_accepts_multiple_entries() {
        let parsed = parse_headers(&[
            "Authorization: Bearer token:abc".to_string(),
            "X-Org: pagelens".to_string(),
        ])
        .expect("headers should parse");

        assert_eq!(parsed.get("Authorization"), Some(&"Bearer token:abc".to_string()));
        assert_eq!(parsed.get("X-Org"), Some(&"pagelens".to_string()));
    }

    #[test]
    fn parse_headers_rejects_invalid_format() {
        let err = parse_headers(&["InvalidHeaderWithoutColon".to_string()])
            .expect_err("should fail for missing colon");
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
    fn parse_rgb_triplet_supports_hex() {
        assert_eq!(parse_rgb_triplet("#ffffff"), Some((255, 255, 255)));
        assert_eq!(parse_rgb_triplet("#123"), Some((17, 34, 51)));
        assert_eq!(parse_rgb_triplet("#00000000"), None);
    }

    #[test]
    fn contrast_report_handles_oklab_colors() {
        let styles = vec![pagelens_core::snapshot::ComputedStyle {
            selector: "div.oklab".to_string(),
            color: Some("oklab(0.2 0 0)".to_string()),
            background_color: Some("oklab(0.95 0 0)".to_string()),
            font_size: None,
            font_weight: None,
        }];

        let report = analyze_contrast(&styles);
        assert_eq!(report.analyzed_elements, 1);
    }

    #[test]
    fn analysis_mode_defaults_include_full() {
        let variants = AnalysisMode::value_variants();
        assert!(variants.contains(&AnalysisMode::Full));
    }
}
