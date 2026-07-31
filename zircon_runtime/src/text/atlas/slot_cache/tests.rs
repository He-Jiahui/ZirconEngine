use super::*;
use crate::text::atlas::{
    GlyphAtlasFormat, GlyphHintingMode, GlyphSmoothingMode, SyntheticGlyphStyle,
};
use crate::text::InstancedFaceId;

#[test]
fn render_text_atlas_slot_cache_invalidates_allocator_and_slots_by_page() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 2);
    let key = raster_key(17);
    let mut cache = GlyphAtlasSlotCache::default();
    let allocation = cache
        .allocate(page_key, UVec2::new(32, 32), 2, UVec2::new(8, 8))
        .expect("first persistent allocation");
    cache.insert_slot(
        key,
        GlyphAtlasPersistentSlot {
            page_key,
            page_generation: 3,
            inserted_frame_index: 9,
            rect: allocation.rect,
            content_size: UVec2::new(8, 8),
        },
    );

    cache.invalidate_page(page_key);

    assert_eq!(cache.slot(key), None);
    assert_eq!(cache.slot_count(), 0);
    assert_eq!(
        cache
            .allocate(page_key, UVec2::new(32, 32), 2, UVec2::new(8, 8))
            .expect("allocator restarts after page invalidation")
            .rect,
        GlyphAtlasRect {
            x: 0,
            y: 0,
            width: 8,
            height: 8,
        }
    );
}

fn raster_key(glyph_id: u32) -> GlyphRasterKey {
    GlyphRasterKey {
        face: InstancedFaceId(7),
        glyph_id,
        px_size_bucket: 16,
        subpixel_bin: 0,
        vertical_subpixel_bin: 0,
        format: GlyphAtlasFormat::AlphaMask,
        hinting: GlyphHintingMode::Full,
        smoothing: GlyphSmoothingMode::Grayscale,
        synthetic: SyntheticGlyphStyle::default(),
    }
}
