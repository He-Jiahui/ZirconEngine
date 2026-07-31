use glyphon::cosmic_text::{fontdb, CacheKey, CacheKeyFlags, SubpixelBin, Weight};

use super::*;

#[test]
fn native_bitmap_atlas_physical_cache_key_rebuckets_at_2x_scale() {
    let base = native_bitmap_atlas_raster_key_from_physical_cache(
        InstancedFaceId(7),
        physical_cache_key(12.0),
        GlyphAtlasFormat::AlphaMask,
    )
    .expect("alpha bitmap cache key should project");
    let high_dpi = native_bitmap_atlas_raster_key_from_physical_cache(
        InstancedFaceId(7),
        physical_cache_key(24.0),
        GlyphAtlasFormat::AlphaMask,
    )
    .expect("alpha bitmap cache key should project");

    assert_eq!(base.px_size_bucket, 12);
    assert_eq!(high_dpi.px_size_bucket, 24);
    assert_ne!(base, high_dpi);
}

fn physical_cache_key(physical_px: f32) -> CacheKey {
    CacheKey {
        font_id: fontdb::ID::dummy(),
        glyph_id: 42,
        font_size_bits: physical_px.to_bits(),
        x_bin: SubpixelBin::Zero,
        y_bin: SubpixelBin::Zero,
        font_weight: Weight(400),
        flags: CacheKeyFlags::empty(),
    }
}
