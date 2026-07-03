use super::super::render_plan::GlyphAtlasScreenRect;
use super::super::GlyphAtlasFormat;
use super::types::GlyphAtlasBitmapSource;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphAtlasBitmapPlaceholderMode {
    TransparentQuad,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GlyphAtlasBitmapPlaceholderGlyph {
    pub(crate) source_index: usize,
    pub(crate) format: GlyphAtlasFormat,
    pub(crate) screen_rect: GlyphAtlasScreenRect,
    pub(crate) retry_frame_index: u64,
    pub(crate) mode: GlyphAtlasBitmapPlaceholderMode,
}

pub(super) fn bitmap_placeholder_glyph(
    source_index: usize,
    source: GlyphAtlasBitmapSource,
    retry_frame_index: u64,
) -> GlyphAtlasBitmapPlaceholderGlyph {
    GlyphAtlasBitmapPlaceholderGlyph {
        source_index,
        format: source.format,
        screen_rect: source.screen_rect,
        retry_frame_index,
        mode: GlyphAtlasBitmapPlaceholderMode::TransparentQuad,
    }
}
