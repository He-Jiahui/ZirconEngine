mod row;

use fontdue::layout::GlyphPosition;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::PixelRect;
use super::super::raster::rasterize_cached_glyph;
use row::draw_glyph_row;

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
    let raster = rasterize_cached_glyph(glyph.key.glyph_index, glyph.key.px);
    let metrics = &raster.metrics;
    let bitmap = raster.bitmap.as_ref();
    if metrics.width == 0 || metrics.height == 0 {
        return;
    }
    let glyph_x = glyph.x.round() as i32;
    let glyph_y = glyph.y.round() as i32;
    for row in 0..metrics.height {
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
            row,
            glyph_x,
            y,
            color,
            style,
        );
    }
}
