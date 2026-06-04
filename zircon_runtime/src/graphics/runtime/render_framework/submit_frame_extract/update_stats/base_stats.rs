use crate::core::framework::render::{
    PostProcessEffectKind, PostProcessGraphResourceNames, PostProcessPassGraph,
    RenderLightReadinessReport, RenderPostProcessEffectStackReport,
    RenderPostProcessEffectStackResourceStatus,
};
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
    state.stats.last_frame_target_size = Some(context.size());
    state.stats.last_frame_render_size = Some(context.render_size());
    state.stats.last_frame_history = Some(record_update.history_handle());
    state.stats.last_frame_history_status = record_update.history_status();
    let compiled_pipeline = context.compiled_pipeline();
    state.stats.last_effective_features = compiled_feature_names(compiled_pipeline);
    let graph_stats = compiled_pipeline.graph.stats();
    state.stats.last_graph_pass_count = graph_stats.total_pass_count;
    state.stats.last_graph_culled_pass_count = graph_stats.culled_pass_count;
    state.stats.last_graph_resource_lifetime_count = graph_stats.resource_lifetime_count;
    state.stats.last_graph_sparse_texture_lifetime_count =
        graph_stats.sparse_texture_lifetime_count;
    state.stats.last_graph_planned_resource_access_count = graph_stats.total_resource_access_count;
    state.stats.last_graph_planned_dependency_count = graph_stats.total_dependency_count;
    let allocation_plan = compiled_pipeline.graph.transient_allocation_plan();
    state.stats.last_graph_transient_texture_slot_count = allocation_plan.texture_slot_count;
    state.stats.last_graph_sparse_texture_slot_count = allocation_plan.sparse_texture_slot_count;
    state.stats.last_graph_transient_buffer_slot_count = allocation_plan.buffer_slot_count;
    state.stats.last_graph_transient_texture_bytes_reserved =
        allocation_plan.dense_texture_bytes_reserved;
    state.stats.last_graph_transient_buffer_bytes_reserved =
        allocation_plan.dense_buffer_bytes_reserved;
    state.stats.last_graph_transient_dense_bytes_reserved =
        allocation_plan.total_dense_bytes_reserved();
    state.stats.last_graph_sparse_texture_virtual_bytes =
        allocation_plan.sparse_texture_virtual_bytes;
    state.stats.last_graph_executed_passes =
        state.renderer.last_render_graph_executed_passes().to_vec();
    state.stats.last_graph_executed_executor_ids = state
        .renderer
        .last_render_graph_executed_executor_ids()
        .to_vec();
    state.stats.last_graph_executed_debug_markers = state
        .renderer
        .last_render_graph_executed_debug_markers()
        .to_vec();
    state.stats.last_graph_executed_pass_count = state.stats.last_graph_executed_passes.len();
    state.stats.last_graph_executed_resource_access_count = state
        .renderer
        .last_render_graph_executed_resource_access_count();
    state.stats.last_graph_executed_dependency_count =
        state.renderer.last_render_graph_executed_dependency_count();
    state.stats.last_graph_compute_dispatch_count =
        state.renderer.last_render_graph_compute_dispatch_count();
    state.stats.last_graph_compute_dispatch_group_count = state
        .renderer
        .last_render_graph_compute_dispatch_group_count();
    state.stats.last_graph_compute_storage_write_resource_count = state
        .renderer
        .last_render_graph_compute_storage_write_resource_count();
    state.stats.last_graph_compute_planned_workload_count = state
        .renderer
        .last_render_graph_compute_planned_workload_count();
    state.stats.last_graph_compute_matched_workload_count = state
        .renderer
        .last_render_graph_compute_matched_workload_count();
    state.stats.last_graph_compute_missing_dispatch_count = state
        .renderer
        .last_render_graph_compute_missing_dispatch_count();
    state.stats.last_graph_compute_workload_mismatch_count = state
        .renderer
        .last_render_graph_compute_workload_mismatch_count();
    state.stats.last_graph_compute_unexpected_dispatch_count = state
        .renderer
        .last_render_graph_compute_unexpected_dispatch_count();
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
    state.stats.last_post_process_effect_stack_report =
        RenderPostProcessEffectStackReport::from_settings_with_resources(
            context.post_process_effect_stack(),
            effect_stack_resource_status(&post_process_graph),
        );
    state.stats.last_post_process_lut_request_count =
        state.renderer.last_post_process_lut_request_count();
    state.stats.last_post_process_lut_ready_count =
        state.renderer.last_post_process_lut_ready_count();
    state.stats.last_post_process_lut_fallback_count =
        state.renderer.last_post_process_lut_fallback_count();
    state.stats.last_post_process_lut_2d_strip_ready_count =
        state.renderer.last_post_process_lut_2d_strip_ready_count();
    state.stats.last_post_process_lut_3d_request_count =
        state.renderer.last_post_process_lut_3d_request_count();
    state.stats.last_post_process_lut_unsupported_shape_count = state
        .renderer
        .last_post_process_lut_unsupported_shape_count();
    state.stats.last_anti_alias_fallback = context.anti_alias_fallback();
    state.stats.last_graph_requested_msaa_sample_count =
        context.anti_alias_fallback().requested_graph_sample_count();
    state.stats.last_graph_effective_msaa_sample_count =
        context.anti_alias_fallback().effective_graph_sample_count();
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
    state.stats.last_shadow_graph_executed_pass_count =
        count_executor_prefix(&state.stats.last_graph_executed_executor_ids, "shadow.");
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
    state.stats.last_material_diagnostic_count = state.renderer.last_material_diagnostic_count();
    let prepared_mesh_queue_stats = state.renderer.last_prepared_mesh_queue_stats();
    state.stats.last_mesh_draw_count = prepared_mesh_queue_stats.draw_count;
    state.stats.last_mesh_opaque_draw_count = prepared_mesh_queue_stats.opaque_draw_count;
    state.stats.last_mesh_alpha_mask_draw_count = prepared_mesh_queue_stats.alpha_mask_draw_count;
    state.stats.last_mesh_transparent_draw_count = prepared_mesh_queue_stats.transparent_draw_count;
    state.stats.last_mesh_early_z_draw_count = prepared_mesh_queue_stats.early_z_draw_count;
    state.stats.last_mesh_prepared_geometry_draw_count =
        prepared_mesh_queue_stats.prepared_geometry_draw_count;
    state.stats.last_mesh_dynamic_geometry_draw_count =
        prepared_mesh_queue_stats.dynamic_geometry_draw_count;
    state.stats.last_mesh_indirect_draw_count = prepared_mesh_queue_stats.indirect_draw_count;
    state.stats.last_mesh_static_batch_candidate_group_count =
        prepared_mesh_queue_stats.static_batch_candidate_group_count;
    state.stats.last_mesh_static_batch_candidate_draw_count =
        prepared_mesh_queue_stats.static_batch_candidate_draw_count;
    state.stats.last_mesh_dynamic_batch_candidate_group_count =
        prepared_mesh_queue_stats.dynamic_batch_candidate_group_count;
    state.stats.last_mesh_dynamic_batch_candidate_draw_count =
        prepared_mesh_queue_stats.dynamic_batch_candidate_draw_count;
    state.stats.last_mesh_gpu_instancing_candidate_group_count =
        prepared_mesh_queue_stats.gpu_instancing_candidate_group_count;
    state.stats.last_mesh_gpu_instancing_candidate_draw_count =
        prepared_mesh_queue_stats.gpu_instancing_candidate_draw_count;
    state.stats.last_sprite_count = state.renderer.last_sprite_count();
    state.stats.last_sprite_ready_count = state.renderer.last_sprite_ready_count();
    state.stats.last_sprite_texture_fallback_count =
        state.renderer.last_sprite_texture_fallback_count();
    state.stats.last_sprite_graph_executed_pass_count =
        count_executor_prefix(&state.stats.last_graph_executed_executor_ids, "sprite.");
    let prepared_sprite_queue_stats = state.renderer.last_prepared_sprite_queue_stats();
    state.stats.last_sprite_draw_batch_count = prepared_sprite_queue_stats.draw_batch_count;
    state.stats.last_sprite_batched_sprite_count = prepared_sprite_queue_stats.sprite_count;
    state.stats.last_sprite_vertex_count = prepared_sprite_queue_stats.vertex_count;
    state.stats.last_sprite_opaque_draw_batch_count =
        prepared_sprite_queue_stats.opaque_draw_batch_count;
    state.stats.last_sprite_alpha_mask_draw_batch_count =
        prepared_sprite_queue_stats.alpha_mask_draw_batch_count;
    state.stats.last_sprite_transparent_draw_batch_count =
        prepared_sprite_queue_stats.transparent_draw_batch_count;
    let light_readiness = RenderLightReadinessReport::from_light_slices(
        context.scene_directional_lights().len(),
        context.scene_point_lights().len(),
        context.scene_spot_lights().len(),
        context.scene_ambient_lights(),
        context.scene_rect_lights(),
    );
    state.stats.last_directional_light_count = light_readiness.directional.total_count;
    state.stats.last_directional_light_ready_count = light_readiness.directional.ready_count;
    state.stats.last_directional_light_degraded_count = light_readiness.directional.degraded_count;
    state.stats.last_point_light_count = light_readiness.point.total_count;
    state.stats.last_point_light_ready_count = light_readiness.point.ready_count;
    state.stats.last_point_light_degraded_count = light_readiness.point.degraded_count;
    state.stats.last_spot_light_count = light_readiness.spot.total_count;
    state.stats.last_spot_light_ready_count = light_readiness.spot.ready_count;
    state.stats.last_spot_light_degraded_count = light_readiness.spot.degraded_count;
    state.stats.last_ambient_light_count = light_readiness.ambient.total_count;
    state.stats.last_ambient_light_ready_count = light_readiness.ambient.ready_count;
    state.stats.last_ambient_light_degraded_count = light_readiness.ambient.degraded_count;
    state.stats.last_rect_light_count = light_readiness.rect.total_count;
    state.stats.last_rect_light_ready_count = light_readiness.rect.ready_count;
    state.stats.last_rect_light_degraded_count = light_readiness.rect.degraded_count;
}

fn count_executor_prefix(executor_ids: &[String], prefix: &str) -> usize {
    executor_ids
        .iter()
        .filter(|executor_id| executor_id.starts_with(prefix))
        .count()
}

fn effect_stack_resource_status(
    post_process_graph: &PostProcessPassGraph,
) -> RenderPostProcessEffectStackResourceStatus {
    let ssr_normal_available = post_process_graph.nodes.iter().any(|node| {
        node.kind == PostProcessEffectKind::EffectStack
            && node
                .required_inputs
                .iter()
                .any(|resource| resource == PostProcessGraphResourceNames::GBUFFER_NORMAL)
    });

    RenderPostProcessEffectStackResourceStatus {
        ssr_normal_available,
    }
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
    use super::{effect_stack_resource_status, runtime_ui_graph_pass_order};
    use crate::core::framework::render::{
        PostProcessEffectKind, PostProcessGraphResourceNames, PostProcessPassGraph,
        PostProcessPassNode,
    };

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

    #[test]
    fn effect_stack_resource_status_detects_graph_bound_ssr_normal() {
        let graph = PostProcessPassGraph {
            nodes: vec![PostProcessPassNode {
                name: "effect-stack".to_string(),
                kind: PostProcessEffectKind::EffectStack,
                required_inputs: vec![PostProcessGraphResourceNames::GBUFFER_NORMAL.to_string()],
                produced_outputs: Vec::new(),
                after: Vec::new(),
            }],
            skipped_nodes: Vec::new(),
            final_composite_node: None,
        };

        assert!(effect_stack_resource_status(&graph).ssr_normal_available);
    }
}
