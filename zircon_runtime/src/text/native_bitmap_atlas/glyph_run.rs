use crate::text::atlas::{render_plan::GlyphAtlasScreenRect, GlyphRasterKey};

/// Renderer-facing input for a native bitmap glyph.
///
/// Text shaping owns glyph selection, font-instance selection, advances, and offsets. The native
/// atlas only receives their final screen-space projection plus a stable raster identity; it never
/// observes a source string or a shaping buffer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NativeBitmapAtlasGlyph {
    pub(crate) raster_key: GlyphRasterKey,
    pub(crate) screen_x: f32,
    pub(crate) baseline_y: f32,
    pub(crate) placeholder_rect: GlyphAtlasScreenRect,
    pub(crate) foreground_color: [f32; 4],
    pub(crate) background_color: Option<[f32; 4]>,
}

/// A draw-order preserving native bitmap glyph run with one clipping region.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativeBitmapAtlasGlyphRun {
    pub(crate) bounds: GlyphAtlasScreenRect,
    pub(crate) glyphs: Vec<NativeBitmapAtlasGlyph>,
}

impl NativeBitmapAtlasGlyphRun {
    pub(crate) fn new(bounds: GlyphAtlasScreenRect, glyphs: Vec<NativeBitmapAtlasGlyph>) -> Self {
        Self { bounds, glyphs }
    }
}

#[cfg(test)]
#[path = "tests/glyph_run.rs"]
mod tests;
