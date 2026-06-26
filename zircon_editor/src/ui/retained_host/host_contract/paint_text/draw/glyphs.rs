mod row;

use fontdue::layout::GlyphPosition;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::PixelRect;
use super::super::raster::rasterize_cached_glyph;
use row::draw_glyph_row;

const TEXT_RASTER_SUPERSAMPLE: f32 = 2.0;

pub(super) fn draw_layout_glyphs(
    frame: &mut HostRgbaFrame,
    clip: &PixelRect,
    glyphs: &[GlyphPosition],
    color: [u8; 4],
    style: UiTextRunPaintStyle,
) {
    for glyph in glyphs {
        draw_layout_glyph(frame, clip, glyph, color, style);
    }
}

fn draw_layout_glyph(
    frame: &mut HostRgbaFrame,
    clip: &PixelRect,
    glyph: &GlyphPosition,
    color: [u8; 4],
    style: UiTextRunPaintStyle,
) {
    let raster = rasterize_cached_glyph(glyph.key.glyph_index, raster_font_size(glyph.key.px));
    let metrics = &raster.metrics;
    let bitmap = raster.bitmap.as_ref();
    if metrics.width == 0 || metrics.height == 0 {
        return;
    }
    let logical_width = logical_raster_extent(metrics.width);
    let logical_height = logical_raster_extent(metrics.height);
    if logical_width == 0 || logical_height == 0 {
        return;
    }
    let glyph_x = glyph.x.round() as i32;
    let glyph_y = glyph.y.round() as i32;
    for row in 0..logical_height {
        let y = glyph_y + row as i32;
        if y < clip.y0 as i32 || y >= clip.y1 as i32 {
            continue;
        }
        draw_glyph_row(
            frame,
            clip,
            bitmap,
            metrics.width,
            metrics.height,
            logical_width,
            logical_height,
            row,
            glyph_x,
            y,
            color,
            style,
            TEXT_RASTER_SUPERSAMPLE,
        );
    }
}

fn raster_font_size(logical_px: f32) -> f32 {
    logical_px * TEXT_RASTER_SUPERSAMPLE
}

fn logical_raster_extent(raster_extent: usize) -> usize {
    (raster_extent as f32 / TEXT_RASTER_SUPERSAMPLE).ceil() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_text_rasterizes_at_supersampled_font_size() {
        assert_eq!(raster_font_size(10.0), 20.0);
    }

    #[test]
    fn logical_raster_extent_downsamples_scaled_bitmap_bounds() {
        assert_eq!(logical_raster_extent(0), 0);
        assert_eq!(logical_raster_extent(1), 1);
        assert_eq!(logical_raster_extent(2), 1);
        assert_eq!(logical_raster_extent(3), 2);
    }
}
