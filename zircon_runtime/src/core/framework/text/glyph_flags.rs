use super::TextVerticalGlyphDecisionBasis;

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
