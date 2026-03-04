use pagelens_core::seo::{get_meta_content, get_meta_name_or_property};
use scraper::Html;

#[test]
fn get_meta_content_reads_description() {
    let html = r#"<meta name="description" content="Test description">"#;
    let document = Html::parse_fragment(html);
    assert_eq!(
        get_meta_content(&document, "description"),
        Some("Test description".to_string())
    );
}

#[test]
fn get_meta_content_is_case_insensitive() {
    let html = r#"<meta name="DESCRIPTION" content="Test description">"#;
    let document = Html::parse_fragment(html);
    assert_eq!(
        get_meta_content(&document, "description"),
        Some("Test description".to_string())
    );
}

#[test]
fn get_meta_name_or_property_uses_property_fallback() {
    let html = r#"<meta property="twitter:card" content="summary_large_image">"#;
    let document = Html::parse_fragment(html);
    assert_eq!(
        get_meta_name_or_property(&document, "twitter:card"),
        Some("summary_large_image".to_string())
    );
}
