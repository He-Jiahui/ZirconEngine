use zircon_render_graph::QueueLane;

use super::super::super::compiled_feature_names::compiled_feature_names;
use super::super::super::render_server_state::RenderServerState;
use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::submission_record_update::SubmissionRecordUpdate;

pub(super) fn update_base_stats(
    state: &mut RenderServerState,
    context: &FrameSubmissionContext,
    record_update: &SubmissionRecordUpdate,
    frame_generation: u64,
) {
    state.stats.submitted_frames += 1;
    state.stats.last_generation = Some(frame_generation);
    state.stats.last_pipeline = Some(context.pipeline_handle);
    state.stats.last_frame_history = Some(record_update.history_handle);
    state.stats.last_effective_features = compiled_feature_names(&context.compiled_pipeline);
    state.stats.last_async_compute_pass_count = context
        .compiled_pipeline
        .graph
        .passes()
        .iter()
        .filter(|pass| pass.queue == QueueLane::AsyncCompute)
        .count();
}
