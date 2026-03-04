//! Analysis types and functions

use clap::ValueEnum;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize)]
pub enum AnalysisMode {
    Full,
    Custom,
    SeoOnly,
}

#[derive(Debug, Clone, Copy)]
pub struct EffectiveToggles {
    pub assets: bool,
    pub perf: bool,
    pub contrast: bool,
    pub site_files: bool,
}

pub fn resolve_effective_toggles(
    mode: AnalysisMode,
    include_assets: bool,
    include_perf: bool,
    include_contrast: bool,
    include_site_files: bool,
) -> EffectiveToggles {
    match mode {
        AnalysisMode::Full => EffectiveToggles {
            assets: true,
            perf: true,
            contrast: true,
            site_files: true,
        },
        AnalysisMode::Custom => EffectiveToggles {
            assets: include_assets,
            perf: include_perf,
            contrast: include_contrast,
            site_files: include_site_files,
        },
        AnalysisMode::SeoOnly => EffectiveToggles {
            assets: false,
            perf: false,
            contrast: false,
            site_files: false,
        },
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PerformanceReport {
    pub navigation_start_ms: u64,
    pub ttfb_ms: Option<u64>,
    pub dom_interactive_ms: Option<u64>,
    pub dom_content_loaded_ms: Option<u64>,
    pub load_complete_ms: Option<u64>,
    pub first_paint_ms: Option<u64>,
    pub first_contentful_paint_ms: Option<u64>,
}

pub fn build_performance_report(
    timing: &pagelens_core::snapshot::PerformanceTiming,
) -> PerformanceReport {
    PerformanceReport {
        navigation_start_ms: timing.navigation_start,
        ttfb_ms: delta_from_nav(timing.navigation_start, timing.response_start),
        dom_interactive_ms: delta_from_nav(timing.navigation_start, timing.dom_interactive),
        dom_content_loaded_ms: delta_from_nav(timing.navigation_start, timing.dom_content_loaded),
        load_complete_ms: delta_from_nav(timing.navigation_start, timing.load_complete),
        first_paint_ms: delta_from_nav(timing.navigation_start, timing.first_paint),
        first_contentful_paint_ms: delta_from_nav(
            timing.navigation_start,
            timing.first_contentful_paint,
        ),
    }
}

fn delta_from_nav(navigation_start: u64, absolute: Option<u64>) -> Option<u64> {
    absolute.and_then(|value| value.checked_sub(navigation_start))
}

#[derive(Debug, Clone, Serialize)]
pub struct ContrastIssue {
    pub selector: String,
    pub ratio: f64,
    pub foreground: String,
    pub background: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContrastReport {
    pub analyzed_elements: usize,
    pub failing_elements: usize,
    pub min_ratio: Option<f64>,
    pub issues: Vec<ContrastIssue>,
}

pub fn analyze_contrast(styles: &[pagelens_core::snapshot::ComputedStyle]) -> ContrastReport {
    use crate::cli::colors::color_luminance;

    let mut analyzed = 0usize;
    let mut failing = 0usize;
    let mut min_ratio: Option<f64> = None;
    let mut issues = Vec::new();

    for style in styles {
        let (Some(fg), Some(bg)) = (style.color.as_ref(), style.background_color.as_ref()) else {
            continue;
        };

        let (Some(fg_luminance), Some(bg_luminance)) = (color_luminance(fg), color_luminance(bg))
        else {
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

    issues.sort_by(|a, b| {
        a.ratio
            .partial_cmp(&b.ratio)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    issues.truncate(20);

    ContrastReport {
        analyzed_elements: analyzed,
        failing_elements: failing,
        min_ratio,
        issues,
    }
}

fn contrast_ratio_from_luminance(la: f64, lb: f64) -> f64 {
    let (light, dark) = if la >= lb { (la, lb) } else { (lb, la) };
    (light + 0.05) / (dark + 0.05)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize)]
pub enum AssetSort {
    UrlAsc,
    UrlDesc,
    SizeAsc,
    SizeDesc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize)]
pub enum AssetGroup {
    Type,
    Host,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AssetType {
    JavaScript,
    Stylesheet,
    Media,
}

impl AssetType {
    pub fn label(self) -> &'static str {
        match self {
            AssetType::JavaScript => "JavaScript",
            AssetType::Stylesheet => "Stylesheets",
            AssetType::Media => "Media",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AssetSizeEntry {
    pub url: String,
    pub asset_type: AssetType,
    pub bytes: Option<u64>,
    pub source: &'static str,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssetSizeSummary {
    pub entries: Vec<AssetSizeEntry>,
    pub total_bytes: u64,
}

pub fn parse_headers(raw_headers: &[String]) -> Result<HashMap<String, String>, String> {
    use reqwest::header::{HeaderName, HeaderValue};

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
        HeaderName::from_bytes(name.as_bytes())
            .map_err(|e| format!("Invalid header '{raw}': invalid header name ({e})"))?;
        HeaderValue::from_str(value)
            .map_err(|e| format!("Invalid header '{raw}': invalid header value ({e})"))?;
        headers.insert(name.to_string(), value.to_string());
    }
    Ok(headers)
}
