//! CLI module for pagelens-cli

use clap::{ArgAction, Parser};
use std::collections::HashMap;

mod analysis;
mod assets;
mod colors;
mod output;
mod server;

pub use analysis::{
    analyze_contrast, build_performance_report, parse_headers, resolve_effective_toggles,
    AnalysisMode, AssetGroup, AssetSizeEntry, AssetSizeSummary, AssetSort, AssetType,
    ContrastIssue, ContrastReport, EffectiveToggles, PerformanceReport,
};
pub use assets::{
    analyze_asset_sizes, analyze_category_sizes, compare_size_then_url, extract_content_length,
    extract_content_range_total, fetch_asset_size, format_bytes, parse_content_range_total,
    sort_asset_entries, sum_known_bytes,
};
pub use colors::{
    color_luminance, contrast_ratio_from_luminance, oklab_luminance, parse_alpha,
    parse_hex_color, parse_lab_luminance, parse_oklab_luminance, parse_oklch_luminance,
    parse_percent_or_number, parse_rgb_component, parse_rgb_triplet, relative_luminance,
};
pub use output::{
    asset_group_label, asset_sort_label, print_assets_report, print_contrast_report,
    print_group_entries, print_metric, print_performance_report, print_results,
    print_site_files_report,
};
pub use server::{default_server_db_path, open_browser_for_server, run_server};

#[derive(Parser)]
#[command(
    name = "pagelens",
    about = "Audit web pages for contrast, SEO, accessibility, and performance"
)]
#[command(version)]
pub struct Cli {
    #[arg(long, default_value_t = false)]
    pub serve: bool,

    #[arg(long, default_value = "127.0.0.1", requires = "serve")]
    pub host: String,

    #[arg(long, default_value_t = 8787, requires = "serve")]
    pub port: u16,

    #[arg(long, requires = "serve")]
    pub db_path: Option<String>,

    #[arg(required_unless_present = "serve")]
    pub url: Option<String>,

    /// Output results as JSON
    #[arg(long)]
    pub json: bool,

    /// Run only SEO checks
    #[arg(long)]
    pub seo: bool,

    #[arg(long, value_enum, default_value_t = AnalysisMode::Full)]
    pub mode: AnalysisMode,

    #[arg(long, help = "Include referenced JS/CSS/media assets in output")]
    pub assets: bool,

    #[arg(long, value_enum, default_value_t = AssetSort::SizeDesc)]
    pub asset_sort: AssetSort,

    #[arg(long, value_enum, default_value_t = AssetGroup::Type)]
    pub asset_group: AssetGroup,

    #[arg(long = "header", value_name = "KEY:VALUE", action = ArgAction::Append)]
    pub headers: Vec<String>,

    #[arg(long, help = "Include page paint and timing metrics")]
    pub perf: bool,

    #[arg(long, help = "Run basic text/background contrast audit")]
    pub contrast: bool,

    #[arg(long, help = "Analyze robots.txt and sitemap files")]
    pub site_files: bool,
}

impl Cli {
    pub fn validate_and_extract(&self) -> Result<(String, HashMap<String, String>), String> {
        let url = match self.url.as_deref() {
            Some(url) => url,
            None => {
                return Err("URL is required unless --serve is set".to_string());
            }
        };

        // Validate URL
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err("URL must start with http:// or https://".to_string());
        }

        let request_headers = parse_headers(&self.headers)?;

        Ok((url.to_string(), request_headers))
    }
}

/// Run the audit process
pub async fn run_audit(
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
    include_site_files: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use pagelens_core::browser::Browser;
    use pagelens_core::seo::SeoAnalyzer;
    use pagelens_core::site_files::SiteFilesAnalyzer;
    use pagelens_core::snapshot::{SnapshotExt, SnapshotOptions};

    let effective_mode = if seo_only_flag {
        AnalysisMode::SeoOnly
    } else {
        mode
    };
    let toggles = resolve_effective_toggles(
        effective_mode,
        include_assets,
        include_perf,
        include_contrast,
        include_site_files,
    );

    if !json_output {
        println!("🔍 PageLens Audit");
        println!("   URL: {}", url);
        println!();
    }

    // Launch browser
    if !json_output {
        println!("⏳ Launching browser...");
    }

    let browser = Browser::launch()
        .await
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
    snapshot_options.include_computed_styles = toggles.contrast;

    let snapshot = page
        .snapshot(snapshot_options)
        .await
        .map_err(|e| format!("Failed to capture snapshot: {e}"))?;

    // Run SEO analysis
    let seo_report = SeoAnalyzer::analyze(&snapshot);

    let asset_sizes = if toggles.assets {
        Some(analyze_asset_sizes(&snapshot.referenced_assets, request_headers).await)
    } else {
        None
    };

    let performance_report = if toggles.perf {
        Some(build_performance_report(&snapshot.performance_timing))
    } else {
        None
    };

    let contrast_report = if toggles.contrast {
        Some(analyze_contrast(&snapshot.computed_styles))
    } else {
        None
    };

    let (site_files_report, site_files_error) = if toggles.site_files {
        match SiteFilesAnalyzer::analyze(url).await {
            Ok(report) => (Some(report), None),
            Err(err) => (None, Some(err.to_string())),
        }
    } else {
        (None, None)
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
            "asset_sort": if toggles.assets { Some(asset_sort) } else { None },
            "asset_group": if toggles.assets { Some(asset_group) } else { None },
            "analysis_mode": effective_mode,
            "performance": performance_report,
            "contrast": contrast_report,
            "site_files": site_files_report,
            "site_files_error": site_files_error,
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
        if toggles.assets {
            if let Some(sizes) = &asset_sizes {
                print_assets_report(sizes, asset_sort, asset_group);
            }
        }
        if let Some(report) = &site_files_report {
            print_site_files_report(report);
        }
        if let Some(err) = &site_files_error {
            println!("\n🧭 Site Files");
            println!("  ─────────────────────────────────────────────────────────");
            println!("    ❌ Failed to analyze robots/sitemaps: {}", err);
            println!();
        }
    }

    // Shutdown browser
    browser.shutdown().await.ok();

    Ok(())
}
