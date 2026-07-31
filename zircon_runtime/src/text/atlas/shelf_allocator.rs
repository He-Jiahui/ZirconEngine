use crate::core::math::UVec2;

use super::{GlyphAtlasPageKey, GlyphAtlasRect};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasAllocation {
    pub(crate) page_key: GlyphAtlasPageKey,
    pub(crate) rect: GlyphAtlasRect,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasShelfAllocator {
    page_key: GlyphAtlasPageKey,
    page_size: UVec2,
    padding_px: u32,
    cursor_x: u32,
    cursor_y: u32,
    shelf_height: u32,
}

impl GlyphAtlasShelfAllocator {
    pub(crate) fn new(page_key: GlyphAtlasPageKey, page_size: UVec2, padding_px: u32) -> Self {
        Self {
            page_key,
            page_size,
            padding_px,
            cursor_x: 0,
            cursor_y: 0,
            shelf_height: 0,
        }
    }

    pub(crate) fn allocate(&mut self, size: UVec2) -> Option<GlyphAtlasAllocation> {
        if size.x == 0 || size.y == 0 || size.x > self.page_size.x || size.y > self.page_size.y {
            return None;
        }

        let mut cursor_x = self.cursor_x;
        let mut cursor_y = self.cursor_y;
        let mut shelf_height = self.shelf_height;
        if cursor_x > 0 && cursor_x.saturating_add(size.x) > self.page_size.x {
            cursor_x = 0;
            cursor_y = cursor_y
                .saturating_add(self.shelf_height)
                .saturating_add(self.padding_px);
            shelf_height = 0;
        }

        if cursor_y.saturating_add(size.y) > self.page_size.y {
            return None;
        }

        let rect = GlyphAtlasRect {
            x: cursor_x,
            y: cursor_y,
            width: size.x,
            height: size.y,
        };
        self.cursor_x = cursor_x
            .saturating_add(size.x)
            .saturating_add(self.padding_px);
        self.cursor_y = cursor_y;
        self.shelf_height = shelf_height.max(size.y);

        Some(GlyphAtlasAllocation {
            page_key: self.page_key,
            rect,
        })
    }

    pub(super) fn matches_configuration(
        &self,
        page_key: GlyphAtlasPageKey,
        page_size: UVec2,
        padding_px: u32,
    ) -> bool {
        self.page_key == page_key && self.page_size == page_size && self.padding_px == padding_px
    }
}

#[cfg(test)]
mod tests {
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

    fn atlas_rect(x: u32, y: u32, width: u32, height: u32) -> GlyphAtlasRect {
        GlyphAtlasRect {
            x,
            y,
            width,
            height,
        }
    }
}
