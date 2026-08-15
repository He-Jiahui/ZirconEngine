use crate::text::atlas::{GlyphAtlasFormat, GlyphRasterPlacement, GlyphSmoothingMode};
use crate::text::shaping::{vertical_glyph_advance, vertical_glyph_rotation};
use crate::text::ShapedGlyphRotation;
use crate::text::VerticalMode;
use zircon_runtime_interface::ui::layout::UiFrame;

use super::super::super::render::{ScreenSpaceUiShapedGlyph, ScreenSpaceUiTextBatch};
use super::super::super::sdf_atlas::SdfAtlasPlan;
use super::super::super::text_pixel_snap::{text_frame_device_origin, text_glyph_device_frame};
use super::super::artifact_vertices::{
    push_horizontal_artifact_sdf_text_vertices, push_vertical_artifact_sdf_text_vertices,
};
use super::super::shaped_advances::resolved_horizontal_shaped_glyph_advances;
use super::{
    aligned_text_start_x, atlas_uv_rect, horizontal_sdf_text_baseline, push_clipped_glyph_quad,
    resolve_sdf_glyph_advances, resolve_vertical_sdf_glyph_advances, RunGlyph,
    ScreenSpaceUiSdfVertex,
};

pub(super) fn push_horizontal_sdf_text_vertices(
    vertices: &mut Vec<ScreenSpaceUiSdfVertex>,
    text: &ScreenSpaceUiTextBatch,
    glyphs: Vec<RunGlyph>,
    plan: &SdfAtlasPlan,
    clip: UiFrame,
    viewport: UiFrame,
) {
    if text.glyph_artifact_line.is_some() {
        push_horizontal_artifact_sdf_text_vertices(vertices, text, glyphs, plan, clip, viewport);
        return;
    }
    if text.shaped_glyphs.len() == glyphs.len() && !text.shaped_glyphs.is_empty() {
        push_horizontal_shaped_sdf_text_vertices(vertices, text, glyphs, plan, clip, viewport);
        return;
    }

    let natural_advances = glyphs
        .iter()
        .map(|glyph| glyph.metrics.advance)
        .collect::<Vec<_>>();
    let natural_text_width = natural_advances.iter().sum();
    let glyph_advances = resolve_sdf_glyph_advances(text, natural_advances, natural_text_width);
    let text_width = glyph_advances.iter().sum();
    let positioned_frame = text_frame_device_origin(text.frame);
    let baseline = horizontal_sdf_text_baseline(text, positioned_frame, &glyphs);
    let mut cursor_x = aligned_text_start_x(text, text_width);

    for (glyph, advance) in glyphs.into_iter().zip(glyph_advances) {
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
        let frame = horizontal_sdf_glyph_frame(cursor_x, baseline, &glyph);
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
            ShapedGlyphRotation::None,
        );
        cursor_x += advance;
    }
}

fn push_horizontal_shaped_sdf_text_vertices(
    vertices: &mut Vec<ScreenSpaceUiSdfVertex>,
    text: &ScreenSpaceUiTextBatch,
    glyphs: Vec<RunGlyph>,
    plan: &SdfAtlasPlan,
    clip: UiFrame,
    viewport: UiFrame,
) {
    let positioned_frame = text_frame_device_origin(text.frame);
    let baseline = horizontal_sdf_text_baseline(text, positioned_frame, &glyphs);
    let glyph_advances = resolved_horizontal_shaped_glyph_advances(text);
    let text_width = glyph_advances.iter().sum();
    let mut cursor_x = aligned_text_start_x(text, text_width);

    for ((shaped, glyph), advance) in text.shaped_glyphs.iter().zip(glyphs).zip(glyph_advances) {
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
        let frame = horizontal_shaped_sdf_glyph_frame(cursor_x, baseline, &glyph, shaped);
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
            shaped.rotation,
        );
        cursor_x += advance;
    }
}

pub(in super::super) fn horizontal_sdf_glyph_frame(
    cursor_x: f32,
    baseline: f32,
    glyph: &RunGlyph,
) -> UiFrame {
    let requested_x = cursor_x + glyph.metrics.bitmap_left;
    let placement = GlyphRasterPlacement::from_raster_input(
        GlyphAtlasFormat::Sdf,
        GlyphSmoothingMode::None,
        false,
        requested_x,
    );
    text_glyph_device_frame(UiFrame::new(
        placement.snapped_x,
        baseline - (glyph.metrics.bitmap_bottom + glyph.metrics.bitmap_height as f32),
        glyph.metrics.bitmap_width as f32,
        glyph.metrics.bitmap_height as f32,
    ))
}

pub(in super::super) fn horizontal_shaped_sdf_glyph_frame(
    cursor_x: f32,
    baseline: f32,
    glyph: &RunGlyph,
    shaped: &ScreenSpaceUiShapedGlyph,
) -> UiFrame {
    let requested_x = cursor_x + shaped.offset_x + glyph.metrics.bitmap_left;
    let placement = GlyphRasterPlacement::from_raster_input(
        GlyphAtlasFormat::Sdf,
        GlyphSmoothingMode::None,
        false,
        requested_x,
    );
    text_glyph_device_frame(UiFrame::new(
        placement.snapped_x,
        baseline + shaped.offset_y
            - (glyph.metrics.bitmap_bottom + glyph.metrics.bitmap_height as f32),
        glyph.metrics.bitmap_width as f32,
        glyph.metrics.bitmap_height as f32,
    ))
}

pub(super) fn push_vertical_sdf_text_vertices(
    vertices: &mut Vec<ScreenSpaceUiSdfVertex>,
    text: &ScreenSpaceUiTextBatch,
    glyphs: Vec<RunGlyph>,
    plan: &SdfAtlasPlan,
    clip: UiFrame,
    viewport: UiFrame,
) {
    if text.glyph_artifact_line.is_some() {
        push_vertical_artifact_sdf_text_vertices(vertices, text, glyphs, plan, clip, viewport);
        return;
    }
    if text.shaped_glyphs.len() == glyphs.len() && !text.shaped_glyphs.is_empty() {
        push_vertical_shaped_sdf_text_vertices(vertices, text, glyphs, plan, clip, viewport);
        return;
    }

    let natural_advances = text
        .text
        .chars()
        .zip(glyphs.iter())
        .map(|(character, glyph)| {
            let mut cluster_bytes = [0_u8; 4];
            vertical_glyph_advance(
                VerticalMode::Mixed,
                character.encode_utf8(&mut cluster_bytes),
                glyph.metrics.advance,
                text.font_size,
            )
        })
        .collect::<Vec<_>>();
    let glyph_advances = resolve_vertical_sdf_glyph_advances(text, natural_advances);
    let mut cursor_y = text_frame_device_origin(text.frame).y;
    for ((character, glyph), advance) in text.text.chars().zip(glyphs).zip(glyph_advances) {
        let advance = advance.max(0.0);
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
        let mut cluster_bytes = [0_u8; 4];
        let cluster_text = character.encode_utf8(&mut cluster_bytes);
        let rotation = vertical_glyph_rotation(VerticalMode::Mixed, cluster_text);
        let frame = vertical_sdf_glyph_frame(text, &glyph, cursor_y, advance, rotation);
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
            rotation,
        );
        cursor_y += advance;
    }
}

fn push_vertical_shaped_sdf_text_vertices(
    vertices: &mut Vec<ScreenSpaceUiSdfVertex>,
    text: &ScreenSpaceUiTextBatch,
    glyphs: Vec<RunGlyph>,
    plan: &SdfAtlasPlan,
    clip: UiFrame,
    viewport: UiFrame,
) {
    let mut cursor_y = text_frame_device_origin(text.frame).y;
    for (shaped, glyph) in text.shaped_glyphs.iter().zip(glyphs) {
        let advance = shaped.advance.max(0.0);
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
        let frame = vertical_shaped_sdf_glyph_frame(text, &glyph, cursor_y, advance, shaped);
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
            shaped.rotation,
        );
        cursor_y += advance;
    }
}

pub(in super::super) fn vertical_shaped_sdf_glyph_frame(
    text: &ScreenSpaceUiTextBatch,
    glyph: &RunGlyph,
    cursor_y: f32,
    advance: f32,
    shaped: &ScreenSpaceUiShapedGlyph,
) -> UiFrame {
    let has_vertical_origin = matches!(shaped.rotation, ShapedGlyphRotation::None)
        && (shaped.offset_x.abs() > f32::EPSILON || shaped.offset_y.abs() > f32::EPSILON);
    if !has_vertical_origin {
        return vertical_sdf_glyph_frame(text, glyph, cursor_y, advance, shaped.rotation);
    }

    let positioned_frame = text_frame_device_origin(text.frame);
    text_glyph_device_frame(UiFrame::new(
        positioned_frame.x
            + positioned_frame.width * 0.5
            + shaped.offset_x
            + glyph.metrics.bitmap_left,
        cursor_y + shaped.offset_y
            - (glyph.metrics.bitmap_bottom + glyph.metrics.bitmap_height as f32),
        glyph.metrics.bitmap_width as f32,
        glyph.metrics.bitmap_height as f32,
    ))
}

pub(in super::super) fn vertical_sdf_glyph_frame(
    text: &ScreenSpaceUiTextBatch,
    glyph: &RunGlyph,
    cursor_y: f32,
    advance: f32,
    rotation: ShapedGlyphRotation,
) -> UiFrame {
    let positioned_frame = text_frame_device_origin(text.frame);
    let bitmap_width = glyph.metrics.bitmap_width as f32;
    let bitmap_height = glyph.metrics.bitmap_height as f32;
    let (width, height) = match rotation {
        ShapedGlyphRotation::None => (bitmap_width, bitmap_height),
        ShapedGlyphRotation::Cw90 => (bitmap_height, bitmap_width),
    };
    text_glyph_device_frame(UiFrame::new(
        positioned_frame.x + (positioned_frame.width - width).max(0.0) * 0.5,
        cursor_y + (advance - height).max(0.0) * 0.5,
        width,
        height,
    ))
}
