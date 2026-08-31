use crate::text::FontFaceId;
use crate::text::font::FontDatabase;
use crate::text::sdf::{SdfGlyphData, SdfGlyphGenerationError};

use super::{RawBakedGlyph, RawBakedGlyphSource, SdfAtlasGlyphKey, SdfGlyphMetrics};

pub(super) fn raw_baked_glyph(glyph: SdfGlyphData) -> RawBakedGlyph {
    let metrics = SdfGlyphMetrics {
        bitmap_width: glyph.size.x,
        bitmap_height: glyph.size.y,
        bitmap_left: glyph.bitmap_left,
        bitmap_bottom: glyph.bitmap_bottom,
        advance: glyph.advance,
        ascent: glyph.ascent,
    };
    let visible =
        glyph.size.x > 0 && glyph.size.y > 0 && glyph.pixels.iter().any(|sample| *sample != 0);

    RawBakedGlyph {
        metrics,
        bitmap: glyph.pixels.into(),
        visible,
        generation_error: None,
        source: RawBakedGlyphSource::Dynamic,
    }
}

pub(super) fn glyph_id_for_key(
    key: &SdfAtlasGlyphKey,
    face_id: FontFaceId,
    resolved_shaped_face: Option<FontFaceId>,
    font_database: &FontDatabase,
) -> Result<u16, SdfGlyphGenerationError> {
    if let Some(glyph_id) = super::shaped_glyph_id_for_face(key, face_id, resolved_shaped_face) {
        return Ok(glyph_id);
    }
    font_database
        .face_glyph_id(face_id, key.glyph)
        .map_err(|_| SdfGlyphGenerationError::InvalidFaceIndex(0))?
        .ok_or(SdfGlyphGenerationError::MissingGlyphOutline(0))
}
