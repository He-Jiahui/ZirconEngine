//! 核对单页坐标、换行与局部失败不提交游标的静态夹具。
//! 不创建 GPU 页面，也不验收外层预约、上传或离线像素拷贝的事务性。

use super::*;
use crate::text::atlas::GlyphAtlasFormat;

/// 连续成功的同一行矩形之间只追加给定间隔。
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

/// 水平不适配时，按上一行最大高度与间隔开始下一行。
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

/// 失败后仍可在原行接续成功，核对分配器局部状态保留。
#[test]
fn render_text_atlas_failed_allocation_preserves_current_shelf() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0);
    let mut allocator = GlyphAtlasShelfAllocator::new(page_key, UVec2::new(64, 64), 2);

    allocator.allocate(UVec2::new(32, 16)).unwrap();
    assert_eq!(allocator.allocate(UVec2::new(40, 60)), None);
    let after_failure = allocator.allocate(UVec2::new(20, 16)).unwrap();

    assert_eq!(after_failure.rect, atlas_rect(34, 0, 20, 16));
}

/// 用整数极值核对纵坐标溢出返回 `None`，无需实际巨型页面分配。
#[test]
fn render_text_atlas_shelf_rejects_vertical_coordinate_overflow() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0);
    let mut allocator = GlyphAtlasShelfAllocator::new(page_key, UVec2::new(4, u32::MAX), 0);

    assert!(allocator.allocate(UVec2::new(4, u32::MAX)).is_some());
    assert_eq!(allocator.allocate(UVec2::new(1, 1)), None);
}

/// 按四个像素标量构造预期矩形，不补间隔或核验页面。
fn atlas_rect(x: u32, y: u32, width: u32, height: u32) -> GlyphAtlasRect {
    GlyphAtlasRect {
        x,
        y,
        width,
        height,
    }
}
