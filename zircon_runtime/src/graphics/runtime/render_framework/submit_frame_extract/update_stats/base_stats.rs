use crate::graphics::pipeline::RenderPassStage;
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
    let post_process_graph = state
        .renderer
        .last_render_graph_post_process_graph()
        .unwrap_or_else(|| context.post_process_graph());
    state.stats.last_post_process_graph_node_count = post_process_graph.node_count();
    state.stats.last_post_process_graph_skipped_node_count =
        post_process_graph.skipped_node_count();
    state.stats.last_post_process_final_composite_node =
        post_process_graph.final_composite_node.clone();
    state.stats.last_post_process_graph_executed_nodes = state
        .renderer
        .last_render_graph_executed_post_process_nodes()
        .to_vec();
    state.stats.last_anti_alias_fallback = context.anti_alias_fallback();
    state.stats.last_advanced_provider_reports = context.advanced_provider_reports().to_vec();
    state.stats.last_solari_runtime_report = context.solari_runtime_report().clone();
    state.stats.last_anti_alias_graph_executed_pass_count =
        count_executor_prefix(&state.stats.last_graph_executed_executor_ids, "post.fxaa");
    state.stats.last_graph_queue_fallback_pass_count = state
        .renderer
        .last_render_graph_executed_queue_fallback_count();
    state.stats.last_virtual_geometry_graph_executed_pass_count = count_executor_prefix(
        &state.stats.last_graph_executed_executor_ids,
        "virtual-geometry.",
    );
    state.stats.last_hybrid_gi_graph_executed_pass_count =
        count_executor_prefix(&state.stats.last_graph_executed_executor_ids, "hybrid-gi.");
    state.stats.last_particle_graph_executed_pass_count =
        count_executor_prefix(&state.stats.last_graph_executed_executor_ids, "particle.");
    state.stats.last_transparent_graph_executed_pass_count = state
        .renderer
        .last_render_graph_executed_stage_count(RenderPassStage::Transparent3d);
    state.stats.last_async_compute_pass_count = state
        .renderer
        .last_render_graph_executed_queue_lane_count(QueueLane::AsyncCompute);
    let ui_stats = context.ui_stats();
    state.stats.last_ui_command_count = ui_stats.command_count();
    state.stats.last_ui_quad_count = ui_stats.quad_count();
    state.stats.last_ui_text_payload_count = ui_stats.text_payload_count();
    state.stats.last_ui_image_payload_count = ui_stats.image_payload_count();
    state.stats.last_ui_clipped_command_count = ui_stats.clipped_command_count();
    state.stats.last_ui_graph_executed_pass_count = state
        .renderer
        .last_render_graph_executed_stage_count(RenderPassStage::Ui);
    state.stats.last_ui_target_size =
        (state.stats.last_ui_graph_executed_pass_count > 0).then(|| context.size());
    state.stats.last_ui_graph_pass_order = runtime_ui_graph_pass_order(
        &state.stats.last_graph_executed_passes,
        state.stats.last_ui_graph_executed_pass_count,
    );
    state.stats.last_material_count = state.renderer.last_material_count();
    state.stats.last_material_ready_count = state.renderer.last_material_ready_count();
    state.stats.last_material_fallback_count = state.renderer.last_material_fallback_count();
    state.stats.last_material_validation_error_count =
        state.renderer.last_material_validation_error_count();
    state.stats.last_sprite_count = state.renderer.last_sprite_count();
    state.stats.last_sprite_ready_count = state.renderer.last_sprite_ready_count();
    state.stats.last_sprite_texture_fallback_count =
        state.renderer.last_sprite_texture_fallback_count();
    state.stats.last_sprite_graph_executed_pass_count =
        count_executor_prefix(&state.stats.last_graph_executed_executor_ids, "sprite.");
    state.stats.last_directional_light_count = context.scene_directional_lights().len();
    state.stats.last_point_light_count = context.scene_point_lights().len();
    state.stats.last_spot_light_count = context.scene_spot_lights().len();
    state.stats.last_ambient_light_count = context.scene_ambient_lights().len();
    state.stats.last_rect_light_count = context.scene_rect_lights().len();
}

fn count_executor_prefix(executor_ids: &[String], prefix: &str) -> usize {
    executor_ids
        .iter()
        .filter(|executor_id| executor_id.starts_with(prefix))
        .count()
}

fn runtime_ui_graph_pass_order(
    executed_passes: &[String],
    ui_graph_executed_pass_count: usize,
) -> Option<String> {
    if ui_graph_executed_pass_count == 0 {
        return None;
    }

    let postprocess = executed_passes
        .iter()
        .position(|pass| pass == "post-process")?;
    let runtime_ui = executed_passes
        .iter()
        .position(|pass| pass == "runtime-ui")?;
    let overlay = executed_passes
        .iter()
        .position(|pass| pass == "overlay-gizmo")?;

    (postprocess < runtime_ui && runtime_ui < overlay).then(|| "postprocess-ui-overlay".to_string())
}

#[cfg(test)]
mod tests {
    use super::runtime_ui_graph_pass_order;

    #[test]
    fn runtime_ui_graph_pass_order_requires_actual_graph_order() {
        let passes = ["post-process", "runtime-ui", "overlay-gizmo"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();

        assert_eq!(
            runtime_ui_graph_pass_order(&passes, 1).as_deref(),
            Some("postprocess-ui-overlay")
        );

        let unordered = ["runtime-ui", "post-process", "overlay-gizmo"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();

        assert_eq!(runtime_ui_graph_pass_order(&unordered, 1), None);
    }

    #[test]
    fn runtime_ui_graph_pass_order_is_absent_without_ui_execution() {
        let passes = ["post-process", "runtime-ui", "overlay-gizmo"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();

        assert_eq!(runtime_ui_graph_pass_order(&passes, 0), None);
    }
}
