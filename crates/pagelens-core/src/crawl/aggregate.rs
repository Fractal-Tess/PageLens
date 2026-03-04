use super::types::{AggregateMetrics, CrawlResult, CrawledPage};

pub(super) fn calculate_aggregates(pages: &[CrawledPage]) -> AggregateMetrics {
    if pages.is_empty() {
        return AggregateMetrics::default();
    }

    let scores: Vec<f64> = pages
        .iter()
        .filter(|page| page.success)
        .map(|page| page.seo_report.score)
        .collect();

    if scores.is_empty() {
        return AggregateMetrics::default();
    }

    let avg = scores.iter().sum::<f64>() / scores.len() as f64;
    let min = scores
        .iter()
        .fold(f64::INFINITY, |acc, value| acc.min(*value));
    let max = scores
        .iter()
        .fold(f64::NEG_INFINITY, |acc, value| acc.max(*value));

    let total_issues: usize = pages.iter().map(|page| page.seo_report.issues.len()).sum();

    let total_errors: usize = pages
        .iter()
        .map(|page| {
            page.seo_report
                .issues
                .iter()
                .filter(|issue| matches!(issue.severity, crate::seo::Severity::Error))
                .count()
        })
        .sum();

    let total_warnings: usize = pages
        .iter()
        .map(|page| {
            page.seo_report
                .issues
                .iter()
                .filter(|issue| matches!(issue.severity, crate::seo::Severity::Warning))
                .count()
        })
        .sum();

    let mut score_distribution = vec![0, 0, 0, 0, 0];
    for score in &scores {
        let idx = match *score as usize {
            0..=20 => 0,
            21..=40 => 1,
            41..=60 => 2,
            61..=80 => 3,
            _ => 4,
        };
        score_distribution[idx] += 1;
    }

    AggregateMetrics {
        avg_seo_score: avg,
        min_seo_score: if min == f64::INFINITY { 0.0 } else { min },
        max_seo_score: if max == f64::NEG_INFINITY { 0.0 } else { max },
        total_issues,
        total_errors,
        total_warnings,
        score_distribution,
    }
}

pub(super) fn generate_summary(result: &CrawlResult) -> String {
    let aggregate = &result.aggregate;
    let stats = &result.stats;

    format!(
        "Crawl complete: {} pages crawled in {}ms. Average SEO score: {:.0}/100. Total issues: {} ({} errors, {} warnings).",
        stats.crawled_pages,
        stats.crawl_time_ms,
        aggregate.avg_seo_score,
        aggregate.total_issues,
        aggregate.total_errors,
        aggregate.total_warnings
    )
}
