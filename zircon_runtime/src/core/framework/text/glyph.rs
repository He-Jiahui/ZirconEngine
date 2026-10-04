use std::ops::Range;

use super::{TextFontFaceHandle, TextGlyphFlags, TextGlyphRotation, TextVerticalGlyphDecision};

/// 形状输出的中立字形，保留源文本/可视范围及字体句柄供后续栅格化与命中处理。
///
/// 缺失字体句柄时调用方应走不可栅格化路径，不能复用旧代际的字体槽位。
#[derive(Clone, Debug, PartialEq)]
pub struct TextGlyph {
    pub glyph_id: u32,
    pub source_range: Range<usize>,
    pub visual_range: Range<usize>,
    pub advance: f32,
    pub position: [f32; 2],
    pub offset: [f32; 2],
    pub font_face: Option<TextFontFaceHandle>,
    pub font_instance: Option<TextFontFaceHandle>,
    pub rotation: TextGlyphRotation,
    pub bidi_level: u8,
    pub flags: TextGlyphFlags,
    pub requires_rasterization: bool,
}

impl TextGlyph {
    /// 仅从纵排簇首暴露方向判定收据，避免后续字形重复声明同一簇的决策。
    pub fn vertical_glyph_decision(&self) -> Option<TextVerticalGlyphDecision> {
        let basis = self
            .flags
            .cluster_start
            .then_some(self.flags.vertical_decision)
            .flatten()?;
        Some(TextVerticalGlyphDecision {
            basis,
            rotation: self.rotation,
            font_face: self.font_face,
            font_instance: self.font_instance,
        })
    }
}
