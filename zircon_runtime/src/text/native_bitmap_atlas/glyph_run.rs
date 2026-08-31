use crate::text::atlas::{GlyphRasterKey, render_plan::GlyphAtlasScreenRect};

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
mod tests {
    use crate::text::InstancedFaceId;
    use crate::text::atlas::{
        GlyphAtlasFormat, GlyphHintingMode, GlyphRasterKey, GlyphSmoothingMode,
        SyntheticGlyphStyle, render_plan::GlyphAtlasScreenRect,
    };

    use super::{NativeBitmapAtlasGlyph, NativeBitmapAtlasGlyphRun};

    #[test]
    fn native_bitmap_glyph_run_keeps_only_prepared_glyph_data() {
        let glyph = NativeBitmapAtlasGlyph {
            raster_key: GlyphRasterKey {
                face: InstancedFaceId(12),
                glyph_id: 46,
                px_size_bucket: 18,
                subpixel_bin: 0,
                vertical_subpixel_bin: 2,
                format: GlyphAtlasFormat::AlphaMask,
                hinting: GlyphHintingMode::Full,
                smoothing: GlyphSmoothingMode::Grayscale,
                synthetic: SyntheticGlyphStyle::default(),
            },
            screen_x: 24.25,
            baseline_y: 42.5,
            placeholder_rect: GlyphAtlasScreenRect::new(24.0, 24.0, 12.0, 20.0),
            foreground_color: [1.0; 4],
            background_color: None,
        };
        let run = NativeBitmapAtlasGlyphRun::new(
            GlyphAtlasScreenRect::new(0.0, 0.0, 128.0, 64.0),
            vec![glyph],
        );

        assert_eq!(run.glyphs[0].raster_key.face, InstancedFaceId(12));
        assert_eq!(run.glyphs[0].raster_key.glyph_id, 46);
        assert_eq!(run.glyphs[0].baseline_y, 42.5);
    }
}
