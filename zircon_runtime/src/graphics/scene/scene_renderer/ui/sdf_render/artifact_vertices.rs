use crate::core::framework::text::{TextGlyph, TextGlyphRotation};
use crate::text::ShapedGlyphRotation;
use crate::text::atlas::{GlyphAtlasFormat, GlyphRasterPlacement, GlyphSmoothingMode};
use zircon_runtime_interface::ui::layout::UiFrame;

use super::super::render::ScreenSpaceUiTextBatch;
use super::super::sdf_atlas::SdfAtlasPlan;
use super::super::text_pixel_snap::{text_frame_device_origin, text_glyph_device_frame};
use super::vertices::{
    RunGlyph, ScreenSpaceUiSdfVertex, aligned_text_start_x, atlas_uv_rect,
    horizontal_sdf_text_baseline, push_clipped_glyph_quad, vertical_sdf_glyph_frame,
};

pub(super) fn push_horizontal_artifact_sdf_text_vertices(
    vertices: &mut Vec<ScreenSpaceUiSdfVertex>,
    text: &ScreenSpaceUiTextBatch,
    glyphs: Vec<RunGlyph>,
    plan: &SdfAtlasPlan,
    clip: UiFrame,
    viewport: UiFrame,
) {
    let Some(artifact_glyphs) = text
        .glyph_artifact_line
        .as_ref()
        .and_then(|line| line.glyphs())
    else {
        return;
    };
    if artifact_glyphs.len() != glyphs.len() {
        return;
    }
    let positioned_frame = text_frame_device_origin(text.frame);
    let baseline = horizontal_artifact_baseline(text, positioned_frame, &glyphs);
    let text_width = artifact_glyphs
        .iter()
        .map(|glyph| glyph.advance.max(0.0))
        .sum();
    let mut cursor_x = aligned_text_start_x(text, text_width);

    for (artifact_glyph, glyph) in artifact_glyphs.iter().zip(glyphs) {
        let advance = artifact_glyph.advance.max(0.0);
        let Some(slot_index) = glyph.slot_index else {
            cursor_x += advance;
            continue;
        };
        let Some(slot) = plan.slots.get(slot_index) else {
            cursor_x += advance;
            continue;
        };
        if !glyph.visible || glyph.metrics.bitmap_width == 0 || glyph.metrics.bitmap_height == 0 {
            cursor_x += advance;
            continue;
        }
        let frame = horizontal_artifact_sdf_glyph_frame(cursor_x, baseline, &glyph, artifact_glyph);
        push_clipped_glyph_quad(
            vertices,
            frame,
            clip,
            viewport,
            atlas_uv_rect(slot.rect, plan.atlas_size, &glyph),
            text.color,
            glyph.screen_px_range,
            glyph.atlas_px_range,
            slot.page_key.page_index,
            slot.key.bake_params.mode,
            artifact_glyph_rotation(artifact_glyph),
        );
        cursor_x += advance;
    }
}

fn horizontal_artifact_baseline(
    text: &ScreenSpaceUiTextBatch,
    positioned_frame: UiFrame,
    glyphs: &[RunGlyph],
) -> f32 {
    if let Some(baseline) = text
        .glyph_artifact_line
        .as_ref()
        .and_then(|line| line.layout_baseline())
    {
        // The resolved line owns the baseline; raster metrics only describe the bitmap bearing.
        return positioned_frame.y + baseline;
    }
    horizontal_sdf_text_baseline(text, positioned_frame, glyphs)
}

pub(super) fn push_vertical_artifact_sdf_text_vertices(
    vertices: &mut Vec<ScreenSpaceUiSdfVertex>,
    text: &ScreenSpaceUiTextBatch,
    glyphs: Vec<RunGlyph>,
    plan: &SdfAtlasPlan,
    clip: UiFrame,
    viewport: UiFrame,
) {
    let Some(artifact_glyphs) = text
        .glyph_artifact_line
        .as_ref()
        .and_then(|line| line.glyphs())
    else {
        return;
    };
    if artifact_glyphs.len() != glyphs.len() {
        return;
    }
    let mut cursor_y = text_frame_device_origin(text.frame).y;
    for (artifact_glyph, glyph) in artifact_glyphs.iter().zip(glyphs) {
        let advance = artifact_glyph.advance.max(0.0);
        let Some(slot_index) = glyph.slot_index else {
            cursor_y += advance;
            continue;
        };
        let Some(slot) = plan.slots.get(slot_index) else {
            cursor_y += advance;
            continue;
        };
        if !glyph.visible || glyph.metrics.bitmap_width == 0 || glyph.metrics.bitmap_height == 0 {
            cursor_y += advance;
            continue;
        }
        let frame =
            vertical_artifact_sdf_glyph_frame(text, &glyph, cursor_y, advance, artifact_glyph);
        push_clipped_glyph_quad(
            vertices,
            frame,
            clip,
            viewport,
            atlas_uv_rect(slot.rect, plan.atlas_size, &glyph),
            text.color,
            glyph.screen_px_range,
            glyph.atlas_px_range,
            slot.page_key.page_index,
            slot.key.bake_params.mode,
            artifact_glyph_rotation(artifact_glyph),
        );
        cursor_y += advance;
    }
}

fn horizontal_artifact_sdf_glyph_frame(
    cursor_x: f32,
    baseline: f32,
    glyph: &RunGlyph,
    artifact_glyph: &TextGlyph,
) -> UiFrame {
    let requested_x = cursor_x + artifact_glyph.offset[0] + glyph.metrics.bitmap_left;
    let placement = GlyphRasterPlacement::from_raster_input(
        GlyphAtlasFormat::Sdf,
        GlyphSmoothingMode::None,
        false,
        requested_x,
    );
    text_glyph_device_frame(UiFrame::new(
        placement.snapped_x,
        baseline + artifact_glyph.offset[1]
            - (glyph.metrics.bitmap_bottom + glyph.metrics.bitmap_height as f32),
        glyph.metrics.bitmap_width as f32,
        glyph.metrics.bitmap_height as f32,
    ))
}

fn vertical_artifact_sdf_glyph_frame(
    text: &ScreenSpaceUiTextBatch,
    glyph: &RunGlyph,
    cursor_y: f32,
    advance: f32,
    artifact_glyph: &TextGlyph,
) -> UiFrame {
    let rotation = artifact_glyph_rotation(artifact_glyph);
    let has_vertical_origin = matches!(rotation, ShapedGlyphRotation::None)
        && (artifact_glyph.offset[0].abs() > f32::EPSILON
            || artifact_glyph.offset[1].abs() > f32::EPSILON);
    if !has_vertical_origin {
        return vertical_sdf_glyph_frame(text, glyph, cursor_y, advance, rotation);
    }

    let positioned_frame = text_frame_device_origin(text.frame);
    text_glyph_device_frame(UiFrame::new(
        positioned_frame.x
            + positioned_frame.width * 0.5
            + artifact_glyph.offset[0]
            + glyph.metrics.bitmap_left,
        cursor_y + artifact_glyph.offset[1]
            - (glyph.metrics.bitmap_bottom + glyph.metrics.bitmap_height as f32),
        glyph.metrics.bitmap_width as f32,
        glyph.metrics.bitmap_height as f32,
    ))
}

fn artifact_glyph_rotation(glyph: &TextGlyph) -> ShapedGlyphRotation {
    match glyph.rotation {
        TextGlyphRotation::None => ShapedGlyphRotation::None,
        TextGlyphRotation::Clockwise90 => ShapedGlyphRotation::Cw90,
    }
}
