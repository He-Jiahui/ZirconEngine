use crate::core::framework::render::{
    FrameHistoryHandle, RenderFrameworkError, RenderViewportHandle,
};
use crate::graphics::scene::RenderGraphLightGridReport;

use super::super::super::render_framework_state::RenderFrameworkState;
use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::record_submission::{
    record_history, update_hybrid_gi_runtime, update_virtual_geometry_runtime,
};
use super::super::runtime_feedback_batch::RuntimeFeedbackBatch;
use super::update_particle_previous_state::update_particle_previous_state_after_success;
use super::update_temporal_camera_history::update_temporal_camera_history_after_success;
use crate::graphics::ViewportRenderFrame;

pub(super) fn record_non_viewport_camera_state_after_success(
    state: &mut RenderFrameworkState,
    viewport: RenderViewportHandle,
    context: &FrameSubmissionContext,
    frame: &ViewportRenderFrame,
    light_grid_report: Option<RenderGraphLightGridReport>,
    runtime_feedback: RuntimeFeedbackBatch,
    generation: u64,
    allocated_history: Option<FrameHistoryHandle>,
) -> Result<(), RenderFrameworkError> {
    let previous_to_release = {
        let record =
            state
                .viewports
                .get_mut(&viewport)
                .ok_or(RenderFrameworkError::UnknownViewport {
                    viewport: viewport.raw(),
                })?;
        let (previous_handle, _, _) =
            record_history(record, context, generation, allocated_history);
        record.record_camera_product_reports(
            context.camera_history_key(),
            light_grid_report,
            frame.virtual_geometry_debug_snapshot.as_ref(),
        );
        let (hybrid_gi_feedback, _particle_feedback, virtual_geometry_feedback) =
            runtime_feedback.into_parts();

        update_hybrid_gi_runtime(record, context.camera_history_key(), hybrid_gi_feedback);
        update_virtual_geometry_runtime(
            record,
            context.camera_history_key(),
            virtual_geometry_feedback,
        );
        update_temporal_camera_history_after_success(
            record,
            frame,
            context.camera_history_key(),
            false,
        );
        update_particle_previous_state_after_success(record, frame, context.camera_history_key());

        allocated_history
            .is_some()
            .then_some(previous_handle)
            .flatten()
    };
    if let Some(handle) = previous_to_release {
        state.renderer.release_history(handle);
    }
    Ok(())
}
