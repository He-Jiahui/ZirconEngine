use super::*;
use crate::text::InstancedFaceId;
use crate::text::atlas::{
    GlyphAtlasFormat, GlyphHintingMode, GlyphSmoothingMode, SyntheticGlyphStyle,
};

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

#[test]
fn render_text_atlas_slot_cache_tracks_slots_by_page_for_targeted_invalidation() {
    let source = include_str!("../slot_cache.rs");
    let reverse_index = concat!("page_", "slots");

    assert!(
        source.contains(reverse_index),
        "page invalidation requires a reverse slot index"
    );
}

#[test]
fn render_text_atlas_slot_cache_keeps_replaced_slot_in_its_new_page_index() {
    let first_page = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 1);
    let second_page = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 2);
    let moved_key = raster_key(18);
    let first_page_key = raster_key(19);
    let mut cache = GlyphAtlasSlotCache::default();

    cache.insert_slot(moved_key, persistent_slot(first_page));
    cache.insert_slot(moved_key, persistent_slot(second_page));
    cache.insert_slot(first_page_key, persistent_slot(first_page));

    let invalidated_first_page = cache.invalidate_page(first_page);
    assert_eq!(invalidated_first_page, vec![first_page_key]);
    assert_eq!(
        cache.slot(moved_key).map(|slot| slot.page_key),
        Some(second_page)
    );

    assert_eq!(cache.invalidate_page(second_page), vec![moved_key]);
    assert!(cache.slot_count() == 0);
}

fn persistent_slot(page_key: GlyphAtlasPageKey) -> GlyphAtlasPersistentSlot {
    GlyphAtlasPersistentSlot {
        page_key,
        page_generation: 3,
        inserted_frame_index: 9,
        rect: GlyphAtlasRect {
            x: 0,
            y: 0,
            width: 8,
            height: 8,
        },
        content_size: UVec2::new(8, 8),
    }
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
