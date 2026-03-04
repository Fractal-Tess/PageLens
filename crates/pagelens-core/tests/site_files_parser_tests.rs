use pagelens_core::site_files::{
    canonicalize_url_for_compare, parse_disallow_all_for_star, parse_robots_sitemaps,
    parse_sitemap_kind, parse_sitemap_loc_values, SitemapKind,
};

#[test]
fn parse_robots_extracts_sitemaps() {
    let txt =
        "User-agent: *\nSitemap: https://example.com/sitemap.xml\nSitemap: /sitemap-blog.xml\n";
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
