use crate::text::atlas::GlyphAtlasBitmapSource;
use crate::text::atlas::render_plan::GlyphAtlasScreenRect;

use super::bitmap::GlyphBitmap;

pub(crate) fn glyph_atlas_bitmap_source_from_glyph_bitmap(
    bitmap: &GlyphBitmap,
    screen_rect: GlyphAtlasScreenRect,
    foreground_color: [f32; 4],
    background_color: [f32; 4],
) -> GlyphAtlasBitmapSource {
    GlyphAtlasBitmapSource {
        raster_key: None,
        format: bitmap.required_atlas_format(),
        content_size: bitmap.size,
        screen_rect,
        foreground_color,
        background_color,
        source_byte_len: bitmap.data.len(),
    }
}
