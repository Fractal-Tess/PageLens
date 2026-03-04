//! Asset size analysis

use crate::cli::analysis::{AssetSizeEntry, AssetSizeSummary, AssetSort, AssetType};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_LENGTH, CONTENT_RANGE, RANGE};
use std::collections::HashMap;
use std::time::Duration;

pub async fn analyze_asset_sizes(
    assets: &pagelens_core::snapshot::ReferencedAssets,
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
        entries.extend(
            analyze_category_sizes(&client, &assets.javascript, AssetType::JavaScript).await,
        );
        entries.extend(
            analyze_category_sizes(&client, &assets.stylesheets, AssetType::Stylesheet).await,
        );
        entries.extend(analyze_category_sizes(&client, &assets.media, AssetType::Media).await);
        let total_bytes = sum_known_bytes(&entries);

        AssetSizeSummary {
            entries,
            total_bytes,
        }
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
        AssetSizeSummary {
            entries,
            total_bytes: 0,
        }
    }
}

pub async fn analyze_category_sizes(
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

pub async fn fetch_asset_size(
    client: &reqwest::Client,
    url: &str,
    asset_type: AssetType,
) -> AssetSizeEntry {
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
            if !resp.status().is_success() && resp.status() != reqwest::StatusCode::PARTIAL_CONTENT
            {
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

pub fn extract_content_length(resp: &reqwest::Response) -> Option<u64> {
    let value = resp.headers().get(CONTENT_LENGTH)?;
    value.to_str().ok()?.parse::<u64>().ok()
}

pub fn extract_content_range_total(resp: &reqwest::Response) -> Option<u64> {
    let value = resp.headers().get(CONTENT_RANGE)?;
    let raw = value.to_str().ok()?;
    parse_content_range_total(raw)
}

pub fn parse_content_range_total(raw: &str) -> Option<u64> {
    let total = raw.split('/').nth(1)?;
    if total == "*" {
        return None;
    }
    total.parse::<u64>().ok()
}

pub fn sum_known_bytes(entries: &[AssetSizeEntry]) -> u64 {
    entries.iter().filter_map(|e| e.bytes).sum()
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        return format!("{:.2} MB", bytes as f64 / 1024.0 / 1024.0);
    }
    if bytes >= 1024 {
        return format!("{:.2} KB", bytes as f64 / 1024.0);
    }
    format!("{} B", bytes)
}

pub fn sort_asset_entries(entries: &mut [AssetSizeEntry], sort: AssetSort) {
    match sort {
        AssetSort::UrlAsc => entries.sort_by(|a, b| a.url.cmp(&b.url)),
        AssetSort::UrlDesc => entries.sort_by(|a, b| b.url.cmp(&a.url)),
        AssetSort::SizeAsc => entries.sort_by(|a, b| compare_size_then_url(a, b, true)),
        AssetSort::SizeDesc => entries.sort_by(|a, b| compare_size_then_url(a, b, false)),
    }
}

pub fn compare_size_then_url(a: &AssetSizeEntry, b: &AssetSizeEntry, asc: bool) -> std::cmp::Ordering {
    let left = a.bytes.unwrap_or(0);
    let right = b.bytes.unwrap_or(0);
    if asc {
        left.cmp(&right).then_with(|| a.url.cmp(&b.url))
    } else {
        right.cmp(&left).then_with(|| a.url.cmp(&b.url))
    }
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
