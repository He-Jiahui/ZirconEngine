use super::trim_block_delimiters;

#[test]
fn trim_block_delimiters_uses_all_canonical_hard_line_separators() {
    let text = "\r\n\u{2028}content\u{0085}\u{000b}";

    assert_eq!(
        trim_block_delimiters(text, 0, text.len()).expect("valid text range"),
        "\r\n\u{2028}".len().."\r\n\u{2028}content".len()
    );
}

#[test]
fn trim_block_delimiters_rejects_reversed_or_non_boundary_ranges() {
    let text = "a\u{4e2d}b";

    assert!(trim_block_delimiters(text, 3, 2).is_err());
    assert!(trim_block_delimiters(text, 2, text.len()).is_err());
    assert!(trim_block_delimiters(text, 0, text.len() + 1).is_err());
}
