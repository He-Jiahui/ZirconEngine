use super::{trailing_wrap_space_byte_len, trim_leading_wrap_spaces};

#[test]
fn trim_leading_wrap_spaces_advances_source_offset() {
    let (trimmed, source_start) = trim_leading_wrap_spaces("  word", 8);

    assert_eq!(trimmed, "word");
    assert_eq!(source_start, 10);
}

#[test]
fn trim_leading_wrap_spaces_preserves_non_breaking_space() {
    let text = "\u{00a0}word";
    let (trimmed, source_start) = trim_leading_wrap_spaces(text, 8);

    assert_eq!(trimmed, text);
    assert_eq!(source_start, 8);
}

#[test]
fn trailing_wrap_space_byte_len_counts_only_ascii_spaces() {
    assert_eq!(trailing_wrap_space_byte_len("word  "), 2);
    assert_eq!(trailing_wrap_space_byte_len("word\u{00a0}"), 0);
    assert_eq!(trailing_wrap_space_byte_len("word\u{00a0} "), 1);
}
