use zircon_render_server::{RenderServerError, RenderViewportHandle};
use zircon_scene::RenderFrameExtract;

use crate::VisibilityContext;

use super::super::super::wgpu_render_server::WgpuRenderServer;
use super::super::frame_submission_context::FrameSubmissionContext;
use super::compile_pipeline::compile_submission_pipeline;
use super::resolve_enabled_features::resolve_enabled_features;
use super::resolve_viewport_record_state::resolve_viewport_record_state;

pub(in crate::runtime::server::submit_frame_extract) fn build_frame_submission_context(
    server: &WgpuRenderServer,
    viewport: RenderViewportHandle,
    extract: &RenderFrameExtract,
) -> Result<FrameSubmissionContext, RenderServerError> {
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
