use super::super::measure::measure_line_width;
use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::surface::UiResolvedStyle;

pub(super) fn should_fallback_to_glyph_wrap(
    allow_glyph_fallback: bool,
    candidate_text: &str,
    max_width: f32,
    style: &UiResolvedStyle,
) -> bool {
    allow_glyph_fallback
        && !line_text_fits(candidate_text, max_width, style)
        && has_more_than_one_grapheme(candidate_text)
}

fn line_text_fits(text: &str, max_width: f32, style: &UiResolvedStyle) -> bool {
    measure_line_width(text, style) <= max_width + 0.01
}

fn has_more_than_one_grapheme(text: &str) -> bool {
    text.graphemes(true).nth(1).is_some()
}

#[cfg(test)]
mod tests {
    use super::super::super::measure::measure_line_width;
    use super::should_fallback_to_glyph_wrap;
    use zircon_runtime_interface::ui::surface::UiResolvedStyle;

    #[test]
    fn overwide_plain_chunk_requests_glyph_wrap_fallback() {
        let style = UiResolvedStyle::default();
        let max_width = measure_line_width("a", &style) + 0.1;

        assert!(should_fallback_to_glyph_wrap(
            true, "abcd", max_width, &style
        ));
    }

    #[test]
    fn overwide_glue_chunk_does_not_request_glyph_wrap_fallback() {
        let style = UiResolvedStyle::default();
        let max_width = measure_line_width("a", &style) + 0.1;

        assert!(!should_fallback_to_glyph_wrap(
            false,
            "a\u{2060}b",
            max_width,
            &style
        ));
    }

    #[test]
    fn single_grapheme_chunk_does_not_request_glyph_wrap_fallback() {
        let style = UiResolvedStyle::default();

        assert!(!should_fallback_to_glyph_wrap(true, "W", 1.0, &style));
    }
}
