use crate::render_graph::QueueLane;

use super::super::super::compiled_feature_names::compiled_feature_names;
use super::super::super::render_framework_state::RenderFrameworkState;
use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::submission_record_update::SubmissionRecordUpdate;

pub(super) fn update_base_stats(
    state: &mut RenderFrameworkState,
    context: &FrameSubmissionContext,
    record_update: &SubmissionRecordUpdate,
    frame_generation: u64,
) {
    state.stats.submitted_frames += 1;
    state.stats.last_generation = Some(frame_generation);
    state.stats.last_pipeline = Some(context.pipeline_handle());
    state.stats.last_frame_history = Some(record_update.history_handle());
    let compiled_pipeline = context.compiled_pipeline();
    state.stats.last_effective_features = compiled_feature_names(compiled_pipeline);
    let graph_stats = compiled_pipeline.graph.stats();
    state.stats.last_graph_pass_count = graph_stats.total_pass_count;
    state.stats.last_graph_culled_pass_count = graph_stats.culled_pass_count;
    state.stats.last_graph_resource_lifetime_count = graph_stats.resource_lifetime_count;
    state.stats.last_graph_planned_resource_access_count = graph_stats.total_resource_access_count;
    state.stats.last_graph_planned_dependency_count = graph_stats.total_dependency_count;
    let allocation_plan = compiled_pipeline.graph.transient_allocation_plan();
    state.stats.last_graph_transient_texture_slot_count = allocation_plan.texture_slot_count;
    state.stats.last_graph_transient_buffer_slot_count = allocation_plan.buffer_slot_count;
    state.stats.last_graph_executed_passes =
        state.renderer.last_render_graph_executed_passes().to_vec();
    state.stats.last_graph_executed_executor_ids = state
        .renderer
        .last_render_graph_executed_executor_ids()
        .to_vec();
    state.stats.last_graph_executed_pass_count = state.stats.last_graph_executed_passes.len();
    state.stats.last_graph_executed_resource_access_count = state
        .renderer
        .last_render_graph_executed_resource_access_count();
    state.stats.last_graph_executed_dependency_count =
        state.renderer.last_render_graph_executed_dependency_count();
    state.stats.last_graph_queue_fallback_pass_count = state
        .renderer
        .last_render_graph_executed_queue_fallback_count();
    state.stats.last_virtual_geometry_graph_executed_pass_count = count_executor_prefix(
        &state.stats.last_graph_executed_executor_ids,
        "virtual-geometry.",
    );
    state.stats.last_hybrid_gi_graph_executed_pass_count =
        count_executor_prefix(&state.stats.last_graph_executed_executor_ids, "hybrid-gi.");
    state.stats.last_async_compute_pass_count = state
        .renderer
        .last_render_graph_executed_queue_lane_count(QueueLane::AsyncCompute);
    let ui_stats = context.ui_stats();
    state.stats.last_ui_command_count = ui_stats.command_count();
    state.stats.last_ui_quad_count = ui_stats.quad_count();
    state.stats.last_ui_text_payload_count = ui_stats.text_payload_count();
    state.stats.last_ui_image_payload_count = ui_stats.image_payload_count();
    state.stats.last_ui_clipped_command_count = ui_stats.clipped_command_count();
}

fn count_executor_prefix(executor_ids: &[String], prefix: &str) -> usize {
    executor_ids
        .iter()
        .filter(|executor_id| executor_id.starts_with(prefix))
        .count()
}
