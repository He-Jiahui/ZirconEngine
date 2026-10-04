use crate::text::atlas::{
    render_plan::GlyphAtlasScreenRect, GlyphAtlasFormat, GlyphHintingMode, GlyphRasterKey,
    GlyphSmoothingMode, SyntheticGlyphStyle,
};
use crate::text::InstancedFaceId;

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
