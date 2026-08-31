use std::ops::Range;

use super::{TextFontFaceHandle, TextGlyphFlags, TextGlyphRotation, TextVerticalGlyphDecision};

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
