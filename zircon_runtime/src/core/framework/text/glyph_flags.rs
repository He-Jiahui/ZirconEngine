use super::TextVerticalGlyphDecisionBasis;

/// 排版与渲染共享的字形类别和簇首信息；纵排来源只在簇首有意义。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextGlyphFlags {
    pub cluster_start: bool,
    pub right_to_left: bool,
    pub whitespace: bool,
    pub space: bool,
    pub tab: bool,
    pub mandatory_break: bool,
    pub soft_break: bool,
    pub virtual_glyph: bool,
    /// Present only on a vertical cluster head.
    pub vertical_decision: Option<TextVerticalGlyphDecisionBasis>,
}
