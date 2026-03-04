use pagelens_cli::cli::{
    parse_content_range_total, sort_asset_entries, AssetSizeEntry, AssetSort, AssetType,
};

#[test]
fn parses_content_range_total() {
    assert_eq!(parse_content_range_total("bytes 0-0/12345"), Some(12345));
    assert_eq!(parse_content_range_total("bytes 0-1023/2048"), Some(2048));
    assert_eq!(parse_content_range_total("bytes */*"), None);
    assert_eq!(parse_content_range_total("garbage"), None);
}

#[test]
fn sort_by_size_desc_orders_largest_first() {
    let mut entries = vec![
        AssetSizeEntry {
            url: "https://a.example/small.js".to_string(),
            asset_type: AssetType::JavaScript,
            bytes: Some(10),
            source: "head",
            error: None,
        },
        AssetSizeEntry {
            url: "https://a.example/unknown.js".to_string(),
            asset_type: AssetType::JavaScript,
            bytes: None,
            source: "head",
            error: Some("missing".to_string()),
        },
        AssetSizeEntry {
            url: "https://a.example/large.js".to_string(),
            asset_type: AssetType::JavaScript,
            bytes: Some(100),
            source: "head",
            error: None,
        },
    ];

    sort_asset_entries(&mut entries, AssetSort::SizeDesc);

    assert_eq!(entries[0].url, "https://a.example/large.js");
    assert_eq!(entries[1].url, "https://a.example/small.js");
    assert_eq!(entries[2].url, "https://a.example/unknown.js");
}
