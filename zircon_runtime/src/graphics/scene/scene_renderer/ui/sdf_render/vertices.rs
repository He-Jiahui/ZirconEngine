use std::ops::Range;

use bytemuck::{Pod, Zeroable};

use crate::core::math::UVec2;
use crate::text::ShapedGlyphRotation;
#[cfg(test)]
use crate::text::TextRenderState;
use crate::text::layout::justify_line_advances;
use crate::text::sdf::{
    SdfAtlasBake, SdfAtlasRect, SdfBakeParams, SdfBakedGlyph, SdfGlyphMetrics, SdfMode,
    SdfRunCpuPreparation, scale_sdf_metrics_for_display,
};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{UiTextAlign, UiTextDirection, UiTextWritingMode};

use super::super::render::ScreenSpaceUiTextBatch;
use super::super::sdf_advances::resolved_layout_advances_for_sdf_glyphs;
use super::super::sdf_atlas::{SdfAtlasPlan, SdfAtlasRun};
use super::super::text_pixel_snap::text_frame_device_origin;

mod text;

pub(super) use self::text::{
    horizontal_sdf_glyph_frame, horizontal_shaped_sdf_glyph_frame, vertical_sdf_glyph_frame,
    vertical_shaped_sdf_glyph_frame,
};
use self::text::{push_horizontal_sdf_text_vertices, push_vertical_sdf_text_vertices};

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

#[cfg(test)]
pub(super) fn build_sdf_vertices(
    texts: &[ScreenSpaceUiTextBatch],
    plan: &SdfAtlasPlan,
    atlas_bake: &SdfAtlasBake,
    asset_manager: &crate::asset::ProjectAssetManager,
    viewport_size: UVec2,
) -> Vec<ScreenSpaceUiSdfVertex> {
    let mut text_state = TextRenderState::new(0);
    let cpu_runs = text_state.prepare_sdf_runs_cpu(texts, asset_manager);
    let mut vertices = Vec::new();
    let mut text_ranges = Vec::new();
    build_sdf_vertex_plan(
        &mut vertices,
        &mut text_ranges,
        texts,
        plan,
        atlas_bake,
        &cpu_runs,
        viewport_size,
    );
    vertices
}

#[cfg(test)]
pub(super) fn build_sdf_vertex_plan(
    vertices: &mut Vec<ScreenSpaceUiSdfVertex>,
    text_ranges: &mut Vec<Range<u32>>,
    texts: &[ScreenSpaceUiTextBatch],
    plan: &SdfAtlasPlan,
    atlas_bake: &SdfAtlasBake,
    cpu_runs: &[SdfRunCpuPreparation],
    viewport_size: UVec2,
) {
    build_sdf_vertex_plan_iter(
        vertices,
        text_ranges,
        texts.iter(),
        texts.len(),
        plan,
        atlas_bake,
        cpu_runs,
        viewport_size,
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_sdf_vertex_plan_iter<'a, Texts>(
    vertices: &mut Vec<ScreenSpaceUiSdfVertex>,
    text_ranges: &mut Vec<Range<u32>>,
    texts: Texts,
    text_batch_count: usize,
    plan: &SdfAtlasPlan,
    atlas_bake: &SdfAtlasBake,
    cpu_runs: &[SdfRunCpuPreparation],
    viewport_size: UVec2,
) where
    Texts: IntoIterator<Item = &'a ScreenSpaceUiTextBatch>,
{
    let viewport = UiFrame::new(
        0.0,
        0.0,
        viewport_size.x.max(1) as f32,
        viewport_size.y.max(1) as f32,
    );
    text_ranges.clear();
    text_ranges.reserve(text_batch_count);
    for (index, text) in texts.into_iter().enumerate() {
        let start = vertices.len() as u32;
        if let (Some(run), Some(cpu_run), Some(text_frame_clip)) = (
            plan.runs.get(index),
            cpu_runs.get(index),
            text.frame.intersection(viewport),
        ) {
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
                let glyphs = resolve_run_glyphs(text, run, plan, atlas_bake, cpu_run);
                if matches!(text.writing_mode, UiTextWritingMode::VerticalRl) {
                    push_vertical_sdf_text_vertices(
                        vertices, text, glyphs, plan, clipped, viewport,
                    );
                } else {
                    push_horizontal_sdf_text_vertices(
                        vertices, text, glyphs, plan, clipped, viewport,
                    );
                }
            }
        }
        if let Some(transform) = text.clip_transform {
            transform_sdf_vertices(&mut vertices[start as usize..], transform);
        }
        text_ranges.push(start..vertices.len() as u32);
    }
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
    cpu_run: &SdfRunCpuPreparation,
) -> Vec<RunGlyph> {
    run.glyph_slot_indices
        .iter()
        .copied()
        .enumerate()
        .map(|(glyph_index, slot_index)| match slot_index {
            Some(slot_index) => match (
                atlas_bake.glyphs.get(slot_index),
                plan.slots.get(slot_index),
            ) {
                (Some(baked), Some(slot)) => {
                    run_glyph_from_bake(slot_index, baked, slot.key.bake_params, text.font_size)
                }
                _ => measured_run_glyph(
                    cpu_run
                        .glyph_metrics
                        .get(glyph_index)
                        .copied()
                        .unwrap_or_default(),
                    text.font_size,
                ),
            },
            None => measured_run_glyph(
                cpu_run
                    .glyph_metrics
                    .get(glyph_index)
                    .copied()
                    .unwrap_or_default(),
                text.font_size,
            ),
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

fn measured_run_glyph(metrics: SdfGlyphMetrics, font_size: f32) -> RunGlyph {
    RunGlyph {
        slot_index: None,
        metrics,
        atlas_bitmap_width: 0,
        atlas_bitmap_height: 0,
        visible: false,
        screen_px_range: sdf_screen_px_range(font_size, SdfBakeParams::default()),
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

pub(super) fn horizontal_sdf_text_baseline(
    text: &ScreenSpaceUiTextBatch,
    positioned_frame: UiFrame,
    glyphs: &[RunGlyph],
) -> f32 {
    if let Some(relative_baseline) = text
        .text_decoration_baseline
        .map(|baseline| baseline - text.frame.y)
        .filter(|baseline| baseline.is_finite())
    {
        return positioned_frame.y + relative_baseline;
    }

    let line_ascent = glyphs
        .iter()
        .map(|glyph| glyph.metrics.ascent)
        .fold(text.font_size.max(1.0), f32::max);
    positioned_frame.y
        + (text.line_height.max(text.font_size) - text.font_size.max(1.0)).max(0.0) * 0.5
        + line_ascent
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

pub(super) fn atlas_uv_rect(rect: SdfAtlasRect, atlas_size: UVec2, glyph: &RunGlyph) -> SdfUvRect {
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

pub(super) fn push_clipped_glyph_quad(
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
