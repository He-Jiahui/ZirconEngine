use glyphon::cosmic_text::{CacheKey, CacheKeyFlags, SubpixelBin};

use crate::text::atlas::{
    GlyphAtlasFormat, GlyphHintingMode, GlyphRasterKey, GlyphRasterRequest, GlyphSmoothingMode,
    SyntheticGlyphStyle,
};
use crate::text::font::FontDatabase;
use crate::text::InstancedFaceId;

pub(super) fn native_bitmap_atlas_raster_key(
    font_database: &FontDatabase,
    cache_key: CacheKey,
    format: Option<GlyphAtlasFormat>,
) -> Option<GlyphRasterKey> {
    let format = format?;
    let cache_key = super::source_cache::native_bitmap_atlas_stable_raster_cache_key(cache_key);
    if cache_key.flags.contains(CacheKeyFlags::PIXEL_FONT) {
        return None;
    }

    let face = font_database.font_face_id(cache_key.font_id)?;
    let instance = font_database
        .effective_instance_id(face, cache_key.font_weight.0)
        .ok()?;

    native_bitmap_atlas_raster_key_from_physical_cache(instance, cache_key, format)
}

fn native_bitmap_atlas_raster_key_from_physical_cache(
    instance: InstancedFaceId,
    cache_key: CacheKey,
    format: GlyphAtlasFormat,
) -> Option<GlyphRasterKey> {
    let smoothing = match format {
        GlyphAtlasFormat::SubpixelMask => GlyphSmoothingMode::Subpixel,
        GlyphAtlasFormat::AlphaMask => GlyphSmoothingMode::Grayscale,
        GlyphAtlasFormat::Color => GlyphSmoothingMode::None,
        GlyphAtlasFormat::Sdf | GlyphAtlasFormat::Msdf => return None,
    };
    let hinting = if cache_key.flags.contains(CacheKeyFlags::DISABLE_HINTING) {
        GlyphHintingMode::None
    } else {
        GlyphHintingMode::Full
    };
    // `LayoutGlyph::physical(..., scale)` has already multiplied the logical font size by the
    // surface scale before storing it in `CacheKey::font_size_bits`. Keep the generic request's
    // scale at one here so a 2x surface becomes a 2x bucket exactly once.
    let physical_px = f32::from_bits(cache_key.font_size_bits);
    let mut key = GlyphRasterKey::from_request(GlyphRasterRequest {
        face: instance,
        glyph_id: u32::from(cache_key.glyph_id),
        logical_px: physical_px,
        scale_factor: 1.0,
        screen_x: cache_key.x_bin.as_float(),
        snap_to_pixel: false,
        format,
        hinting,
        smoothing,
        synthetic: SyntheticGlyphStyle {
            bold: false,
            oblique: cache_key.flags.contains(CacheKeyFlags::FAKE_ITALIC),
        },
    });
    key.vertical_subpixel_bin = native_bitmap_atlas_subpixel_bin_index(cache_key.y_bin);
    Some(key)
}

const fn native_bitmap_atlas_subpixel_bin_index(bin: SubpixelBin) -> u8 {
    match bin {
        SubpixelBin::Zero => 0,
        SubpixelBin::One => 1,
        SubpixelBin::Two => 2,
        SubpixelBin::Three => 3,
    }
}

#[cfg(test)]
mod tests;
