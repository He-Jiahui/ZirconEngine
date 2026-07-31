use crate::text::FontFaceId;

use super::database::FontDatabase;
use super::face_metadata::FontFaceMetadata;

#[derive(Clone, Copy)]
pub(crate) struct FontVerticalMetrics<'a> {
    metadata: &'a FontFaceMetadata,
    unit_scale: f32,
}

impl FontVerticalMetrics<'_> {
    pub(crate) fn glyph_advance_px(self, glyph_id: u32) -> Option<f32> {
        let advance = f32::from(self.metadata.vertical_advance(glyph_id)?) * self.unit_scale;
        (advance.is_finite() && advance > 0.0).then_some(advance)
    }
}

impl FontDatabase {
    /// Borrows generation-owned vertical metrics once for a face/run. Callers
    /// can reuse the view for every shaped glyph without repeating database or
    /// SFNT-table lookup.
    pub(crate) fn vertical_metrics(
        &self,
        face: FontFaceId,
        font_size: f32,
    ) -> Option<FontVerticalMetrics<'_>> {
        let metadata = self.face_metadata(face).ok()?;
        let metrics = metadata.face_metrics()?;
        Some(FontVerticalMetrics {
            metadata,
            unit_scale: font_size.max(1.0) / f32::from(metrics.units_per_em).max(1.0),
        })
    }
}
