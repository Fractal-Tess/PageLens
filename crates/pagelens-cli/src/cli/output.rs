//! Output formatting and printing

use crate::cli::analysis::{
    AssetGroup, AssetSizeEntry, AssetSizeSummary, AssetSort, AssetType, ContrastReport,
    PerformanceReport,
};
use crate::cli::assets::{format_bytes, sort_asset_entries};
use pagelens_core::seo::Severity;
use std::collections::BTreeMap;

pub fn print_performance_report(report: &PerformanceReport) {
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

pub fn print_metric(label: &str, value_ms: Option<u64>) {
    match value_ms {
        Some(value) => println!("  {}: {} ms", label, value),
        None => println!("  {}: n/a", label),
    }
}

pub fn print_contrast_report(report: &ContrastReport) {
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
                issue.selector, issue.ratio, issue.foreground, issue.background
            );
        }
    }
    println!();
}

pub fn print_assets_report(summary: &AssetSizeSummary, sort: AssetSort, group: AssetGroup) {
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

pub fn print_group_entries(label: &str, entries: &[AssetSizeEntry]) {
    let known = entries.iter().filter_map(|e| e.bytes).sum::<u64>();
    println!(
        "  {}: {} ({} files)",
        label,
        format_bytes(known),
        entries.len()
    );
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

pub fn asset_sort_label(sort: AssetSort) -> &'static str {
    match sort {
        AssetSort::UrlAsc => "url-asc",
        AssetSort::UrlDesc => "url-desc",
        AssetSort::SizeAsc => "size-asc",
        AssetSort::SizeDesc => "size-desc",
    }
}

pub fn asset_group_label(group: AssetGroup) -> &'static str {
    match group {
        AssetGroup::Type => "type",
        AssetGroup::Host => "host",
        AssetGroup::None => "none",
    }
}

pub fn print_site_files_report(report: &pagelens_core::site_files::SiteFilesReport) {
    println!("\n🧭 Site Files");
    println!("  ─────────────────────────────────────────────────────────");
    println!("  Base URL: {}", report.base_url);
    println!(
        "  robots.txt: {}{}",
        if report.robots.found {
            "found"
        } else {
            "missing"
        },
        report
            .robots
            .status
            .map(|s| format!(" (status {})", s))
            .unwrap_or_default()
    );
    println!("  Sitemaps checked: {}", report.sitemaps.len());
    println!("  Sitemap URLs discovered: {}", report.sitemap_urls.len());

    if report.issues.is_empty() {
        println!("  ✅ No robots/sitemap issues detected\n");
        return;
    }

    println!("  Issues ({})", report.issues.len());
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

pub fn print_results(report: &pagelens_core::seo::SeoReport) {
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

    println!(
        "  Overall Score: {}{:.0}/100{}\n",
        score_color, report.score, reset
    );

    // Meta Tags
    println!("  📋 Meta Tags");
    println!("  ─────────────────────────────────────────────────────────");

    if let Some(title) = &report.meta.title {
        let title_status = if title.len() <= 60 { "✅" } else { "⚠️ " };
        println!(
            "    {} Title: {} ({} chars)",
            title_status,
            title,
            title.len()
        );
    } else {
        println!("    ❌ Title: Missing");
    }

    if let Some(desc) = &report.meta.description {
        let desc_status = if desc.len() >= 50 && desc.len() <= 160 {
            "✅"
        } else {
            "⚠️ "
        };
        println!(
            "    {} Description: {} ({} chars)",
            desc_status,
            desc,
            desc.len()
        );
    } else {
        println!("    ❌ Description: Missing");
    }

    println!(
        "    {} Charset",
        if report.meta.charset { "✅" } else { "❌" }
    );
    println!(
        "    {} Viewport",
        if report.meta.viewport { "✅" } else { "❌" }
    );

    if let Some(lang) = &report.meta.language {
        println!("    ✅ Language: {}", lang);
    } else {
        println!("    ⚠️  Language: Not specified");
    }
    println!();

    // Open Graph
    println!("  🔗 Open Graph Tags");
    println!("  ─────────────────────────────────────────────────────────");
    println!(
        "    {} og:title",
        if report.open_graph.title.is_some() {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "    {} og:description",
        if report.open_graph.description.is_some() {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "    {} og:type",
        if report.open_graph.og_type.is_some() {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "    {} og:url",
        if report.open_graph.url.is_some() {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "    {} og:image",
        if report.open_graph.image.is_some() {
            "✅"
        } else {
            "❌"
        }
    );
    println!();

    // Twitter Cards
    println!("  🐦 Twitter Cards");
    println!("  ─────────────────────────────────────────────────────────");
    println!(
        "    {} twitter:card",
        if report.twitter_card.card.is_some() {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "    {} twitter:title",
        if report.twitter_card.title.is_some() {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "    {} twitter:description",
        if report.twitter_card.description.is_some() {
            "✅"
        } else {
            "❌"
        }
    );
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
    println!(
        "    H1: {}  H2: {}  H3: {}  H4: {}  H5: {}  H6: {}",
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
