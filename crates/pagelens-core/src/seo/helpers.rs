//! SEO analysis helper functions

use scraper::{Html, Selector};
use std::sync::LazyLock;

// Pre-compiled CSS selectors for HTML parsing
static META_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("meta").expect("valid meta selector"));
static LINK_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("link").expect("valid link selector"));

/// Get meta tag content by name attribute.
pub fn get_meta_content(document: &Html, name: &str) -> Option<String> {
    document
        .select(&META_SELECTOR)
        .find(|el| {
            el.value()
                .attr("name")
                .map(|n| n.eq_ignore_ascii_case(name))
                .unwrap_or(false)
        })
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.to_string())
}

/// Get meta tag content by property attribute (for Open Graph).
pub fn get_meta_property(document: &Html, property: &str) -> Option<String> {
    document
        .select(&META_SELECTOR)
        .find(|el| {
            el.value()
                .attr("property")
                .map(|p| p.eq_ignore_ascii_case(property))
                .unwrap_or(false)
        })
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.to_string())
}

/// Get meta tag content by name attribute (alias for get_meta_content).
pub fn get_meta_name(document: &Html, name: &str) -> Option<String> {
    get_meta_content(document, name)
}

pub fn get_meta_name_or_property(document: &Html, key: &str) -> Option<String> {
    get_meta_name(document, key).or_else(|| get_meta_property(document, key))
}

pub fn get_link_rels(document: &Html, rel: &str) -> Vec<String> {
    document
        .select(&LINK_SELECTOR)
        .filter(|el| {
            el.value()
                .attr("rel")
                .map(|rels| {
                    rels.split_ascii_whitespace()
                        .any(|token| token.eq_ignore_ascii_case(rel))
                })
                .unwrap_or(false)
        })
        .filter_map(|el| el.value().attr("href"))
        .map(std::string::ToString::to_string)
        .collect()
}

pub fn is_expected_hreflang_code(hreflang: &str) -> bool {
    if hreflang.eq_ignore_ascii_case("x-default") {
        return true;
    }

    let mut parts = hreflang.split('-');
    let Some(lang) = parts.next() else {
        return false;
    };

    if lang.len() < 2 || lang.len() > 3 || !lang.chars().all(|c| c.is_ascii_alphabetic()) {
        return false;
    }

    parts.all(|part| {
        let len = part.len();
        (2..=8).contains(&len) && part.chars().all(|c| c.is_ascii_alphanumeric())
    })
}

pub fn timing_delta(value: Option<u64>, navigation_start: u64) -> Option<u64> {
    value.and_then(|v| v.checked_sub(navigation_start))
}

pub fn find_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    haystack
        .to_ascii_lowercase()
        .find(&needle.to_ascii_lowercase())
}

pub fn parse_color(value: &str) -> Option<(u8, u8, u8)> {
    let v = value.trim();
    if let Some(hex) = v.strip_prefix('#') {
        return match hex.len() {
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
            _ => None,
        };
    }

    let open = v.find('(')?;
    let close = v.rfind(')')?;
    if close <= open {
        return None;
    }

    let body = &v[open + 1..close];
    let channels: Vec<&str> = body.split(',').collect();
    if channels.len() < 3 {
        return None;
    }

    let r = channels[0].trim().parse::<u8>().ok()?;
    let g = channels[1].trim().parse::<u8>().ok()?;
    let b = channels[2].trim().parse::<u8>().ok()?;
    Some((r, g, b))
}

pub fn is_large_text(font_size: Option<&str>, font_weight: Option<&str>) -> bool {
    let size = font_size
        .and_then(|v| v.trim_end_matches("px").trim().parse::<f64>().ok())
        .unwrap_or(0.0);
    let weight = font_weight
        .and_then(|v| v.trim().parse::<u16>().ok())
        .unwrap_or(400);

    size >= 18.0 || (size >= 14.0 && weight >= 700)
}

pub fn relative_luminance((r, g, b): (u8, u8, u8)) -> f64 {
    fn to_linear(channel: u8) -> f64 {
        let c = channel as f64 / 255.0;
        if c <= 0.039_28 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    let r = to_linear(r);
    let g = to_linear(g);
    let b = to_linear(b);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

pub fn contrast_ratio(fg: (u8, u8, u8), bg: (u8, u8, u8)) -> f64 {
    let l1 = relative_luminance(fg);
    let l2 = relative_luminance(bg);
    let (lighter, darker) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (lighter + 0.05) / (darker + 0.05)
}

pub fn header_contains_token(
    headers: &std::collections::HashMap<String, String>,
    header_name: &str,
    token: &str,
) -> bool {
    headers
        .get(header_name)
        .map(|value| {
            value
                .split(',')
                .map(|part| part.trim().to_ascii_lowercase())
                .any(|part| part == token)
        })
        .unwrap_or(false)
}

pub fn extract_hsts_max_age(header_value: &str) -> Option<u64> {
    header_value
        .split(';')
        .map(|part| part.trim())
        .find_map(|part| {
            let (key, value) = part.split_once('=')?;
            if key.trim().eq_ignore_ascii_case("max-age") {
                value.trim().parse::<u64>().ok()
            } else {
                None
            }
        })
}

pub fn should_check_secure_transport(page_url: &str) -> bool {
    let Ok(url) = url::Url::parse(page_url) else {
        return false;
    };

    let Some(host) = url.host_str() else {
        return false;
    };

    !host.eq_ignore_ascii_case("localhost")
        && host != "127.0.0.1"
        && host != "::1"
        && !host.ends_with(".local")
}
