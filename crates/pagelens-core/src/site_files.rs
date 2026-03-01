use crate::prelude::*;
use crate::seo::Severity;
use regex::Regex;
use std::collections::{BTreeSet, HashSet, VecDeque};
use std::sync::LazyLock;
use url::Url;

static LOC_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)<loc>\s*(.*?)\s*</loc>").expect("valid regex"));

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SiteFileIssue {
    pub severity: Severity,
    pub category: String,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RobotsReport {
    pub url: String,
    pub status: Option<u16>,
    pub found: bool,
    pub sitemap_directives: Vec<String>,
    pub disallow_all_for_star: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SitemapKind {
    UrlSet,
    SitemapIndex,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SitemapReport {
    pub url: String,
    pub status: Option<u16>,
    pub found: bool,
    pub kind: SitemapKind,
    pub urls: Vec<String>,
    pub child_sitemaps: Vec<String>,
    pub parse_error: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MiscFileReport {
    pub path: String,
    pub url: String,
    pub status: Option<u16>,
    pub found: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SitemapCrawlDiff {
    pub in_sitemap_not_in_crawl: Vec<String>,
    pub in_crawl_not_in_sitemap: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SiteFilesReport {
    pub base_url: String,
    pub robots: RobotsReport,
    pub sitemaps: Vec<SitemapReport>,
    pub sitemap_urls: Vec<String>,
    pub misc_files: Vec<MiscFileReport>,
    pub crawl_diff: Option<SitemapCrawlDiff>,
    pub issues: Vec<SiteFileIssue>,
}

impl SiteFilesReport {
    fn add_issue(&mut self, severity: Severity, category: &str, message: impl Into<String>) {
        self.issues.push(SiteFileIssue {
            severity,
            category: category.to_string(),
            message: message.into(),
        });
    }
}

pub struct SiteFilesAnalyzer;

impl SiteFilesAnalyzer {
    pub async fn analyze(base_url: &str) -> Result<SiteFilesReport> {
        Self::analyze_with_crawled_urls(base_url, &[]).await
    }

    pub async fn analyze_with_crawled_urls(
        base_url: &str,
        crawled_urls: &[String],
    ) -> Result<SiteFilesReport> {
        let base = normalize_base_url(base_url)?;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| Error::ExtractionFailed(format!("Failed to build HTTP client: {e}")))?;

        let robots = fetch_robots(&client, &base).await;
        let mut report = SiteFilesReport {
            base_url: base.to_string(),
            robots,
            sitemaps: Vec::new(),
            sitemap_urls: Vec::new(),
            misc_files: Vec::new(),
            crawl_diff: None,
            issues: Vec::new(),
        };

        if !report.robots.found {
            report.add_issue(Severity::Warning, "robots", "robots.txt not found");
        }
        if report.robots.disallow_all_for_star {
            report.add_issue(
                Severity::Warning,
                "robots",
                "robots.txt disallows all crawling for user-agent *",
            );
        }

        let mut sitemap_queue: VecDeque<String> = VecDeque::new();
        let mut seen_sitemaps: HashSet<String> = HashSet::new();

        for sm in &report.robots.sitemap_directives {
            if let Some(abs) = absolutize_url(&base, sm) {
                sitemap_queue.push_back(abs);
            }
        }

        let default_sitemap = format!("{}/sitemap.xml", base.trim_end_matches('/'));
        if !sitemap_queue.iter().any(|u| u == &default_sitemap) {
            sitemap_queue.push_back(default_sitemap);
        }

        let max_sitemaps = 20usize;
        while let Some(sm_url) = sitemap_queue.pop_front() {
            if seen_sitemaps.len() >= max_sitemaps {
                report.add_issue(
                    Severity::Info,
                    "sitemap",
                    "Sitemap traversal limit reached (20 files)",
                );
                break;
            }
            if !seen_sitemaps.insert(sm_url.clone()) {
                continue;
            }

            let sm_report = fetch_sitemap(&client, &base, &sm_url).await;
            if !sm_report.found {
                report.add_issue(
                    Severity::Warning,
                    "sitemap",
                    format!("Sitemap not found: {}", sm_report.url),
                );
            }
            if let Some(err) = &sm_report.parse_error {
                report.add_issue(
                    Severity::Warning,
                    "sitemap",
                    format!("Failed parsing sitemap {}: {}", sm_report.url, err),
                );
            }

            for child in &sm_report.child_sitemaps {
                if !seen_sitemaps.contains(child) {
                    sitemap_queue.push_back(child.clone());
                }
            }

            report.sitemaps.push(sm_report);
        }

        let mut sitemap_url_set: BTreeSet<String> = BTreeSet::new();
        for sm in &report.sitemaps {
            for url in &sm.urls {
                sitemap_url_set.insert(canonicalize_url_for_compare(url));
            }
        }
        report.sitemap_urls = sitemap_url_set.iter().cloned().collect();

        if report.sitemap_urls.is_empty() {
            report.add_issue(Severity::Warning, "sitemap", "No URLs found in sitemap files");
        }

        let misc_paths = [
            "/manifest.webmanifest",
            "/.well-known/security.txt",
            "/ads.txt",
            "/humans.txt",
            "/llms.txt",
        ];

        for path in misc_paths {
            let file = fetch_misc_file(&client, &base, path).await;
            if !file.found {
                report.add_issue(
                    Severity::Info,
                    "site-files",
                    format!("Optional file not found: {}", path),
                );
            }
            report.misc_files.push(file);
        }

        if !crawled_urls.is_empty() {
            let crawl_set: BTreeSet<String> = crawled_urls
                .iter()
                .map(|u| canonicalize_url_for_compare(u))
                .collect();

            let sitemap_set: BTreeSet<String> = report.sitemap_urls.iter().cloned().collect();

            let in_sitemap_not_in_crawl: Vec<String> = sitemap_set
                .difference(&crawl_set)
                .cloned()
                .collect();
            let in_crawl_not_in_sitemap: Vec<String> = crawl_set
                .difference(&sitemap_set)
                .cloned()
                .collect();

            if !in_sitemap_not_in_crawl.is_empty() {
                report.add_issue(
                    Severity::Info,
                    "coverage",
                    format!(
                        "{} sitemap URLs were not discovered by crawl",
                        in_sitemap_not_in_crawl.len()
                    ),
                );
            }
            if !in_crawl_not_in_sitemap.is_empty() {
                report.add_issue(
                    Severity::Info,
                    "coverage",
                    format!(
                        "{} crawled URLs were not listed in sitemap",
                        in_crawl_not_in_sitemap.len()
                    ),
                );
            }

            report.crawl_diff = Some(SitemapCrawlDiff {
                in_sitemap_not_in_crawl,
                in_crawl_not_in_sitemap,
            });
        }

        Ok(report)
    }
}

fn normalize_base_url(base_url: &str) -> Result<String> {
    let mut parsed = Url::parse(base_url)
        .map_err(|e| Error::InvalidUrl(format!("{base_url} ({e})")))?;
    parsed.set_query(None);
    parsed.set_fragment(None);
    let mut s = parsed.to_string();
    while s.ends_with('/') {
        s.pop();
    }
    Ok(s)
}

fn absolutize_url(base: &str, candidate: &str) -> Option<String> {
    Url::parse(base)
        .ok()?
        .join(candidate)
        .ok()
        .map(|u| u.to_string())
}

fn canonicalize_url_for_compare(raw: &str) -> String {
    if let Ok(mut parsed) = Url::parse(raw) {
        parsed.set_query(None);
        parsed.set_fragment(None);
        let mut s = parsed.to_string();
        while s.ends_with('/') && s.len() > 1 {
            s.pop();
        }
        return s;
    }
    raw.to_string()
}

fn parse_robots_sitemaps(content: &str) -> Vec<String> {
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| {
            let (key, value) = line.split_once(':')?;
            if key.trim().eq_ignore_ascii_case("sitemap") {
                return Some(value.trim().to_string());
            }
            None
        })
        .collect()
}

fn parse_disallow_all_for_star(content: &str) -> bool {
    let mut in_star_section = false;
    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            if key.eq_ignore_ascii_case("user-agent") {
                in_star_section = value == "*";
                continue;
            }

            if in_star_section && key.eq_ignore_ascii_case("disallow") && value == "/" {
                return true;
            }
        }
    }
    false
}

fn parse_sitemap_kind(body: &str) -> SitemapKind {
    let l = body.to_ascii_lowercase();
    if l.contains("<sitemapindex") {
        SitemapKind::SitemapIndex
    } else if l.contains("<urlset") {
        SitemapKind::UrlSet
    } else {
        SitemapKind::Unknown
    }
}

fn parse_sitemap_loc_values(body: &str) -> Vec<String> {
    LOC_REGEX
        .captures_iter(body)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
        .collect()
}

async fn fetch_robots(client: &reqwest::Client, base_url: &str) -> RobotsReport {
    let url = format!("{}/robots.txt", base_url);
    let response = client.get(&url).send().await;

    match response {
        Ok(resp) => {
            let status = resp.status().as_u16();
            if !resp.status().is_success() {
                return RobotsReport {
                    url,
                    status: Some(status),
                    found: false,
                    sitemap_directives: Vec::new(),
                    disallow_all_for_star: false,
                };
            }

            let body = resp.text().await.unwrap_or_default();
            RobotsReport {
                url,
                status: Some(status),
                found: true,
                sitemap_directives: parse_robots_sitemaps(&body),
                disallow_all_for_star: parse_disallow_all_for_star(&body),
            }
        }
        Err(_) => RobotsReport {
            url,
            status: None,
            found: false,
            sitemap_directives: Vec::new(),
            disallow_all_for_star: false,
        },
    }
}

async fn fetch_sitemap(client: &reqwest::Client, base_url: &str, sitemap_url: &str) -> SitemapReport {
    let response = client.get(sitemap_url).send().await;
    match response {
        Ok(resp) => {
            let status = resp.status().as_u16();
            if !resp.status().is_success() {
                return SitemapReport {
                    url: sitemap_url.to_string(),
                    status: Some(status),
                    found: false,
                    kind: SitemapKind::Unknown,
                    urls: Vec::new(),
                    child_sitemaps: Vec::new(),
                    parse_error: None,
                };
            }

            let body = resp.text().await.unwrap_or_default();
            let kind = parse_sitemap_kind(&body);
            let locs = parse_sitemap_loc_values(&body);
            let mut urls = Vec::new();
            let mut child_sitemaps = Vec::new();

            for loc in locs {
                if let Some(abs) = absolutize_url(base_url, &loc) {
                    match kind {
                        SitemapKind::SitemapIndex => child_sitemaps.push(abs),
                        SitemapKind::UrlSet | SitemapKind::Unknown => urls.push(abs),
                    }
                }
            }

            let parse_error = if matches!(kind, SitemapKind::Unknown) {
                Some("Unrecognized sitemap XML root element".to_string())
            } else {
                None
            };

            SitemapReport {
                url: sitemap_url.to_string(),
                status: Some(status),
                found: true,
                kind,
                urls,
                child_sitemaps,
                parse_error,
            }
        }
        Err(e) => SitemapReport {
            url: sitemap_url.to_string(),
            status: None,
            found: false,
            kind: SitemapKind::Unknown,
            urls: Vec::new(),
            child_sitemaps: Vec::new(),
            parse_error: Some(e.to_string()),
        },
    }
}

async fn fetch_misc_file(client: &reqwest::Client, base_url: &str, path: &str) -> MiscFileReport {
    let url = format!("{}{}", base_url, path);
    let response = client.get(&url).send().await;
    match response {
        Ok(resp) => {
            let status = resp.status().as_u16();
            MiscFileReport {
                path: path.to_string(),
                url,
                status: Some(status),
                found: resp.status().is_success(),
            }
        }
        Err(_) => MiscFileReport {
            path: path.to_string(),
            url,
            status: None,
            found: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_robots_extracts_sitemaps() {
        let txt = "User-agent: *\nSitemap: https://example.com/sitemap.xml\nSitemap: /sitemap-blog.xml\n";
        let sitemaps = parse_robots_sitemaps(txt);
        assert_eq!(sitemaps.len(), 2);
        assert_eq!(sitemaps[0], "https://example.com/sitemap.xml");
        assert_eq!(sitemaps[1], "/sitemap-blog.xml");
    }

    #[test]
    fn parse_robots_detects_disallow_all() {
        let txt = "User-agent: *\nDisallow: /\n";
        assert!(parse_disallow_all_for_star(txt));
    }

    #[test]
    fn parse_sitemap_urlset_loc_values() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url><loc>https://example.com/</loc></url>
  <url><loc>https://example.com/about</loc></url>
</urlset>
"#;
        let locs = parse_sitemap_loc_values(xml);
        assert_eq!(locs.len(), 2);
    }

    #[test]
    fn parse_sitemap_kind_detection() {
        assert_eq!(parse_sitemap_kind("<urlset></urlset>"), SitemapKind::UrlSet);
        assert_eq!(
            parse_sitemap_kind("<sitemapindex></sitemapindex>"),
            SitemapKind::SitemapIndex
        );
        assert_eq!(parse_sitemap_kind("<html></html>"), SitemapKind::Unknown);
    }

    #[test]
    fn canonicalize_removes_fragment_query_and_trailing_slash() {
        let c = canonicalize_url_for_compare("https://example.com/about/?a=1#x");
        assert_eq!(c, "https://example.com/about");
    }
}
