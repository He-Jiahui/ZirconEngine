use std::ops::Range;
use std::sync::Arc;

use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::layout::{UiFrame, UiLayoutMetrics};
use zircon_runtime_interface::ui::surface::{
    UiPaintElement, UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiRenderFrameExtract,
};

use crate::core::framework::render::{SkyboxMode, UiRenderSubmission};
use crate::graphics::scene::resources::ui_image_resource_id;
use crate::graphics::types::ViewportRenderFrame;
use crate::render_graph::{RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps};

use super::image::ScreenSpaceUiImageBatch;

mod background;
mod color;
mod geometry;
mod glyph_artifact;
mod paint_projection;
mod plan_cache;
mod record;
mod resolved_layout;
mod rich_text;
pub(in crate::graphics::scene::scene_renderer::ui) mod text_advances;
mod text_batches;
pub(in crate::graphics::scene::scene_renderer::ui) mod text_decorations;
mod text_distance_field;
pub(in crate::graphics::scene::scene_renderer::ui) mod text_effects;
mod text_paint;
pub(in crate::graphics::scene::scene_renderer::ui) mod text_projection;
mod text_provenance;
mod text_route_identity;

pub(super) use plan_cache::ScreenSpaceUiPlanCache;
pub(crate) use resolved_layout::ScreenSpaceUiResolvedGlyphArtifactRouteReport;
pub(super) use text_batches::{ScreenSpaceUiTextBatch, ScreenSpaceUiTextRouteContext};

use background::{ScreenSpaceUiBackgroundEffect, ScreenSpaceUiBackgroundTracker};
use color::parse_color;
pub(super) use geometry::ScreenSpaceUiVertex;
pub(super) use geometry::{clipped_scissor, frame_to_scissor, ScreenSpaceUiScissor};
use geometry::{
    coverage_frame, push_border_with_radius, push_rect, push_rect_with_radius, push_rounded_box,
};
pub(in crate::graphics::scene::scene_renderer::ui) use glyph_artifact::{
    ScreenSpaceUiGlyphArtifactCacheIdentity, ScreenSpaceUiGlyphArtifactLine,
};
use paint_projection::{project_transient_paint_elements, ScreenSpaceUiTextPaintProjectionReport};
pub(super) use text_advances::ScreenSpaceUiShapedGlyph;
use text_batches::{push_text_batches, TextPlanOutcome};
pub(in crate::graphics::scene::scene_renderer::ui) use text_route_identity::ScreenSpaceUiTextRouteIdentity;

pub(super) struct PreparedScreenSpaceUi {
    render_segments: Arc<[Arc<PlannedScreenSpaceUi>]>,
    resolved_glyph_artifact_routes: ScreenSpaceUiResolvedGlyphArtifactRouteReport,
}

#[derive(Clone)]
struct ScreenSpaceUiDraw {
    vertices: Range<u32>,
    scissor: ScreenSpaceUiScissor,
}

#[derive(Default)]
pub(super) struct PlannedScreenSpaceUi {
    vertices: Vec<ScreenSpaceUiVertex>,
    draws: Vec<ScreenSpaceUiDraw>,
    post_text_draws: Vec<ScreenSpaceUiDraw>,
    auto_texts: Vec<ScreenSpaceUiTextBatch>,
    native_texts: Vec<ScreenSpaceUiTextBatch>,
    sdf_texts: Vec<ScreenSpaceUiTextBatch>,
    resolved_glyph_artifact_routes: ScreenSpaceUiResolvedGlyphArtifactRouteReport,
    images: Vec<ScreenSpaceUiImageBatch>,
}

impl PlannedScreenSpaceUi {
    pub(super) fn auto_text_batches(&self) -> &[ScreenSpaceUiTextBatch] {
        &self.auto_texts
    }

    pub(super) fn native_text_batches(&self) -> &[ScreenSpaceUiTextBatch] {
        &self.native_texts
    }

    pub(super) fn sdf_text_batches(&self) -> &[ScreenSpaceUiTextBatch] {
        &self.sdf_texts
    }

    pub(super) fn text_batches(&self) -> impl Iterator<Item = &ScreenSpaceUiTextBatch> {
        self.auto_texts
            .iter()
            .chain(&self.native_texts)
            .chain(&self.sdf_texts)
    }

    pub(super) fn image_batches(&self) -> &[ScreenSpaceUiImageBatch] {
        &self.images
    }

    fn has_render_activity(&self) -> bool {
        !self.draws.is_empty()
            || !self.post_text_draws.is_empty()
            || !self.auto_texts.is_empty()
            || !self.native_texts.is_empty()
            || !self.sdf_texts.is_empty()
            || self.resolved_glyph_artifact_routes.has_activity()
            || !self.images.is_empty()
    }

    fn append_non_render_payload_cloned(&mut self, segment: &Self) {
        self.resolved_glyph_artifact_routes
            .merge(segment.resolved_glyph_artifact_routes);
    }
}

fn plan_screen_space_ui_batches(
    extract: &UiRenderExtract,
    viewport_size: crate::core::math::UVec2,
) -> PlannedScreenSpaceUi {
    plan_screen_space_ui_extract_batches(extract, viewport_size, None)
}

#[cfg(test)]
pub(in crate::graphics::scene::scene_renderer::ui) fn native_text_batches_for_product_proof(
    extract: &UiRenderExtract,
    viewport_size: crate::core::math::UVec2,
) -> Vec<ScreenSpaceUiTextBatch> {
    plan_screen_space_ui_batches(extract, viewport_size).native_texts
}

fn plan_screen_space_ui_batches_with_framebuffer_background(
    submission: &UiRenderSubmission,
    viewport_size: crate::core::math::UVec2,
    framebuffer_background_color: Option<[f32; 4]>,
) -> PlannedScreenSpaceUi {
    let viewport = UiFrame::new(
        0.0,
        0.0,
        viewport_size.x.max(1) as f32,
        viewport_size.y.max(1) as f32,
    );
    let full_scissor = ScreenSpaceUiScissor {
        x: 0,
        y: 0,
        width: viewport_size.x.max(1),
        height: viewport_size.y.max(1),
    };

    let mut plan = PlannedScreenSpaceUi::default();
    let mut backgrounds = ScreenSpaceUiBackgroundTracker::with_framebuffer_background(
        viewport,
        framebuffer_background_color,
    );
    let metrics = UiLayoutMetrics::default();
    let mut paint_elements = Vec::new();
    let mut paint_projection_report = ScreenSpaceUiTextPaintProjectionReport::default();

    for segment in submission.segments() {
        let extract = segment.extract();
        append_screen_space_ui_extract_batches(
            extract,
            segment.route_tree_id(),
            |node_id| segment.project_node_id(node_id),
            viewport,
            full_scissor,
            metrics,
            &mut paint_elements,
            &mut paint_projection_report,
            &mut backgrounds,
            &mut plan,
        );
    }
    paint_projection_report.publish_profile_counters();
    record_background_tracker_profile(backgrounds.stats());

    plan
}

fn plan_screen_space_ui_extract_batches(
    extract: &UiRenderExtract,
    viewport_size: crate::core::math::UVec2,
    framebuffer_background_color: Option<[f32; 4]>,
) -> PlannedScreenSpaceUi {
    let viewport = UiFrame::new(
        0.0,
        0.0,
        viewport_size.x.max(1) as f32,
        viewport_size.y.max(1) as f32,
    );
    let full_scissor = ScreenSpaceUiScissor {
        x: 0,
        y: 0,
        width: viewport_size.x.max(1),
        height: viewport_size.y.max(1),
    };
    let mut plan = PlannedScreenSpaceUi::default();
    let mut backgrounds = ScreenSpaceUiBackgroundTracker::with_framebuffer_background(
        viewport,
        framebuffer_background_color,
    );
    let metrics = UiLayoutMetrics::default();
    let mut paint_elements = Vec::new();
    let mut paint_projection_report = ScreenSpaceUiTextPaintProjectionReport::default();
    let extract = UiRenderFrameExtract::from_extract(extract);
    let route_tree_id = Arc::<str>::from(extract.tree_id.0.as_str());
    append_screen_space_ui_extract_batches(
        &extract,
        &route_tree_id,
        |node_id| node_id,
        viewport,
        full_scissor,
        metrics,
        &mut paint_elements,
        &mut paint_projection_report,
        &mut backgrounds,
        &mut plan,
    );
    paint_projection_report.publish_profile_counters();
    record_background_tracker_profile(backgrounds.stats());
    plan
}

fn record_background_tracker_profile(stats: background::ScreenSpaceUiBackgroundTrackerStats) {
    crate::core::diagnostics::profiling::record_counter_batch(
        "runtime",
        &[
            (
                "ui.screen_space_ui_background.query_count",
                stats.query_count as f64,
            ),
            (
                "ui.screen_space_ui_background.effect_visit_count",
                stats.effect_visit_count as f64,
            ),
            (
                "ui.screen_space_ui_background.max_effect_visit_count",
                stats.max_effect_visit_count as f64,
            ),
        ],
    );
}

fn append_screen_space_ui_extract_batches(
    extract: &UiRenderFrameExtract,
    route_tree_id: &Arc<str>,
    project_node_id: impl Fn(UiNodeId) -> UiNodeId,
    viewport: UiFrame,
    full_scissor: ScreenSpaceUiScissor,
    metrics: UiLayoutMetrics,
    paint_elements: &mut Vec<UiPaintElement>,
    paint_projection_report: &mut ScreenSpaceUiTextPaintProjectionReport,
    backgrounds: &mut ScreenSpaceUiBackgroundTracker,
    plan: &mut PlannedScreenSpaceUi,
) {
    append_screen_space_ui_command_batches(
        &extract.list.commands,
        extract.normalized_raster_scale(),
        route_tree_id,
        project_node_id,
        viewport,
        full_scissor,
        metrics,
        paint_elements,
        paint_projection_report,
        backgrounds,
        plan,
    );
}

pub(super) fn append_screen_space_ui_command_batches<'a>(
    commands: impl IntoIterator<Item = &'a UiRenderCommand>,
    raster_scale: f32,
    route_tree_id: &Arc<str>,
    project_node_id: impl Fn(UiNodeId) -> UiNodeId,
    viewport: UiFrame,
    full_scissor: ScreenSpaceUiScissor,
    metrics: UiLayoutMetrics,
    paint_elements: &mut Vec<UiPaintElement>,
    paint_projection_report: &mut ScreenSpaceUiTextPaintProjectionReport,
    backgrounds: &mut ScreenSpaceUiBackgroundTracker,
    plan: &mut PlannedScreenSpaceUi,
) {
    let raster_scale = raster_scale.max(1.0);
    for command in commands {
        let scissor_frame = coverage_frame(
            command.frame,
            command.style.corner_radius,
            command.style.border_width.max(0.0),
        );
        let Some(scissor) =
            clipped_scissor(scissor_frame, command.clip_frame, viewport, full_scissor)
        else {
            backgrounds.observe_command(command, viewport);
            continue;
        };
        project_transient_paint_elements(
            command,
            0,
            metrics,
            paint_elements,
            paint_projection_report,
        );
        let start = plan.vertices.len() as u32;
        let text_projection_rejected = plan_command_batches(
            command,
            paint_elements,
            route_tree_id,
            project_node_id(command.node_id),
            viewport,
            raster_scale,
            backgrounds,
            plan,
        );
        let end = plan.vertices.len() as u32;
        if end > start {
            plan.draws.push(ScreenSpaceUiDraw {
                vertices: start..end,
                scissor,
            });
        }

        if !text_projection_rejected {
            let post_text_start = plan.vertices.len() as u32;
            text_decorations::push_text_decoration_vertices(
                paint_elements,
                command.opacity,
                viewport,
                &mut plan.vertices,
                false,
            );
            let post_text_end = plan.vertices.len() as u32;
            if post_text_end > post_text_start {
                plan.post_text_draws.push(ScreenSpaceUiDraw {
                    vertices: post_text_start..post_text_end,
                    scissor,
                });
            }
        }
        backgrounds.observe_command(command, viewport);
    }
}

pub(super) fn framebuffer_background_color(
    frame: &ViewportRenderFrame,
    attachment_ops: RenderGraphAttachmentOps,
    pass_clear_color: wgpu::Color,
) -> Option<[f32; 4]> {
    match attachment_ops.load {
        RenderGraphAttachmentLoadOp::Clear => opaque_wgpu_color(pass_clear_color),
        RenderGraphAttachmentLoadOp::Load => known_loaded_framebuffer_background_color(frame),
    }
}

fn known_loaded_framebuffer_background_color(frame: &ViewportRenderFrame) -> Option<[f32; 4]> {
    let overlays = frame.overlays();
    let has_overlay_content = overlays
        .highlights
        .as_ref()
        .is_some_and(|highlights| !highlights.entities().is_empty())
        || !overlays.selection_anchors.is_empty()
        || overlays.grid.as_ref().is_some_and(|grid| grid.visible)
        || !overlays.handles.is_empty()
        || !overlays.scene_gizmos.is_empty();
    if frame.environment().skybox.mode != SkyboxMode::Disabled
        || frame.preview().skybox_enabled
        || !frame.meshes().is_empty()
        || !frame.sprites().is_empty()
        || loaded_frame_has_particle_content(frame)
        || has_overlay_content
    {
        return None;
    }

    let clear = frame.preview().clear_color;
    opaque_f32_color([clear.x, clear.y, clear.z, clear.w])
}

fn loaded_frame_has_particle_content(frame: &ViewportRenderFrame) -> bool {
    let particles = &frame.extract.particles;
    !particles.emitters.is_empty()
        || !particles.sprites.is_empty()
        || !frame.previous_particle_sprites().is_empty()
        || !particles.bounds.is_empty()
        || particles
            .gpu_frame
            .as_ref()
            .is_some_and(|gpu| gpu.alive_count > 0 || gpu.spawned_total > 0)
}

fn opaque_wgpu_color(color: wgpu::Color) -> Option<[f32; 4]> {
    opaque_f32_color([
        color.r as f32,
        color.g as f32,
        color.b as f32,
        color.a as f32,
    ])
}

fn opaque_f32_color(color: [f32; 4]) -> Option<[f32; 4]> {
    if color.iter().all(|component| component.is_finite()) && color[3] >= 1.0 {
        Some([
            color[0].clamp(0.0, 1.0),
            color[1].clamp(0.0, 1.0),
            color[2].clamp(0.0, 1.0),
            1.0,
        ])
    } else {
        None
    }
}

fn plan_command_batches(
    command: &UiRenderCommand,
    paint_elements: &[UiPaintElement],
    route_tree_id: &Arc<str>,
    route_node_id: UiNodeId,
    viewport: UiFrame,
    raster_scale: f32,
    backgrounds: &ScreenSpaceUiBackgroundTracker,
    plan: &mut PlannedScreenSpaceUi,
) -> bool {
    if command.opacity <= 0.0 {
        return false;
    }

    let frame = match viewport.intersection(command.frame) {
        Some(frame) => frame,
        None => return false,
    };

    if matches!(command.kind, UiRenderCommandKind::Quad)
        || command.style.background_color.is_some()
        || command.style.border_color.is_some()
        || command.style.border_width > 0.0
    {
        let fill_color = parse_color(
            command.style.background_color.as_deref(),
            [0.16, 0.19, 0.24, 1.0],
            command.opacity,
        );
        let border_width = command.style.border_width.max(0.0);
        let border_color = (border_width > 0.0).then(|| {
            parse_color(
                command.style.border_color.as_deref(),
                [0.85, 0.88, 0.92, 1.0],
                command.opacity,
            )
            .unwrap_or([0.85, 0.88, 0.92, command.opacity])
        });
        match (fill_color, border_color) {
            (Some(fill_color), Some(border_color)) => push_rounded_box(
                &mut plan.vertices,
                command.frame,
                fill_color,
                border_color,
                border_width,
                command.style.corner_radius,
                viewport,
            ),
            (Some(fill_color), None) => push_rect_with_radius(
                &mut plan.vertices,
                command.frame,
                fill_color,
                command.style.corner_radius,
                viewport,
            ),
            (None, Some(border_color)) => push_border_with_radius(
                &mut plan.vertices,
                command.frame,
                border_width,
                border_color,
                command.style.corner_radius,
                viewport,
            ),
            (None, None) => {}
        }
    }

    if let Some(zircon_runtime_interface::ui::surface::UiVisualAssetRef::Image(source)) =
        command.image.as_ref()
    {
        if let Some(texture) = ui_image_resource_id(source) {
            plan.images.push(ScreenSpaceUiImageBatch {
                texture,
                frame: command.frame,
                clip_frame: command.clip_frame,
                tint: [1.0, 1.0, 1.0, command.opacity.clamp(0.0, 1.0)],
            });
        }
    } else if command.image.is_some() || matches!(command.kind, UiRenderCommandKind::Image) {
        let extent = (frame.width.min(frame.height) * 0.68).max(8.0);
        let icon = UiFrame::new(
            frame.x + (frame.width - extent) * 0.5,
            frame.y + (frame.height - extent) * 0.5,
            extent,
            extent,
        );
        let color = parse_color(
            command.style.foreground_color.as_deref(),
            [0.76, 0.88, 0.98, 1.0],
            command.opacity,
        )
        .unwrap_or([0.76, 0.88, 0.98, command.opacity]);
        push_rect(&mut plan.vertices, icon, color, viewport);
    }

    if command.text.as_ref().is_some_and(|text| !text.is_empty()) {
        let color = parse_color(
            command.style.foreground_color.as_deref(),
            [0.96, 0.96, 0.96, 1.0],
            command.opacity,
        )
        .unwrap_or([0.96, 0.96, 0.96, command.opacity]);
        let text_decoration_vertex_start = plan.vertices.len();
        text_decorations::push_text_decoration_vertices(
            paint_elements,
            command.opacity,
            viewport,
            &mut plan.vertices,
            true,
        );
        if matches!(
            push_text_batches(
                command,
                paint_elements,
                route_tree_id,
                route_node_id,
                frame,
                color,
                viewport,
                raster_scale,
                backgrounds,
                plan,
            ),
            TextPlanOutcome::Rejected
        ) {
            plan.vertices.truncate(text_decoration_vertex_start);
            return true;
        }
    }
    false
}

#[cfg(all(test, feature = "ui"))]
mod tests;
