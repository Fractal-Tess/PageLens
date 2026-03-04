use pagelens_core::snapshot::extract_referenced_assets;

#[test]
fn extracts_js_css_and_media_assets() {
    let html = r#"
        <html>
          <head>
            <script src="/assets/app.js"></script>
            <link rel="modulepreload" href="./chunk.js">
            <link rel="stylesheet" href="/assets/app.css">
          </head>
          <body>
            <img src="/img/hero.jpg" />
            <source srcset="/img/hero-1x.jpg 1x, /img/hero-2x.jpg 2x" />
          </body>
        </html>
    "#;

    let (assets, _favicon) = extract_referenced_assets(html, "https://example.com/page");

    assert!(assets
        .javascript
        .contains(&"https://example.com/assets/app.js".to_string()));
    assert!(assets
        .javascript
        .contains(&"https://example.com/chunk.js".to_string()));
    assert!(assets
        .stylesheets
        .contains(&"https://example.com/assets/app.css".to_string()));
    assert!(assets
        .media
        .contains(&"https://example.com/img/hero.jpg".to_string()));
    assert!(assets
        .media
        .contains(&"https://example.com/img/hero-1x.jpg".to_string()));
}

#[test]
fn deduplicates_assets_and_ignores_data_urls() {
    let html = r#"
        <html>
          <head>
            <script src="/assets/app.js"></script>
            <script src="/assets/app.js"></script>
          </head>
          <body>
            <img src="data:image/png;base64,abc" />
            <img src="/img/one.png" />
            <img src="/img/one.png" />
          </body>
        </html>
    "#;

    let (assets, _favicon) = extract_referenced_assets(html, "https://example.com");

    assert_eq!(assets.javascript.len(), 1);
    assert_eq!(
        assets.media,
        vec!["https://example.com/img/one.png".to_string()]
    );
}
