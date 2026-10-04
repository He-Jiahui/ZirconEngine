mod row;

use zircon_runtime::core::framework::text::{
    TextGlyphRasterHinting, TextGlyphRasterMode, TextGlyphRasterRequest, TextGlyphRasterSmoothing,
    TextGlyphSyntheticStyle,
};
use zircon_runtime::ui::surface::UiTextGlyphArtifactRasterFace;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::PixelRect;
use super::super::super::paint_theme::{current_host_text_preferences, HostTextSmoothing};
use super::layout::RuntimeTextGlyph;
use row::draw_glyph_row;

pub(super) fn draw_layout_glyphs(
    frame: &mut HostRgbaFrame,
    clip: &PixelRect,
    glyphs: &[RuntimeTextGlyph],
    artifact_raster_faces: &[UiTextGlyphArtifactRasterFace],
    color: [u8; 4],
    style: UiTextRunPaintStyle,
) {
    let smoothing = current_host_text_preferences().smoothing;
    for glyph in glyphs {
        let Some(face) = artifact_raster_faces.get(glyph.raster_face_index) else {
            super::visual_evidence::failure("glyph raster face is unavailable");
            zircon_runtime::profile_counter!(
                "editor",
                "retained_text_raster_face_lookup_failure_count",
                1
            );
            continue;
        };
        draw_layout_glyph(frame, clip, face, glyph, color, style, smoothing);
    }
}

fn draw_layout_glyph(
    frame: &mut HostRgbaFrame,
    clip: &PixelRect,
    face: &UiTextGlyphArtifactRasterFace,
    glyph: &RuntimeTextGlyph,
    color: [u8; 4],
    style: UiTextRunPaintStyle,
    smoothing: HostTextSmoothing,
) {
    let request = glyph_raster_request(glyph, style, smoothing);
    let receipt = match face.rasterize_glyph(request) {
        Ok(receipt) => receipt,
        Err(error) => {
            super::visual_evidence::failure(&format!("glyph rasterization failed: {error:?}"));
            zircon_runtime::profile_counter!(
                "editor",
                "retained_text_glyph_raster_failure_count",
                1
            );
            return;
        }
    };
    let Ok(raster_width) = usize::try_from(receipt.size[0]) else {
        return;
    };
    let Ok(raster_height) = usize::try_from(receipt.size[1]) else {
        return;
    };
    let glyph_x = bitmap_left(glyph.origin_x, receipt.bearing[0]);
    let glyph_y = bitmap_top(glyph.baseline_y, receipt.bearing[1]);
    super::visual_evidence::glyph(face, glyph, &receipt, [glyph_x, glyph_y], clip, color[3]);
    if raster_width == 0 || raster_height == 0 {
        return;
    }
    for row in 0..raster_height {
        let y = glyph_y + row as i32;
        if y < clip.y0 as i32 || y >= clip.y1 as i32 {
            continue;
        }
        draw_glyph_row(
            frame,
            clip,
            receipt.bitmap.as_ref(),
            raster_width,
            raster_height,
            receipt.format,
            row,
            glyph_x,
            y,
            color,
        );
    }
}

fn glyph_raster_request(
    glyph: &RuntimeTextGlyph,
    style: UiTextRunPaintStyle,
    smoothing: HostTextSmoothing,
) -> TextGlyphRasterRequest {
    TextGlyphRasterRequest::new(
        glyph.glyph_id,
        glyph.physical_ppem,
        TextGlyphRasterMode::ColorPreferred,
    )
    .with_subpixel_position(glyph.origin_x, glyph.baseline_y)
    .with_hinting(TextGlyphRasterHinting::Full)
    .with_smoothing(match smoothing {
        HostTextSmoothing::Grayscale => TextGlyphRasterSmoothing::Grayscale,
        HostTextSmoothing::Subpixel => TextGlyphRasterSmoothing::Subpixel,
    })
    .with_synthetic_style(TextGlyphSyntheticStyle {
        oblique: style.emphasis,
    })
}

fn bitmap_left(origin_x: f32, bearing_left: f32) -> i32 {
    origin_x.floor() as i32 + bearing_left.round() as i32
}

fn bitmap_top(baseline_y: f32, bearing_top: f32) -> i32 {
    baseline_y.floor() as i32 - bearing_top.round() as i32
}

#[cfg(test)]
#[path = "glyphs/tests/cases.rs"]
mod tests;
