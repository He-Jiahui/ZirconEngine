use bytemuck::{Pod, Zeroable};

use crate::asset::ProjectAssetManager;
use crate::core::framework::render::{ShapedGlyphRotation, VerticalMode};
use crate::core::math::UVec2;
use crate::graphics::text::atlas::{GlyphAtlasFormat, GlyphRasterPlacement, GlyphSmoothingMode};
use crate::graphics::text::font::FontDatabase;
use crate::graphics::text::layout::justify_line_advances;
use crate::graphics::text::shaping::{vertical_glyph_advance, vertical_glyph_rotation};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{UiTextAlign, UiTextDirection, UiTextWritingMode};

use super::super::render::{ScreenSpaceUiShapedGlyph, ScreenSpaceUiTextBatch};
use super::super::sdf_advances::resolved_layout_advances_for_sdf_glyphs;
use super::super::sdf_atlas::{SdfAtlasPlan, SdfAtlasRect, SdfAtlasRun};
use super::super::sdf_char_run::sdf_scalar_is_invisible_format;
use super::super::sdf_font_bake::{
    scale_sdf_metrics_for_display, SdfAtlasBake, SdfBakedGlyph, SdfFontBakeCache, SdfGlyphMetrics,
};
use super::super::text_pixel_snap::{text_frame_device_origin, text_glyph_device_frame};
use crate::graphics::text::sdf::{SdfBakeParams, SdfMode};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable, PartialEq)]
pub(super) struct ScreenSpaceUiSdfVertex {
    pub(super) position: [f32; 4],
    pub(super) uv: [f32; 2],
    pub(super) color: [f32; 4],
    pub(super) screen_px_range: f32,
    pub(super) atlas_px_range: f32,
    pub(super) page_index: u32,
    pub(super) decode_mode: u32,
    pub(super) primitive_kind: u32,
}

pub(super) const SDF_TEXT_PRIMITIVE_GLYPH: u32 = 0;
pub(super) const SDF_TEXT_PRIMITIVE_SOLID: u32 = 1;

#[derive(Clone, Copy)]
pub(super) struct SdfUvRect {
    pub(super) x0: f32,
    pub(super) y0: f32,
    pub(super) x1: f32,
    pub(super) y1: f32,
}

pub(super) struct SdfVertexPlan {
    pub(super) vertices: Vec<ScreenSpaceUiSdfVertex>,
    pub(super) text_ranges: Vec<Range<u32>>,
}

impl ScreenSpaceUiSdfVertex {
    pub(super) fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 8] = wgpu::vertex_attr_array![
            0 => Float32x4,
            1 => Float32x2,
            2 => Float32x4,
            3 => Float32,
            4 => Float32,
            5 => Uint32,
            6 => Uint32,
            7 => Uint32
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

pub(super) fn build_sdf_vertices(
    texts: &[ScreenSpaceUiTextBatch],
    plan: &SdfAtlasPlan,
    atlas_bake: &SdfAtlasBake,
    font_bake: &mut SdfFontBakeCache,
    font_database: &mut FontDatabase,
    asset_manager: &ProjectAssetManager,
    viewport_size: UVec2,
) -> Vec<ScreenSpaceUiSdfVertex> {
    build_sdf_vertex_plan(
        texts,
        plan,
        atlas_bake,
        font_bake,
        font_database,
        asset_manager,
        viewport_size,
    )
    .vertices
}

pub(super) fn build_sdf_vertex_plan(
    texts: &[ScreenSpaceUiTextBatch],
    plan: &SdfAtlasPlan,
    atlas_bake: &SdfAtlasBake,
    font_bake: &mut SdfFontBakeCache,
    font_database: &mut FontDatabase,
    asset_manager: &ProjectAssetManager,
    viewport_size: UVec2,
) -> SdfVertexPlan {
    let viewport = UiFrame::new(
        0.0,
        0.0,
        viewport_size.x.max(1) as f32,
        viewport_size.y.max(1) as f32,
    );
    let mut vertices = Vec::new();
    let mut text_ranges = Vec::with_capacity(texts.len());
    for (index, text) in texts.iter().enumerate() {
        let start = vertices.len() as u32;
        if let (Some(run), Some(text_frame_clip)) =
            (plan.runs.get(index), text.frame.intersection(viewport))
        {
            let has_effect_extent = text.text_effects.outline.is_some()
                || text.text_effects.shadow.is_some()
                || text.text_effects.glow.is_some();
            let clip = if has_effect_extent {
                viewport
            } else {
                text_frame_clip
            };
            let clip_visible = text
                .clip_frame
                .map_or(Some(clip), |clip_frame| clip.intersection(clip_frame));
            if let Some(clipped) = clip_visible {
                let glyphs = resolve_run_glyphs(
                    text,
                    run,
                    plan,
                    atlas_bake,
                    font_bake,
                    font_database,
                    asset_manager,
                );
                if matches!(text.writing_mode, UiTextWritingMode::VerticalRl) {
                    push_vertical_sdf_text_vertices(
                        &mut vertices,
                        text,
                        glyphs,
                        plan,
                        clipped,
                        viewport,
                    );
                } else {
                    push_horizontal_sdf_text_vertices(
                        &mut vertices,
                        text,
                        glyphs,
                        plan,
                        clipped,
                        viewport,
                    );
                }
            }
        }
        if let Some(transform) = text.clip_transform {
            transform_sdf_vertices(&mut vertices[start as usize..], transform);
        }
        text_ranges.push(start..vertices.len() as u32);
    }
    SdfVertexPlan {
        vertices,
        text_ranges,
    }
}

fn push_horizontal_sdf_text_vertices(
    vertices: &mut Vec<ScreenSpaceUiSdfVertex>,
    text: &ScreenSpaceUiTextBatch,
    glyphs: Vec<RunGlyph>,
    plan: &SdfAtlasPlan,
    clip: UiFrame,
    viewport: UiFrame,
) {
    let natural_advances = glyphs
        .iter()
        .map(|glyph| glyph.metrics.advance)
        .collect::<Vec<_>>();
    let natural_text_width = natural_advances.iter().sum();
    let glyph_advances = resolve_sdf_glyph_advances(text, natural_advances, natural_text_width);
    let text_width = glyph_advances.iter().sum();
    let positioned_frame = text_frame_device_origin(text.frame);
    let line_ascent = glyphs
        .iter()
        .map(|glyph| glyph.metrics.ascent)
        .fold(text.font_size.max(1.0), f32::max);
    let baseline = positioned_frame.y
        + (text.line_height.max(text.font_size) - text.font_size.max(1.0)).max(0.0) * 0.5
        + line_ascent;
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

pub(super) fn horizontal_sdf_glyph_frame(
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

fn push_vertical_sdf_text_vertices(
    vertices: &mut Vec<ScreenSpaceUiSdfVertex>,
    text: &ScreenSpaceUiTextBatch,
    glyphs: Vec<RunGlyph>,
    plan: &SdfAtlasPlan,
    clip: UiFrame,
    viewport: UiFrame,
) {
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

pub(super) fn vertical_shaped_sdf_glyph_frame(
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

pub(super) fn vertical_sdf_glyph_frame(
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

#[derive(Clone, Copy)]
pub(super) struct RunGlyph {
    pub(super) slot_index: Option<usize>,
    pub(super) metrics: SdfGlyphMetrics,
    pub(super) atlas_bitmap_width: u32,
    pub(super) atlas_bitmap_height: u32,
    pub(super) visible: bool,
    pub(super) screen_px_range: f32,
    pub(super) atlas_px_range: f32,
}

fn resolve_run_glyphs(
    text: &ScreenSpaceUiTextBatch,
    run: &SdfAtlasRun,
    plan: &SdfAtlasPlan,
    atlas_bake: &SdfAtlasBake,
    font_bake: &mut SdfFontBakeCache,
    font_database: &mut FontDatabase,
    asset_manager: &ProjectAssetManager,
) -> Vec<RunGlyph> {
    let source_scalars = if text.shaped_glyphs.is_empty() {
        text.text.chars().collect::<Vec<_>>()
    } else {
        text.shaped_glyphs
            .iter()
            .map(|glyph| glyph.source_scalar)
            .collect::<Vec<_>>()
    };
    source_scalars
        .into_iter()
        .zip(run.glyph_slot_indices.iter().copied())
        .map(|(glyph, slot_index)| match slot_index {
            Some(slot_index) => match (
                atlas_bake.glyphs.get(slot_index),
                plan.slots.get(slot_index),
            ) {
                (Some(baked), Some(slot)) => {
                    run_glyph_from_bake(slot_index, baked, slot.key.bake_params, text.font_size)
                }
                _ => measured_run_glyph(glyph, text, font_bake, font_database, asset_manager),
            },
            None => measured_run_glyph(glyph, text, font_bake, font_database, asset_manager),
        })
        .collect()
}

fn run_glyph_from_bake(
    slot_index: usize,
    baked: &SdfBakedGlyph,
    bake_params: SdfBakeParams,
    display_px: f32,
) -> RunGlyph {
    RunGlyph {
        slot_index: Some(slot_index),
        metrics: scale_sdf_metrics_for_display(baked.metrics, display_px, bake_params),
        atlas_bitmap_width: baked.metrics.bitmap_width,
        atlas_bitmap_height: baked.metrics.bitmap_height,
        visible: baked.visible,
        screen_px_range: sdf_screen_px_range(display_px, bake_params),
        atlas_px_range: bake_params.spread_px_f32(),
    }
}

fn measured_run_glyph(
    glyph: char,
    text: &ScreenSpaceUiTextBatch,
    font_bake: &mut SdfFontBakeCache,
    font_database: &mut FontDatabase,
    asset_manager: &ProjectAssetManager,
) -> RunGlyph {
    if sdf_scalar_is_invisible_format(glyph) {
        return RunGlyph {
            slot_index: None,
            metrics: SdfGlyphMetrics::default(),
            atlas_bitmap_width: 0,
            atlas_bitmap_height: 0,
            visible: false,
            screen_px_range: sdf_screen_px_range(text.font_size, SdfBakeParams::default()),
            atlas_px_range: SdfBakeParams::default().spread_px_f32(),
        };
    }

    RunGlyph {
        slot_index: None,
        metrics: font_bake.measure_glyph(
            glyph,
            text.font.as_deref(),
            text.font_family.as_deref(),
            text.language.as_deref(),
            text.font_weight,
            text.font_size,
            font_database,
            asset_manager,
        ),
        atlas_bitmap_width: 0,
        atlas_bitmap_height: 0,
        visible: false,
        screen_px_range: sdf_screen_px_range(text.font_size, SdfBakeParams::default()),
        atlas_px_range: SdfBakeParams::default().spread_px_f32(),
    }
}

pub(super) fn aligned_text_start_x(text: &ScreenSpaceUiTextBatch, text_width: f32) -> f32 {
    let positioned_frame = text_frame_device_origin(text.frame);
    let free_width = (positioned_frame.width - text_width).max(0.0);
    let offset = match text.text_align {
        UiTextAlign::Left => 0.0,
        UiTextAlign::Center => free_width * 0.5,
        UiTextAlign::Right => free_width,
        UiTextAlign::Start if matches!(text.text_direction, UiTextDirection::RightToLeft) => {
            free_width
        }
        UiTextAlign::Start => 0.0,
        UiTextAlign::End if matches!(text.text_direction, UiTextDirection::RightToLeft) => 0.0,
        UiTextAlign::End => free_width,
        UiTextAlign::Justify => 0.0,
    };
    positioned_frame.x + offset
}

pub(super) fn resolve_sdf_glyph_advances(
    text: &ScreenSpaceUiTextBatch,
    natural_advances: Vec<f32>,
    natural_width: f32,
) -> Vec<f32> {
    if let Some(layout_advances) = resolved_layout_advances_for_sdf_glyphs(
        text.text.as_str(),
        text.glyph_advances.as_slice(),
        natural_advances.len(),
    ) {
        return layout_advances;
    }

    if !matches!(text.text_align, UiTextAlign::Justify) {
        return natural_advances;
    }
    justify_line_advances(
        text.text.as_str(),
        natural_advances.as_slice(),
        natural_width,
        text.frame.width.max(0.0),
    )
    .unwrap_or(natural_advances)
}

pub(super) fn resolve_vertical_sdf_glyph_advances(
    text: &ScreenSpaceUiTextBatch,
    natural_advances: Vec<f32>,
) -> Vec<f32> {
    resolved_layout_advances_for_sdf_glyphs(
        text.text.as_str(),
        text.glyph_advances.as_slice(),
        natural_advances.len(),
    )
    .unwrap_or(natural_advances)
}

pub(super) fn sdf_screen_px_range(display_px: f32, bake_params: SdfBakeParams) -> f32 {
    bake_params.screen_px_range(display_px)
}

fn atlas_uv_rect(rect: SdfAtlasRect, atlas_size: UVec2, glyph: &RunGlyph) -> SdfUvRect {
    let width = atlas_size.x.max(1) as f32;
    let height = atlas_size.y.max(1) as f32;
    let glyph_width = glyph.atlas_bitmap_width.min(rect.width);
    let glyph_height = glyph.atlas_bitmap_height.min(rect.height);
    SdfUvRect {
        x0: rect.x as f32 / width,
        y0: rect.y as f32 / height,
        x1: rect.x.saturating_add(glyph_width) as f32 / width,
        y1: rect.y.saturating_add(glyph_height) as f32 / height,
    }
}

fn push_clipped_glyph_quad(
    vertices: &mut Vec<ScreenSpaceUiSdfVertex>,
    frame: UiFrame,
    clip: UiFrame,
    viewport: UiFrame,
    uv: SdfUvRect,
    color: [f32; 4],
    screen_px_range: f32,
    atlas_px_range: f32,
    page_index: u32,
    mode: SdfMode,
    rotation: ShapedGlyphRotation,
) {
    let Some(clipped) = frame
        .intersection(clip)
        .and_then(|frame| frame.intersection(viewport))
    else {
        return;
    };
    let left = (clipped.x - frame.x) / frame.width.max(1.0);
    let right = (clipped.right() - frame.x) / frame.width.max(1.0);
    let top = (clipped.y - frame.y) / frame.height.max(1.0);
    let bottom = (clipped.bottom() - frame.y) / frame.height.max(1.0);
    let uv_top_left = sdf_uv_at_destination(uv, left, top, rotation);
    let uv_top_right = sdf_uv_at_destination(uv, right, top, rotation);
    let uv_bottom_right = sdf_uv_at_destination(uv, right, bottom, rotation);
    let uv_bottom_left = sdf_uv_at_destination(uv, left, bottom, rotation);
    let x0 = pixel_to_ndc_x(clipped.x, viewport.width);
    let x1 = pixel_to_ndc_x(clipped.right(), viewport.width);
    let y0 = pixel_to_ndc_y(clipped.y, viewport.height);
    let y1 = pixel_to_ndc_y(clipped.bottom(), viewport.height);

    vertices.extend_from_slice(&[
        ScreenSpaceUiSdfVertex {
            position: clip_position(x0, y0),
            uv: uv_top_left,
            color,
            screen_px_range,
            atlas_px_range,
            page_index,
            decode_mode: mode.shader_discriminant(),
            primitive_kind: SDF_TEXT_PRIMITIVE_GLYPH,
        },
        ScreenSpaceUiSdfVertex {
            position: clip_position(x1, y0),
            uv: uv_top_right,
            color,
            screen_px_range,
            atlas_px_range,
            page_index,
            decode_mode: mode.shader_discriminant(),
            primitive_kind: SDF_TEXT_PRIMITIVE_GLYPH,
        },
        ScreenSpaceUiSdfVertex {
            position: clip_position(x1, y1),
            uv: uv_bottom_right,
            color,
            screen_px_range,
            atlas_px_range,
            page_index,
            decode_mode: mode.shader_discriminant(),
            primitive_kind: SDF_TEXT_PRIMITIVE_GLYPH,
        },
        ScreenSpaceUiSdfVertex {
            position: clip_position(x0, y0),
            uv: uv_top_left,
            color,
            screen_px_range,
            atlas_px_range,
            page_index,
            decode_mode: mode.shader_discriminant(),
            primitive_kind: SDF_TEXT_PRIMITIVE_GLYPH,
        },
        ScreenSpaceUiSdfVertex {
            position: clip_position(x1, y1),
            uv: uv_bottom_right,
            color,
            screen_px_range,
            atlas_px_range,
            page_index,
            decode_mode: mode.shader_discriminant(),
            primitive_kind: SDF_TEXT_PRIMITIVE_GLYPH,
        },
        ScreenSpaceUiSdfVertex {
            position: clip_position(x0, y1),
            uv: uv_bottom_left,
            color,
            screen_px_range,
            atlas_px_range,
            page_index,
            decode_mode: mode.shader_discriminant(),
            primitive_kind: SDF_TEXT_PRIMITIVE_GLYPH,
        },
    ]);
}

pub(super) fn push_clipped_solid_quad(
    vertices: &mut Vec<ScreenSpaceUiSdfVertex>,
    frame: UiFrame,
    clip: UiFrame,
    viewport: UiFrame,
    color: [f32; 4],
) {
    let Some(clipped) = frame
        .intersection(clip)
        .and_then(|frame| frame.intersection(viewport))
    else {
        return;
    };
    let x0 = pixel_to_ndc_x(clipped.x, viewport.width);
    let x1 = pixel_to_ndc_x(clipped.right(), viewport.width);
    let y0 = pixel_to_ndc_y(clipped.y, viewport.height);
    let y1 = pixel_to_ndc_y(clipped.bottom(), viewport.height);
    let vertex = |position: [f32; 2]| ScreenSpaceUiSdfVertex {
        position: clip_position(position[0], position[1]),
        uv: [0.0, 0.0],
        color,
        screen_px_range: 1.0,
        atlas_px_range: 1.0,
        page_index: 0,
        decode_mode: SdfMode::Sdf.shader_discriminant(),
        primitive_kind: SDF_TEXT_PRIMITIVE_SOLID,
    };
    vertices.extend_from_slice(&[
        vertex([x0, y0]),
        vertex([x1, y0]),
        vertex([x1, y1]),
        vertex([x0, y0]),
        vertex([x1, y1]),
        vertex([x0, y1]),
    ]);
}

pub(super) fn transform_sdf_vertices(
    vertices: &mut [ScreenSpaceUiSdfVertex],
    transform: super::super::render::text_projection::ScreenSpaceUiTextClipTransform,
) {
    for vertex in vertices {
        vertex.position = transform.transform_clip_position(vertex.position);
    }
}

fn clip_position(x: f32, y: f32) -> [f32; 4] {
    [x, y, 0.0, 1.0]
}

pub(super) fn sdf_uv_at_destination(
    uv: SdfUvRect,
    destination_x: f32,
    destination_y: f32,
    rotation: ShapedGlyphRotation,
) -> [f32; 2] {
    let (source_x, source_y) = match rotation {
        ShapedGlyphRotation::None => (destination_x, destination_y),
        ShapedGlyphRotation::Cw90 => (destination_y, 1.0 - destination_x),
    };
    [
        uv.x0 + (uv.x1 - uv.x0) * source_x,
        uv.y0 + (uv.y1 - uv.y0) * source_y,
    ]
}

pub(super) fn pixel_to_ndc_x(x: f32, width: f32) -> f32 {
    (x / width.max(1.0)) * 2.0 - 1.0
}

pub(super) fn pixel_to_ndc_y(y: f32, height: f32) -> f32 {
    1.0 - (y / height.max(1.0)) * 2.0
}
use std::ops::Range;
