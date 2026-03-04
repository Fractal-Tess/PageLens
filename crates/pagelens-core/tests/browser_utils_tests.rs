use pagelens_core::browser::url_decode;

#[test]
fn url_decode_decodes_percent_and_plus() {
    assert_eq!(url_decode("hello%20world"), "hello world");
    assert_eq!(url_decode("foo+bar"), "foo bar");
    assert_eq!(url_decode("test%3C%3E"), "test<>");
}
