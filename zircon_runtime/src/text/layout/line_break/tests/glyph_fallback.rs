use super::super::super::measure::measure_line_width;
use super::should_fallback_to_glyph_wrap;
use crate::text::TextStyle;

#[test]
fn overwide_plain_chunk_requests_glyph_wrap_fallback() {
    let style = TextStyle::default();
    let max_width = measure_line_width("a", &style)
        .into_result()
        .expect("measure test line width")
        + 0.1;

    assert!(should_fallback_to_glyph_wrap(
        true, "abcd", max_width, &style
    ));
}

#[test]
fn overwide_glue_chunk_does_not_request_glyph_wrap_fallback() {
    let style = TextStyle::default();
    let max_width = measure_line_width("a", &style)
        .into_result()
        .expect("measure test line width")
        + 0.1;

    assert!(!should_fallback_to_glyph_wrap(
        false,
        "a\u{2060}b",
        max_width,
        &style
    ));
}

#[test]
fn single_grapheme_chunk_does_not_request_glyph_wrap_fallback() {
    let style = TextStyle::default();

    assert!(!should_fallback_to_glyph_wrap(true, "W", 1.0, &style));
}
