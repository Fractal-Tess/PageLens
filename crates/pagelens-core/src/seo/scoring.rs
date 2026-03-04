//! SEO scoring logic

use crate::seo::types::{ScoreConfig, SeoReport, Severity};
use crate::snapshot::Snapshot;

/// Calculate overall SEO score.
pub fn calculate_score(report: &SeoReport, snapshot: &Snapshot, score_config: &ScoreConfig) -> f64 {
    let mut issue_score = 100.0;

    // Deduct points for errors
    let error_count = report
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .count();
    issue_score -= error_count as f64 * score_config.error_penalty;

    // Deduct points for warnings
    let warning_count = report
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Warning)
        .count();
    issue_score -= warning_count as f64 * score_config.warning_penalty;

    // Deduct points for info (minor)
    let info_count = report
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Info)
        .count();
    issue_score -= info_count as f64 * score_config.info_penalty;

    // Bonus for good practices
    if report.meta.title.is_some() {
        issue_score += score_config.title_bonus;
    }
    if report.meta.description.is_some() {
        issue_score += score_config.description_bonus;
    }
    if report.canonical_url.is_some() {
        issue_score += score_config.canonical_bonus;
    }
    if !report.structured_data.is_empty() {
        issue_score += score_config.structured_data_bonus;
    }

    issue_score = issue_score.clamp(0.0, 100.0);

    let Some(performance_score) = calculate_performance_score(snapshot) else {
        return issue_score;
    };

    let issue_weight = score_config.issue_weight.max(0.0);
    let performance_weight = score_config.performance_weight.max(0.0);
    let total_weight = issue_weight + performance_weight;
    if total_weight <= f64::EPSILON {
        return issue_score;
    }

    ((issue_score * issue_weight) + (performance_score * performance_weight)) / total_weight
}

fn calculate_performance_score(snapshot: &Snapshot) -> Option<f64> {
    let timing = &snapshot.performance_timing;
    let nav_start = timing.navigation_start;
    if nav_start == 0 {
        return None;
    }

    let mut weighted_scores = Vec::new();

    if let Some(ttfb_ms) = timing_delta(timing.response_start, nav_start) {
        weighted_scores.push((log_normal_metric_score(ttfb_ms as f64, 800.0, 1800.0), 0.25));
    }

    if let Some(fcp_ms) = timing_delta(timing.first_contentful_paint, nav_start) {
        weighted_scores.push((log_normal_metric_score(fcp_ms as f64, 1800.0, 3000.0), 0.25));
    }

    if let Some(lcp_ms) = timing_delta(timing.largest_contentful_paint, nav_start) {
        weighted_scores.push((log_normal_metric_score(lcp_ms as f64, 2500.0, 4000.0), 0.3));
    }

    if let Some(cls) = timing.cumulative_layout_shift {
        if cls >= 0.0 {
            weighted_scores.push((log_normal_metric_score(cls.max(0.01), 0.1, 0.25), 0.1));
        }
    }

    if let Some(inp_ms) = timing.interaction_to_next_paint {
        weighted_scores.push((log_normal_metric_score(inp_ms as f64, 200.0, 500.0), 0.1));
    }

    if weighted_scores.is_empty() {
        return None;
    }

    let total_weight: f64 = weighted_scores.iter().map(|(_, w)| *w).sum();
    let normalized: f64 = weighted_scores
        .iter()
        .map(|(score, weight)| score * weight)
        .sum::<f64>()
        / total_weight;

    Some((normalized * 100.0).clamp(0.0, 100.0))
}

fn log_normal_metric_score(value: f64, p10: f64, median: f64) -> f64 {
    if value <= 0.0 || p10 <= 0.0 || median <= 0.0 || p10 >= median {
        return 0.0;
    }

    let z_10 = 1.281_551_565_544_600_4_f64;
    let location = median.ln();
    let shape = ((median.ln() - p10.ln()) / z_10).max(1e-6);
    let standardized = (value.ln() - location) / shape;
    let cdf = normal_cdf(standardized);

    (1.0 - cdf).clamp(0.0, 1.0)
}

fn normal_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf_approximation(x / std::f64::consts::SQRT_2))
}

fn erf_approximation(x: f64) -> f64 {
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let a1 = 0.254_829_592_f64;
    let a2 = -0.284_496_736_f64;
    let a3 = 1.421_413_741_f64;
    let a4 = -1.453_152_027_f64;
    let a5 = 1.061_405_429_f64;
    let p = 0.327_591_1_f64;

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t + a3) * t + a2) * t + a1) * t) * (-x * x).exp();

    sign * y
}

fn timing_delta(value: Option<u64>, navigation_start: u64) -> Option<u64> {
    value.and_then(|v| v.checked_sub(navigation_start))
}
