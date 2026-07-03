mod row;

use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::PixelRect;
use super::super::font::HostTextFontFace;
use super::super::raster::rasterize_cached_glyph;
use super::layout::RuntimeTextGlyph;
use super::placement::RetainedGlyphPlacement;
use row::draw_glyph_row;

const TEXT_RASTER_SUPERSAMPLE: f32 = 8.0;

pub(super) fn draw_layout_glyphs(
    frame: &mut HostRgbaFrame,
    clip: &PixelRect,
    font_face: HostTextFontFace,
    glyphs: &[RuntimeTextGlyph],
    color: [u8; 4],
    style: UiTextRunPaintStyle,
) {
    for glyph in glyphs {
        draw_layout_glyph(frame, clip, font_face, glyph, color, style);
    }
}

fn draw_layout_glyph(
    frame: &mut HostRgbaFrame,
    clip: &PixelRect,
    font_face: HostTextFontFace,
    glyph: &RuntimeTextGlyph,
    color: [u8; 4],
    style: UiTextRunPaintStyle,
) {
    let phase_x = if glyph.origin_x.is_finite() {
        glyph.origin_x
    } else {
        glyph.x
    };
    let placement = RetainedGlyphPlacement::from_screen_x(phase_x);
    let raster = rasterize_cached_glyph(
        font_face,
        glyph.glyph_index,
        glyph.px,
        TEXT_RASTER_SUPERSAMPLE,
        placement.subpixel_offset,
    );
    let metrics = &raster.metrics;
    let bitmap = raster.bitmap.as_ref();
    if metrics.width == 0 || metrics.height == 0 {
        return;
    }
    let logical_width =
        logical_raster_extent(metrics.width, raster.raster_scale, raster.sample_offset_x);
    let logical_height = logical_raster_extent(metrics.height, raster.raster_scale, 0.0);
    if logical_width == 0 || logical_height == 0 {
        return;
    }
    let glyph_x = placement.pixel_x + metrics.x_offset;
    let glyph_y = glyph.y.round() as i32 + metrics.y_offset;
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
            raster.format,
            logical_width,
            logical_height,
            row,
            glyph_x,
            y,
            color,
            style,
            raster.raster_scale,
            raster.sample_offset_x,
        );
    }
}

fn logical_raster_extent(raster_extent: usize, raster_scale: f32, sample_offset: f32) -> usize {
    let raster_scale = if raster_scale.is_finite() && raster_scale > 1.0 {
        raster_scale
    } else {
        1.0
    };
    let sample_offset = if sample_offset.is_finite() {
        sample_offset.clamp(0.0, 0.999)
    } else {
        0.0
    };
    (raster_extent as f32 / raster_scale + sample_offset).ceil() as usize
}

#[cfg(test)]
mod tests;
