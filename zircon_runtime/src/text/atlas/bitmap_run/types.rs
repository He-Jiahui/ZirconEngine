//! 位图路径的分配、上传与绘制交接描述。
//! 一次字形出现与可复用光栅内容分开建模，使同一缓存槽能对应多个屏幕位置和颜色。

use std::collections::BTreeSet;

use crate::core::math::UVec2;

use super::super::render_plan::{GlyphAtlasDrawGlyph, GlyphAtlasScreenRect};
use super::super::{
    GlyphAtlasDirtyPage, GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasRect, GlyphAtlasSet,
    GlyphAtlasUploadCommand, GlyphRasterKey,
};
use super::failure::{GlyphAtlasBitmapAllocationFailure, GlyphAtlasBitmapQueuedGlyph};
use super::placeholder::GlyphAtlasBitmapPlaceholderGlyph;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GlyphAtlasBitmapSource {
    /// Stable raster identity; sources without one use frame-local allocation.
    pub(crate) raster_key: Option<GlyphRasterKey>,
    pub(crate) format: GlyphAtlasFormat,
    pub(crate) content_size: UVec2,
    pub(crate) screen_rect: GlyphAtlasScreenRect,
    pub(crate) foreground_color: [f32; 4],
    pub(crate) background_color: [f32; 4],
    pub(crate) source_byte_len: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GlyphAtlasBitmapGlyph {
    pub(crate) source_index: usize,
    pub(crate) page_key: GlyphAtlasPageKey,
    pub(crate) atlas_rect: GlyphAtlasRect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapUploadCopy {
    pub(crate) source_index: usize,
    pub(crate) page_key: GlyphAtlasPageKey,
    pub(crate) atlas_rect: GlyphAtlasRect,
    pub(crate) content_size: UVec2,
    pub(crate) source_bytes_per_row: u32,
    pub(crate) source_byte_len: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapSlotInvalidation {
    pub(crate) page_key: GlyphAtlasPageKey,
    pub(crate) page_generation: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
/// 分配完成后的图集候选快照与后续工作清单；分配成功不等于纹理上传成功。
/// 持有者需在实际提交后维护阴影及失败失效状态，不能仅凭缓存槽命中省略尚未确认的上传。
pub(crate) struct GlyphAtlasBitmapRunPlan {
    pub(crate) atlas: GlyphAtlasSet,
    pub(crate) glyphs: Vec<GlyphAtlasBitmapGlyph>,
    pub(crate) draw_glyphs: Vec<GlyphAtlasDrawGlyph>,
    pub(crate) dirty_pages: Vec<GlyphAtlasDirtyPage>,
    pub(crate) upload_copies: Vec<GlyphAtlasBitmapUploadCopy>,
    pub(crate) upload_commands: Vec<GlyphAtlasUploadCommand>,
    /// Pages which were semantically blank before this run and may acquire a
    /// zero-based CPU shadow only after every associated upload is accepted.
    pub(crate) zero_initialize_shadow_pages: BTreeSet<GlyphAtlasPageKey>,
    pub(crate) rebuilt_pages: Vec<GlyphAtlasPageKey>,
    pub(crate) slot_invalidations: Vec<GlyphAtlasBitmapSlotInvalidation>,
    pub(crate) invalidated_raster_keys: Vec<GlyphRasterKey>,
    pub(crate) allocation_failures: Vec<GlyphAtlasBitmapAllocationFailure>,
    pub(crate) blocked_glyphs: Vec<GlyphAtlasBitmapQueuedGlyph>,
    pub(crate) placeholder_glyphs: Vec<GlyphAtlasBitmapPlaceholderGlyph>,
    pub(crate) slot_cache_hit_count: usize,
    pub(crate) slot_cache_miss_count: usize,
    pub(crate) slot_cache_insert_count: usize,
}
