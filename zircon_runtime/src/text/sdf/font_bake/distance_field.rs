use std::sync::Arc;

use crate::text::font::FontDatabase;
use crate::text::sdf::{generate_distance_field_glyph_with_variations, SdfGlyphGenerationError};
use crate::text::{FontFaceId, VariationCoords};

use super::{RawBakedGlyph, RawBakedGlyphSource, SdfAtlasGlyphKey, SdfGlyphMetrics};

pub(super) fn bake_distance_field_glyph(
    key: &SdfAtlasGlyphKey,
    face_id: FontFaceId,
    font_database: &FontDatabase,
) -> Result<RawBakedGlyph, SdfGlyphGenerationError> {
    let bytes = font_database
        .standalone_face_bytes(face_id)
        .map_err(|_| SdfGlyphGenerationError::InvalidFaceIndex(0))?;
    let variations = variations_for_key(key, face_id, font_database);
    let glyph_id = glyph_id_for_key(key, face_id, font_database)?;
    let glyph = generate_distance_field_glyph_with_variations(
        bytes.as_ref(),
        0,
        glyph_id,
        key.bake_params,
        &variations,
    )?;
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

    Ok(RawBakedGlyph {
        metrics,
        bitmap: glyph.pixels,
        visible,
        generation_error: None,
        source: RawBakedGlyphSource::Dynamic,
    })
}

fn variations_for_key(
    key: &SdfAtlasGlyphKey,
    face_id: FontFaceId,
    font_database: &FontDatabase,
) -> Arc<VariationCoords> {
    font_database
        .effective_instance_variations_shared(
            face_id,
            key.font_instance_id
                .and_then(crate::text::font::resolve_font_instance_handle),
            key.font_weight,
        )
        .unwrap_or_default()
}

pub(super) fn glyph_id_for_key(
    key: &SdfAtlasGlyphKey,
    face_id: FontFaceId,
    font_database: &FontDatabase,
) -> Result<u16, SdfGlyphGenerationError> {
    if let Some(glyph_id) = super::shaped_glyph_id_for_face(key, face_id, font_database) {
        return Ok(glyph_id);
    }
    font_database
        .face_glyph_id(face_id, key.glyph)
        .map_err(|_| SdfGlyphGenerationError::InvalidFaceIndex(0))?
        .ok_or(SdfGlyphGenerationError::MissingGlyphOutline(0))
}
