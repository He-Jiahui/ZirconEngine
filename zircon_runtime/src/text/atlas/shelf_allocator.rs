//! 为单页内容分配有间隔的矩形，供持久槽缓存复用同一页面的空间。
//! 不单独回收矩形；页重建时整体重置，失败分配不得推进游标以免挤掉后续可容纳字形。

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
        if cursor_x > 0
            && cursor_x
                .checked_add(size.x)
                .is_none_or(|right| right > self.page_size.x)
        {
            cursor_x = 0;
            cursor_y = cursor_y
                .checked_add(self.shelf_height)
                .and_then(|next_y| next_y.checked_add(self.padding_px))?;
            shelf_height = 0;
        }

        if cursor_y
            .checked_add(size.y)
            .is_none_or(|bottom| bottom > self.page_size.y)
        {
            return None;
        }

        let rect = GlyphAtlasRect {
            x: cursor_x,
            y: cursor_y,
            width: size.x,
            height: size.y,
        };
        self.cursor_x = cursor_x
            .checked_add(size.x)
            .and_then(|next_x| next_x.checked_add(self.padding_px))
            .unwrap_or(u32::MAX);
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
#[path = "tests/shelf_allocator.rs"]
mod tests;
