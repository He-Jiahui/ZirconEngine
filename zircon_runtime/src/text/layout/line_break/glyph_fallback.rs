#[cfg(test)]
use super::super::measure::measure_line_width;
#[cfg(test)]
use super::super::measure::measure_line_width_with_provider;
#[cfg(test)]
use crate::text::TextStyle;
#[cfg(test)]
use crate::text::shaping::DirectTextShapeRunProvider;
#[cfg(test)]
use crate::text::shaping::TextShapeRunProvider;
use unicode_segmentation::UnicodeSegmentation;

#[cfg(test)]
pub(super) fn should_fallback_to_glyph_wrap(
    allow_glyph_fallback: bool,
    candidate_text: &str,
    max_width: f32,
    style: &TextStyle,
) -> bool {
    let mut provider = DirectTextShapeRunProvider::default();
    should_fallback_to_glyph_wrap_with_provider(
        allow_glyph_fallback,
        candidate_text,
        max_width,
        style,
        &mut provider,
    )
    .into_result()
    .expect("test shaping request must be valid")
}

#[cfg(test)]
pub(super) fn should_fallback_to_glyph_wrap_with_provider<P>(
    allow_glyph_fallback: bool,
    candidate_text: &str,
    max_width: f32,
    style: &TextStyle,
    provider: &mut P,
) -> crate::text::shaping::TextLayoutOutcome<bool>
where
    P: TextShapeRunProvider + ?Sized,
{
    line_text_fits_with_provider(candidate_text, max_width, style, provider)
        .map(|fits| allow_glyph_fallback && !fits && has_more_than_one_grapheme(candidate_text))
}

pub(super) fn should_fallback_to_glyph_wrap_with_advance(
    allow_glyph_fallback: bool,
    candidate_text: &str,
    candidate_advance: f32,
    max_width: f32,
) -> bool {
    allow_glyph_fallback
        && candidate_advance.is_finite()
        && candidate_advance > max_width.max(0.0) + 0.01
        && has_more_than_one_grapheme(candidate_text)
}

#[cfg(test)]
fn line_text_fits(text: &str, max_width: f32, style: &TextStyle) -> bool {
    measure_line_width(text, style)
        .into_result()
        .expect("measure test line width")
        <= max_width + 0.01
}

#[cfg(test)]
fn line_text_fits_with_provider<P>(
    text: &str,
    max_width: f32,
    style: &TextStyle,
    provider: &mut P,
) -> crate::text::shaping::TextLayoutOutcome<bool>
where
    P: TextShapeRunProvider + ?Sized,
{
    measure_line_width_with_provider(text, style, provider).map(|width| width <= max_width + 0.01)
}

fn has_more_than_one_grapheme(text: &str) -> bool {
    text.graphemes(true).nth(1).is_some()
}

#[cfg(test)]
mod tests {
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
}
