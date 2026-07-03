const NON_BREAKING_HYPHEN: char = '\u{2011}';
const NON_BREAKING_SPACE: char = '\u{00a0}';
const NARROW_NON_BREAKING_SPACE: char = '\u{202f}';
const WORD_JOINER: char = '\u{2060}';
const ZERO_WIDTH_NON_BREAKING_SPACE: char = '\u{feff}';
const ZERO_WIDTH_JOINER: char = '\u{200d}';
const VARIATION_SELECTOR_START: char = '\u{fe00}';
const VARIATION_SELECTOR_END: char = '\u{fe0f}';
const SUPPLEMENTARY_VARIATION_SELECTOR_START: char = '\u{e0100}';
const SUPPLEMENTARY_VARIATION_SELECTOR_END: char = '\u{e01ef}';

pub(super) fn allows_glyph_fallback(text: &str) -> bool {
    !text.chars().any(is_glue_character) && !text.chars().any(is_variation_selector)
}

fn is_glue_character(ch: char) -> bool {
    matches!(
        ch,
        NON_BREAKING_HYPHEN
            | NON_BREAKING_SPACE
            | NARROW_NON_BREAKING_SPACE
            | WORD_JOINER
            | ZERO_WIDTH_NON_BREAKING_SPACE
            | ZERO_WIDTH_JOINER
    )
}

fn is_variation_selector(ch: char) -> bool {
    (VARIATION_SELECTOR_START..=VARIATION_SELECTOR_END).contains(&ch)
        || (SUPPLEMENTARY_VARIATION_SELECTOR_START..=SUPPLEMENTARY_VARIATION_SELECTOR_END)
            .contains(&ch)
}

#[cfg(test)]
mod tests {
    use super::allows_glyph_fallback;

    #[test]
    fn rejects_glyph_fallback_for_variation_selector_sequences() {
        assert!(!allows_glyph_fallback("✈\u{fe0f}"));
        assert!(!allows_glyph_fallback("禰\u{e0100}"));
    }

    #[test]
    fn rejects_glyph_fallback_for_additional_glue_characters() {
        for text in ["a\u{2011}b", "a\u{202f}b", "a\u{2060}b", "a\u{feff}b"] {
            assert!(!allows_glyph_fallback(text), "{text:?} must be glue");
        }
    }
}
