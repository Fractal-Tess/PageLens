mod error;
mod prelude;

use clap::Parser;
use pagelens_core::browser::Browser;
use pagelens_core::seo::SeoAnalyzer;
use pagelens_core::snapshot::{SnapshotExt, SnapshotOptions};

#[derive(Parser)]
#[command(name = "pagelens", about = "Audit web pages for contrast, SEO, accessibility, and performance")]
#[command(version)]
struct Cli {
    /// URL to analyze
    url: Option<String>,
    
    /// Output results as JSON
    #[arg(long)]
    json: bool,
    
    /// Run only SEO checks
    #[arg(long)]
    seo: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let url = match cli.url {
        Some(u) => u,
        None => {
            eprintln!("Error: URL required");
            eprintln!("Usage: pagelens <url>");
            std::process::exit(1);
        }
    };

    // Validate URL
    if !url.starts_with("http://") && !url.starts_with("https://") {
        eprintln!("Error: URL must start with http:// or https://");
        std::process::exit(1);
    }

    if let Err(e) = run_audit(&url, cli.json, cli.seo).await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

async fn run_audit(url: &str, json_output: bool, _seo_only: bool) -> Result<(), Box<dyn std::error::Error>> {
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
    
    let page = browser.navigate(url).await
        .map_err(|e| format!("Failed to navigate: {e}"))?;

    // Capture snapshot
    if !json_output {
        println!("⏳ Capturing page snapshot...");
    }
    
    let snapshot = page.snapshot(SnapshotOptions::default()).await
        .map_err(|e| format!("Failed to capture snapshot: {e}"))?;

    // Run SEO analysis
    let seo_report = SeoAnalyzer::analyze(&snapshot);

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
            }
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        // Human-readable output
        print_results(&seo_report);
    }

    // Shutdown browser
    browser.shutdown().await.ok();

    Ok(())
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
