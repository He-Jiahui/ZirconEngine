use crate::core::framework::render::{RenderFrameExtract, RenderFrameworkError, RenderViewportHandle};
use crate::ui::surface::{UiRenderCommandKind, UiRenderExtract};

use crate::VisibilityContext;

use super::super::super::wgpu_render_framework::WgpuRenderFramework;
use super::super::frame_submission_context::{FrameSubmissionContext, UiSubmissionStats};
use super::compile_pipeline::compile_submission_pipeline;
use super::resolve_enabled_features::resolve_enabled_features;
use super::resolve_viewport_record_state::resolve_viewport_record_state;

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn build_frame_submission_context(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    extract: &RenderFrameExtract,
    ui_extract: Option<&UiRenderExtract>,
) -> Result<FrameSubmissionContext, RenderFrameworkError> {
    let viewport_state = resolve_viewport_record_state(server, viewport)?;
    let compiled_pipeline = compile_submission_pipeline(&viewport_state, extract)?;
    let (hybrid_gi_enabled, virtual_geometry_enabled) =
        resolve_enabled_features(&compiled_pipeline);
    let visibility_context = VisibilityContext::from_extract_with_history(
        extract,
        viewport_state.previous_visibility.as_ref(),
    );
    let hybrid_gi_update_plan =
        hybrid_gi_enabled.then(|| visibility_context.hybrid_gi_update_plan.clone());
    let hybrid_gi_feedback =
        hybrid_gi_enabled.then(|| visibility_context.hybrid_gi_feedback.clone());
    let virtual_geometry_page_upload_plan = virtual_geometry_enabled
        .then(|| visibility_context.virtual_geometry_page_upload_plan.clone());
    let virtual_geometry_feedback =
        virtual_geometry_enabled.then(|| visibility_context.virtual_geometry_feedback.clone());

    Ok(FrameSubmissionContext {
        size: viewport_state.size,
        pipeline_handle: viewport_state.pipeline_handle,
        quality_profile: viewport_state.quality_profile,
        compiled_pipeline,
        visibility_context,
        ui_stats: ui_extract
            .map(compute_ui_submission_stats)
            .unwrap_or_default(),
        previous_hybrid_gi_runtime: viewport_state.previous_hybrid_gi_runtime,
        previous_virtual_geometry_runtime: viewport_state.previous_virtual_geometry_runtime,
        hybrid_gi_enabled,
        virtual_geometry_enabled,
        hybrid_gi_extract: hybrid_gi_enabled
            .then(|| extract.lighting.hybrid_global_illumination.clone())
            .flatten(),
        hybrid_gi_update_plan,
        hybrid_gi_feedback,
        virtual_geometry_extract: virtual_geometry_enabled
            .then(|| extract.geometry.virtual_geometry.clone())
            .flatten(),
        virtual_geometry_page_upload_plan,
        virtual_geometry_feedback,
        predicted_generation: viewport_state.predicted_generation,
    })
}

fn compute_ui_submission_stats(extract: &UiRenderExtract) -> UiSubmissionStats {
    let mut stats = UiSubmissionStats::default();
    for command in &extract.list.commands {
        stats.command_count += 1;
        if matches!(command.kind, UiRenderCommandKind::Quad) {
            stats.quad_count += 1;
        }
        if command.text.is_some() {
            stats.text_payload_count += 1;
        }
        if command.image.is_some() {
            stats.image_payload_count += 1;
        }
        if command.clip_frame.is_some() {
            stats.clipped_command_count += 1;
        }
    }
    stats
}
