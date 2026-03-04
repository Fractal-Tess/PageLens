use pagelens_core::crawl::{extract_links, resolve_url, url_decode, CrawlOptions};

#[test]
fn extract_links_skips_anchors_and_js() {
    let html = "<a href=\"/about\">About</a>\
                <a href=\"https://example.com/page\">External</a>\
                <a href=\"#anchor\">Anchor</a>\
                <a href=\"javascript:void(0)\">JS</a>";

    let links = extract_links(html, "https://mysite.com/");

    assert_eq!(links.len(), 2);
    assert!(links.contains(&"https://mysite.com/about".to_string()));
    assert!(links.contains(&"https://example.com/page".to_string()));
}

#[test]
fn resolve_url_handles_absolute_and_relative() {
    assert_eq!(
        resolve_url("https://example.com/", "https://other.com/page"),
        Some("https://other.com/page".to_string())
    );

    assert_eq!(
        resolve_url("https://example.com/dir/", "page.html"),
        Some("https://example.com/dir/page.html".to_string())
    );

    assert_eq!(
        resolve_url("https://example.com/", "/page"),
        Some("https://example.com/page".to_string())
    );
}

#[test]
fn crawl_options_default_values_match_expected() {
    let options = CrawlOptions::default();
    assert_eq!(options.max_pages, 50);
    assert_eq!(options.max_depth, 3);
    assert!(!options.follow_external_links);
}

#[test]
fn url_decode_decodes_percent_and_plus() {
    assert_eq!(url_decode("hello%20world"), "hello world");
    assert_eq!(url_decode("foo+bar"), "foo bar");
    assert_eq!(url_decode("test%3C%3E"), "test<>");
}
