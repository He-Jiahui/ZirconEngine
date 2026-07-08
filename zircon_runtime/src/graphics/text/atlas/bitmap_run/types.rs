use crate::core::math::UVec2;

use super::super::render_plan::{GlyphAtlasDrawGlyph, GlyphAtlasScreenRect};
use super::super::{
    GlyphAtlasDirtyPage, GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasRect, GlyphAtlasSet,
    GlyphAtlasUploadCommand,
};
use super::failure::{GlyphAtlasBitmapAllocationFailure, GlyphAtlasBitmapQueuedGlyph};
use super::placeholder::GlyphAtlasBitmapPlaceholderGlyph;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GlyphAtlasBitmapSource {
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
    pub(crate) draw_glyph: GlyphAtlasDrawGlyph,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapUploadCopy {
    pub(crate) source_index: usize,
    pub(crate) page_key: GlyphAtlasPageKey,
    pub(crate) atlas_rect: GlyphAtlasRect,
    pub(crate) content_size: UVec2,
    pub(crate) source_bytes_per_row: u32,
    pub(crate) source_byte_len: usize,
    pub(crate) atlas_bytes_per_row: u32,
    pub(crate) atlas_byte_offset: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapSlotInvalidation {
    pub(crate) page_key: GlyphAtlasPageKey,
    pub(crate) page_generation: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct GlyphAtlasBitmapRunPlan {
    pub(crate) atlas: GlyphAtlasSet,
    pub(crate) glyphs: Vec<GlyphAtlasBitmapGlyph>,
    pub(crate) draw_glyphs: Vec<GlyphAtlasDrawGlyph>,
    pub(crate) dirty_pages: Vec<GlyphAtlasDirtyPage>,
    pub(crate) upload_copies: Vec<GlyphAtlasBitmapUploadCopy>,
    pub(crate) upload_commands: Vec<GlyphAtlasUploadCommand>,
    pub(crate) rebuilt_pages: Vec<GlyphAtlasPageKey>,
    pub(crate) slot_invalidations: Vec<GlyphAtlasBitmapSlotInvalidation>,
    pub(crate) allocation_failures: Vec<GlyphAtlasBitmapAllocationFailure>,
    pub(crate) blocked_glyphs: Vec<GlyphAtlasBitmapQueuedGlyph>,
    pub(crate) placeholder_glyphs: Vec<GlyphAtlasBitmapPlaceholderGlyph>,
}
