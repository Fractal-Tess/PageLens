use pagelens_cli::cli::{color_luminance, parse_rgb_triplet};

#[test]
fn parse_rgb_triplet_supports_hex() {
    assert_eq!(parse_rgb_triplet("#ffffff"), Some((255, 255, 255)));
    assert_eq!(parse_rgb_triplet("#123"), Some((17, 34, 51)));
    assert_eq!(parse_rgb_triplet("#00000000"), None);
}

#[test]
fn color_luminance_handles_oklab_colors() {
    assert!(color_luminance("oklab(0.2 0 0)").is_some());
    assert!(color_luminance("oklab(0.95 0 0)").is_some());
}
