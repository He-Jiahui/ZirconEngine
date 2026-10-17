//! 为已形成的折行候选计算是否允许继续兜底拆分。
//! 这里只筛显式连接字符和变体选择器，不检查字体覆盖或完整 Unicode 断行规则。

// 以下六个字符一旦出现在候选中，就关闭本层兜底拆分许可。
const NON_BREAKING_HYPHEN: char = '\u{2011}';
const NON_BREAKING_SPACE: char = '\u{00a0}';
const NARROW_NON_BREAKING_SPACE: char = '\u{202f}';
const WORD_JOINER: char = '\u{2060}';
const ZERO_WIDTH_NON_BREAKING_SPACE: char = '\u{feff}';
const ZERO_WIDTH_JOINER: char = '\u{200d}';
// 两组变体选择器按 Unicode 标量值闭区间检查。
const VARIATION_SELECTOR_START: char = '\u{fe00}';
const VARIATION_SELECTOR_END: char = '\u{fe0f}';
const SUPPLEMENTARY_VARIATION_SELECTOR_START: char = '\u{e0100}';
const SUPPLEMENTARY_VARIATION_SELECTOR_END: char = '\u{e01ef}';

/// 折行候选的兜底拆分许可：只允许不会拆散连接字符或变体选择序列的候选。
/// 调用方据此决定能否再按字形切开已整形的块，而不是判断字体是否具备字符覆盖。
/// 借用 `text`，返回本表许可；ASCII（含空串）直接通过，任一禁项即返回 `false`。
/// 父模块将结果存入 [`super::LineBreakChunk`]，后续规则仍可关闭许可。
pub(super) fn allows_glyph_fallback(text: &str) -> bool {
    // ASCII 字节不会命中下方列出的非 ASCII 禁项。
    if text.is_ascii() {
        return true;
    }
    // 逐字符筛选，遇到首个禁项即停止；不分析前后字符组成的完整序列。
    text.chars()
        .all(|character| !is_glue_character(character) && !is_variation_selector(character))
}

/// 仅检查上述六个显式连接字符，不扫描相邻字符或验证序列。
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

/// 检查两组选择器闭区间，不验证它前方是否存在基础字符。
fn is_variation_selector(ch: char) -> bool {
    (VARIATION_SELECTOR_START..=VARIATION_SELECTOR_END).contains(&ch)
        || (SUPPLEMENTARY_VARIATION_SELECTOR_START..=SUPPLEMENTARY_VARIATION_SELECTOR_END)
            .contains(&ch)
}

#[cfg(test)]
#[path = "tests/glue.rs"]
mod tests;
