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

/// 折行候选的兜底拆分许可：只允许不会拆散连接字符或变体选择序列的候选。
/// 调用方据此决定能否再按字形切开已整形的块，而不是判断字体是否具备字符覆盖。
pub(super) fn allows_glyph_fallback(text: &str) -> bool {
    if text.is_ascii() {
        return true;
    }
    text.chars()
        .all(|character| !is_glue_character(character) && !is_variation_selector(character))
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
#[path = "tests/glue.rs"]
mod tests;
