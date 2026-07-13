use ttf_parser::{Face, GlyphId, Tag};

use crate::core::framework::render::{FontFaceId, InstancedFaceId, VariationCoords};
use crate::graphics::text::font::FontDatabase;
use crate::graphics::text::sdf::{
    generate_distance_field_glyph_with_variations, SdfGlyphGenerationError,
};

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
    let mut face =
        Face::parse(bytes.as_ref(), 0).map_err(|_| SdfGlyphGenerationError::InvalidFaceIndex(0))?;
    apply_variations(&mut face, &variations);
    let glyph_id = glyph_id_for_key(&face, key)?;
    let glyph = generate_distance_field_glyph_with_variations(
        bytes.as_ref(),
        0,
        glyph_id.0,
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
) -> VariationCoords {
    font_database
        .effective_instance_variations(
            face_id,
            key.font_instance_id.map(InstancedFaceId),
            key.font_weight,
        )
        .unwrap_or_default()
}

fn apply_variations(face: &mut Face<'_>, variations: &VariationCoords) {
    for (tag, value) in &variations.0 {
        let _ = face.set_variation(Tag::from_bytes(&tag.to_be_bytes()), *value);
    }
}

pub(super) fn glyph_id_for_key(
    face: &Face<'_>,
    key: &SdfAtlasGlyphKey,
) -> Result<GlyphId, SdfGlyphGenerationError> {
    if let Some(glyph_id) = key
        .glyph_id
        .and_then(|glyph_id| u16::try_from(glyph_id).ok())
    {
        return Ok(GlyphId(glyph_id));
    }
    face.glyph_index(key.glyph)
        .ok_or(SdfGlyphGenerationError::MissingGlyphOutline(0))
}
