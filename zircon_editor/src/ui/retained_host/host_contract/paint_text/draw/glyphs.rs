mod metrics;
mod row;

use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::PixelRect;
use super::super::super::paint_theme::{current_host_text_preferences, HostTextSmoothing};
use super::super::font::{HostTextFontFace, HostTextFontSnapshot};
use super::super::raster::{rasterize_cached_glyph, rasterize_cached_runtime_artifact_glyph};
use super::layout::RuntimeTextGlyph;
use super::placement::retained_glyph_placement_for_smoothing;
use metrics::{logical_raster_extent, TEXT_RASTER_SUPERSAMPLE};
use row::draw_glyph_row;

pub(super) fn draw_layout_glyphs(
    frame: &mut HostRgbaFrame,
    clip: &PixelRect,
    font_face: HostTextFontFace,
    glyphs: &[RuntimeTextGlyph],
    artifact_raster_fonts: &[HostTextFontSnapshot],
    color: [u8; 4],
    style: UiTextRunPaintStyle,
) {
    let smoothing = current_host_text_preferences().smoothing;
    for glyph in glyphs {
        draw_layout_glyph(
            frame,
            clip,
            font_face,
            glyph,
            artifact_raster_fonts,
            color,
            style,
            smoothing,
        );
    }
}

fn draw_layout_glyph(
    frame: &mut HostRgbaFrame,
    clip: &PixelRect,
    font_face: HostTextFontFace,
    glyph: &RuntimeTextGlyph,
    artifact_raster_fonts: &[HostTextFontSnapshot],
    color: [u8; 4],
    style: UiTextRunPaintStyle,
    smoothing: HostTextSmoothing,
) {
    let phase_x = if glyph.origin_x.is_finite() {
        glyph.origin_x
    } else {
        glyph.x
    };
    let origin_placement = retained_glyph_placement_for_smoothing(phase_x, smoothing);
    let raster = glyph
        .raster_font_index
        .and_then(|index| artifact_raster_fonts.get(index))
        .map(|font| {
            rasterize_cached_runtime_artifact_glyph(
                font,
                glyph.glyph_index,
                glyph.px,
                TEXT_RASTER_SUPERSAMPLE,
                origin_placement.subpixel_offset,
            )
        })
        .unwrap_or_else(|| {
            rasterize_cached_glyph(
                font_face,
                glyph.glyph_index,
                glyph.px,
                TEXT_RASTER_SUPERSAMPLE,
                origin_placement.subpixel_offset,
            )
        });
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
    let layout_bitmap_left_x = if glyph.x.is_finite() {
        glyph.x
    } else {
        phase_x + metrics.x_offset as f32
    };
    let layout_bitmap_left_placement =
        retained_glyph_placement_for_smoothing(layout_bitmap_left_x, smoothing);
    let glyph_x = retained_glyph_bitmap_pixel_x(
        glyph,
        layout_bitmap_left_placement.pixel_x,
        origin_placement.pixel_x,
        metrics.x_offset,
    );
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

fn retained_glyph_bitmap_pixel_x(
    glyph: &RuntimeTextGlyph,
    layout_bitmap_left_pixel_x: i32,
    origin_pixel_x: i32,
    raster_metrics_x_offset: i32,
) -> i32 {
    if glyph.origin_x.is_finite() {
        origin_pixel_x + raster_metrics_x_offset
    } else if glyph.x.is_finite() {
        layout_bitmap_left_pixel_x
    } else {
        origin_pixel_x + raster_metrics_x_offset
    }
}

#[cfg(test)]
mod tests;
