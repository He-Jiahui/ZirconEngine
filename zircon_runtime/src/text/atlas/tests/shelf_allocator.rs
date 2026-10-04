use super::*;
use crate::text::atlas::GlyphAtlasFormat;

#[test]
fn render_text_atlas_shelf_allocates_same_height_into_one_row() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0);
    let mut allocator = GlyphAtlasShelfAllocator::new(page_key, UVec2::new(256, 64), 2);

    let first = allocator.allocate(UVec2::new(32, 16)).unwrap();
    let second = allocator.allocate(UVec2::new(32, 16)).unwrap();
    let third = allocator.allocate(UVec2::new(32, 16)).unwrap();

    assert_eq!(first.page_key, page_key);
    assert_eq!(first.rect, atlas_rect(0, 0, 32, 16));
    assert_eq!(second.rect, atlas_rect(34, 0, 32, 16));
    assert_eq!(third.rect, atlas_rect(68, 0, 32, 16));
}

#[test]
fn render_text_atlas_shelf_starts_new_row_before_page_overflow() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0);
    let mut allocator = GlyphAtlasShelfAllocator::new(page_key, UVec2::new(64, 64), 2);

    let first = allocator.allocate(UVec2::new(32, 16)).unwrap();
    let second = allocator.allocate(UVec2::new(32, 24)).unwrap();
    let third = allocator.allocate(UVec2::new(32, 16)).unwrap();

    assert_eq!(first.rect, atlas_rect(0, 0, 32, 16));
    assert_eq!(second.rect, atlas_rect(0, 18, 32, 24));
    assert_eq!(third.rect, atlas_rect(0, 44, 32, 16));
}

#[test]
fn render_text_atlas_failed_allocation_preserves_current_shelf() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0);
    let mut allocator = GlyphAtlasShelfAllocator::new(page_key, UVec2::new(64, 64), 2);

    allocator.allocate(UVec2::new(32, 16)).unwrap();
    assert_eq!(allocator.allocate(UVec2::new(40, 60)), None);
    let after_failure = allocator.allocate(UVec2::new(20, 16)).unwrap();

    assert_eq!(after_failure.rect, atlas_rect(34, 0, 20, 16));
}

#[test]
fn render_text_atlas_shelf_rejects_vertical_coordinate_overflow() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0);
    let mut allocator = GlyphAtlasShelfAllocator::new(page_key, UVec2::new(4, u32::MAX), 0);

    assert!(allocator.allocate(UVec2::new(4, u32::MAX)).is_some());
    assert_eq!(allocator.allocate(UVec2::new(1, 1)), None);
}

fn atlas_rect(x: u32, y: u32, width: u32, height: u32) -> GlyphAtlasRect {
    GlyphAtlasRect {
        x,
        y,
        width,
        height,
    }
}
