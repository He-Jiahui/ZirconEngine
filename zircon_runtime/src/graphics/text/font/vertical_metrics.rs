use ttf_parser::{Face, GlyphId};

use crate::core::framework::render::FontFaceId;

use super::database::FontDatabase;

impl FontDatabase {
    /// Returns native `vmtx` advance in display pixels when the selected face
    /// exposes vertical metrics for the backend-shaped glyph.
    pub(crate) fn vertical_glyph_advance_px(
        &self,
        face: FontFaceId,
        glyph_id: u32,
        font_size: f32,
    ) -> Option<f32> {
        let bytes = self.face_bytes(face).ok()?;
        let face_index = self.face_index(face).ok()?;
        let parsed = Face::parse(bytes.as_ref(), face_index).ok()?;
        let glyph_id = GlyphId(u16::try_from(glyph_id).ok()?);
        let scale = font_size.max(1.0) / f32::from(parsed.units_per_em()).max(1.0);
        let advance = f32::from(parsed.glyph_ver_advance(glyph_id)?) * scale;
        (advance.is_finite() && advance > 0.0).then_some(advance)
    }
}
