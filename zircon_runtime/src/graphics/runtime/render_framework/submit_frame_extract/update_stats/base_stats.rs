use std::collections::BTreeSet;

use crate::core::framework::render::{
    PostProcessGraphResourceNames, PostProcessPassGraph, RenderGraphExecutionCoverageReport,
    RenderLightReadinessReport, RenderPostProcessEffectStackReport,
    RenderPostProcessEffectStackResourceStatus, RenderPostProcessEffectStackSettings,
    RenderShadowExecutionReport, RenderStats,
};
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::scene::anti_alias::fxaa::FXAA_EXECUTOR_ID;
use crate::graphics::scene::anti_alias::smaa::SMAA_EXECUTOR_ID;
use crate::graphics::visibility::{
    FrameVisibility, HzbBuilder, HzbOcclusionCullReport, VisibilityStaticIndexReport,
};
use crate::render_graph::{QueueLane, RenderGraphResourceAccessKind};

use super::super::super::compiled_feature_names::compiled_feature_names;
use super::super::super::render_framework_state::RenderFrameworkState;
use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::submission_record_update::SubmissionRecordUpdate;
use super::light_grid_stats::update_light_grid_stats;
use super::shared_product_reports::SharedViewportProductReports;
use super::ui_stats::runtime_ui_graph_pass_order;

pub(super) fn update_base_stats(
    state: &mut RenderFrameworkState,
    context: &FrameSubmissionContext,
    record_update: &SubmissionRecordUpdate,
    frame_generation: u64,
    shared_product_reports: SharedViewportProductReports,
) {
    state.stats.submitted_frames += 1;
    state.stats.last_generation = Some(frame_generation);
    state.stats.last_pipeline = Some(context.pipeline_handle());
    state.stats.last_frame_target_size = Some(context.size());
    state.stats.last_frame_render_size = Some(context.render_size());
    state.stats.last_frame_history = Some(record_update.history_handle());
    state.stats.last_frame_history_status = record_update.history_status();
    state.stats.last_frame_history_copy_report = state.renderer.last_frame_history_copy_report();
    state.stats.last_camera_target_resolution = context.camera_target_resolution();
    state.stats.last_camera_target_graph_import =
        state.renderer.last_output_target_graph_import_report();
    state.stats.last_camera_target_writeback = state.renderer.last_output_target_writeback_report();
    state.stats.last_capture_report = record_update.capture_report();
    state.stats.last_scene_camera_scheduled_count = context
        .scene_camera_order_report()
        .map_or(0, |report| report.cameras.len());
    state.stats.last_scene_camera_order_ambiguity_count = context
        .scene_camera_order_report()
        .map_or(0, |report| report.ambiguities.len());
    update_visibility_stats(
        &mut state.stats,
        &context.visibility_context().frame_visibility,
    );
    update_visibility_static_index_stats(
        &mut state.stats,
        &context.visibility_context().static_index_report,
    );
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
    let compiled_graph_cache_stats = state.compiled_graph_cache.stats();
    state.stats.last_graph_compiled_cache_hit_count = compiled_graph_cache_stats.hits;
    state.stats.last_graph_compiled_cache_miss_count = compiled_graph_cache_stats.misses;
    state.stats.last_graph_compiled_cache_eviction_count = compiled_graph_cache_stats.evictions;
    state.stats.last_graph_compiled_cache_entry_count = compiled_graph_cache_stats.entries;
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
    state.stats.last_graph_execution_coverage_report =
        graph_execution_coverage_report(compiled_pipeline, &state.stats.last_graph_executed_passes);
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
    state.stats.last_graph_execution_resource_report =
        state.renderer.last_render_graph_execution_resource_report();
    state.stats.last_graph_materialization_report =
        state.renderer.last_render_graph_materialization_report();
    state.stats.last_graph_execution_alias_report = state
        .renderer
        .last_render_graph_execution_alias_report()
        .clone();
    state.stats.last_graph_execution_profile_report =
        state.renderer.last_render_graph_execution_profile_report();
    state.stats.last_graph_stage_execution_report =
        state.renderer.last_render_graph_stage_execution_report();
    state.stats.last_scene_velocity_readback_report =
        state.renderer.last_scene_velocity_readback_report();
    state.stats.last_exposure_readback_report = state.renderer.last_exposure_readback_report();
    state.stats.last_color_lut_readback_report = state.renderer.last_color_lut_readback_report();
    let post_process_graph = state
        .renderer
        .last_render_graph_post_process_graph()
        .unwrap_or_else(|| context.post_process_graph());
    state.stats.last_post_process_graph_node_count = post_process_graph.node_count();
    state.stats.last_post_process_graph_skipped_node_count =
        post_process_graph.skipped_node_count();
    state.stats.last_post_process_output_transfer_node =
        post_process_graph.output_transfer_node.clone();
    state.stats.last_post_process_graph_executed_nodes = state
        .renderer
        .last_render_graph_executed_post_process_nodes()
        .to_vec();
    let motion_vector_camera_status = state.renderer.last_motion_vector_camera_status();
    state.stats.last_motion_vector_camera_status = motion_vector_camera_status;
    state.stats.last_post_process_effect_stack_report =
        RenderPostProcessEffectStackReport::from_settings_with_resources(
            context.post_process_effect_stack(),
            effect_stack_resource_status(
                &post_process_graph,
                &state.stats.last_graph_executed_executor_ids,
                motion_vector_camera_status,
            ),
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
    state.stats.last_anti_alias_graph_executed_pass_count = count_executor_prefix(
        &state.stats.last_graph_executed_executor_ids,
        FXAA_EXECUTOR_ID,
    ) + count_executor_prefix(
        &state.stats.last_graph_executed_executor_ids,
        SMAA_EXECUTOR_ID,
    ) + count_executor_prefix(
        &state.stats.last_graph_executed_executor_ids,
        "temporal.taa-resolve",
    );
    let hzb_plan = HzbBuilder::new(context.render_size()).build_plan();
    state.stats.last_hzb_mip_count = hzb_plan.mip_count as usize;
    state.stats.last_hzb_graph_executed_pass_count = count_executor_prefix(
        &state.stats.last_graph_executed_executor_ids,
        "visibility.hzb-",
    );
    update_hzb_occlusion_stats(
        &mut state.stats,
        state.renderer.last_hzb_occlusion_cull_report(),
    );
    update_light_grid_stats(&mut state.stats, shared_product_reports.light_grid_report());
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
    state.stats.last_particle_velocity_missing_sprite_count =
        particle_velocity_missing_sprite_count(
            context.post_process_effect_stack(),
            &state.stats.last_graph_executed_executor_ids,
            context.particle_sprite_count(),
            context.particle_previous_state_sprite_count(),
        );
    state
        .stats
        .last_particle_velocity_anonymous_stream_ambiguity_count =
        particle_velocity_anonymous_stream_ambiguity_count(
            context.post_process_effect_stack(),
            &state.stats.last_graph_executed_executor_ids,
            context.particle_sprite_count(),
            context.particle_anonymous_stream_ambiguity_sprite_count(),
        );
    state.stats.last_shadow_graph_executed_pass_count =
        count_executor_prefix(&state.stats.last_graph_executed_executor_ids, "shadow.");
    let shadow_atlas_write_count = state
        .renderer
        .last_render_graph_executed_resource_access_count_for(
            PostProcessGraphResourceNames::SHADOW_ATLAS,
            RenderGraphResourceAccessKind::Write,
        );
    let shadow_atlas_read_count = state
        .renderer
        .last_render_graph_executed_resource_access_count_for(
            PostProcessGraphResourceNames::SHADOW_ATLAS,
            RenderGraphResourceAccessKind::Read,
        );
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
    state.stats.last_shader_variant_miss_report = state.renderer.last_shader_variant_miss_report();
    let prepared_mesh_queue_stats = state.renderer.last_prepared_mesh_queue_stats();
    state.stats.last_mesh_draw_count = prepared_mesh_queue_stats.draw_count;
    state.stats.last_mesh_opaque_draw_count = prepared_mesh_queue_stats.opaque_draw_count;
    state.stats.last_mesh_alpha_mask_draw_count = prepared_mesh_queue_stats.alpha_mask_draw_count;
    state.stats.last_mesh_transparent_draw_count = prepared_mesh_queue_stats.transparent_draw_count;
    state.stats.last_mesh_early_z_draw_count = prepared_mesh_queue_stats.early_z_draw_count;
    state.stats.last_mesh_shadow_caster_draw_count =
        prepared_mesh_queue_stats.shadow_caster_draw_count;
    state.stats.last_mesh_alpha_mask_shadow_caster_draw_count =
        prepared_mesh_queue_stats.alpha_mask_shadow_caster_draw_count;
    state.stats.last_mesh_prepared_geometry_draw_count =
        prepared_mesh_queue_stats.prepared_geometry_draw_count;
    state.stats.last_mesh_dynamic_geometry_draw_count =
        prepared_mesh_queue_stats.dynamic_geometry_draw_count;
    state.stats.last_mesh_skinned_draw_count = prepared_mesh_queue_stats.skinned_draw_count;
    state.stats.last_mesh_skinned_palette_upload_count =
        prepared_mesh_queue_stats.skinned_palette_upload_count;
    state.stats.last_mesh_skinned_previous_palette_upload_count =
        prepared_mesh_queue_stats.skinned_previous_palette_upload_count;
    state.stats.last_mesh_skinned_gpu_source_candidate_count =
        prepared_mesh_queue_stats.skinned_gpu_source_candidate_count;
    state
        .stats
        .last_mesh_skinned_gpu_cpu_morphed_source_candidate_count =
        prepared_mesh_queue_stats.skinned_gpu_cpu_morphed_source_candidate_count;
    state
        .stats
        .last_mesh_skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count =
        prepared_mesh_queue_stats.skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count;
    state.stats.last_mesh_skinned_gpu_skinning_draw_count =
        prepared_mesh_queue_stats.skinned_gpu_skinning_draw_count;
    state.stats.last_mesh_skinned_gpu_velocity_draw_count =
        prepared_mesh_queue_stats.skinned_gpu_velocity_draw_count;
    state.stats.last_mesh_indirect_draw_count = prepared_mesh_queue_stats.indirect_draw_count;
    state.stats.last_mesh_lod_draw_count = prepared_mesh_queue_stats.lod_draw_count;
    state.stats.last_mesh_previous_velocity_transform_draw_count =
        prepared_mesh_queue_stats.previous_velocity_transform_draw_count;
    state.stats.last_mesh_missing_velocity_transform_draw_count =
        prepared_mesh_queue_stats.missing_velocity_transform_draw_count;
    state.stats.last_mesh_taa_reactive_mask_command_count =
        prepared_mesh_queue_stats.taa_reactive_mask_command_count;
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
    state.stats.last_mesh_command_count = prepared_mesh_queue_stats.command_count;
    state.stats.last_mesh_cached_command_hit_count =
        prepared_mesh_queue_stats.cached_command_hit_count;
    state.stats.last_mesh_command_rebuild_count = prepared_mesh_queue_stats.command_rebuild_count;
    state.stats.last_mesh_dynamic_command_count = prepared_mesh_queue_stats.dynamic_command_count;
    state.stats.last_mesh_command_cache_miss_count = prepared_mesh_queue_stats.cache_miss_count;
    state
        .stats
        .last_mesh_command_cache_invalidated_transform_count =
        prepared_mesh_queue_stats.cache_invalidated_transform_count;
    state
        .stats
        .last_mesh_command_cache_invalidated_geometry_count =
        prepared_mesh_queue_stats.cache_invalidated_geometry_count;
    state
        .stats
        .last_mesh_command_cache_invalidated_material_count =
        prepared_mesh_queue_stats.cache_invalidated_material_count;
    state.stats.last_mesh_replay_state_change_count = prepared_mesh_queue_stats.state_change_count;
    state.stats.last_mesh_replay_bind_skip_count = prepared_mesh_queue_stats.bind_skip_count;
    state.stats.last_indirect_batch_count = prepared_mesh_queue_stats.indirect_batch_count;
    state.stats.last_indirect_batched_draw_count =
        prepared_mesh_queue_stats.indirect_batched_draw_count;
    state.stats.last_indirect_fallback_draw_count =
        prepared_mesh_queue_stats.indirect_fallback_draw_count;
    state.stats.last_indirect_args_count = prepared_mesh_queue_stats.indirect_args_count;
    state.stats.last_gpu_scene_primitive_count =
        prepared_mesh_queue_stats.gpu_scene_primitive_count;
    state.stats.last_gpu_scene_instance_count = prepared_mesh_queue_stats.gpu_scene_instance_count;
    state.stats.last_gpu_scene_dirty_entry_count =
        prepared_mesh_queue_stats.gpu_scene_dirty_entry_count;
    state.stats.last_gpu_scene_uploaded_bytes = prepared_mesh_queue_stats.gpu_scene_uploaded_bytes;
    state.stats.last_gpu_scene_upload_path = prepared_mesh_queue_stats.gpu_scene_upload_path;
    state.stats.last_gpu_scene_free_span_count =
        prepared_mesh_queue_stats.gpu_scene_free_span_count;
    state.stats.last_gpu_scene_primitive_upload_range_count =
        prepared_mesh_queue_stats.gpu_scene_primitive_upload_range_count;
    state.stats.last_gpu_scene_instance_upload_range_count =
        prepared_mesh_queue_stats.gpu_scene_instance_upload_range_count;
    state.stats.last_sprite_count = state.renderer.last_sprite_count();
    state.stats.last_sprite_ready_count = state.renderer.last_sprite_ready_count();
    state.stats.last_sprite_texture_fallback_count =
        state.renderer.last_sprite_texture_fallback_count();
    state.stats.last_sprite_graph_executed_pass_count =
        count_executor_prefix(&state.stats.last_graph_executed_executor_ids, "sprite.");
    let prepared_sprite_queue_stats = state.renderer.last_prepared_sprite_queue_stats();
    state.stats.last_sprite_draw_batch_count = prepared_sprite_queue_stats.draw_batch_count;
    state.stats.last_sprite_batched_sprite_count = prepared_sprite_queue_stats.sprite_count;
    state.stats.last_sprite_image_slice_count = prepared_sprite_queue_stats.image_slice_count;
    state.stats.last_sprite_expanded_image_slice_count =
        prepared_sprite_queue_stats.expanded_image_slice_count;
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
    let shadowed_light_count = shadow_casting_atlas_light_count(context);
    state.stats.last_shadow_execution_report = RenderShadowExecutionReport::new(
        state.stats.last_shadow_graph_executed_pass_count,
        shadow_atlas_write_count,
        shadow_atlas_read_count,
        state.stats.last_mesh_shadow_caster_draw_count,
        state.stats.last_mesh_alpha_mask_shadow_caster_draw_count,
        shadowed_light_count,
        state.stats.last_directional_light_ready_count,
    );
}

fn shadow_casting_atlas_light_count(context: &FrameSubmissionContext) -> usize {
    context
        .scene_directional_lights()
        .iter()
        .filter(|light| matches!(light.shadow, Some(shadow) if shadow.casts_shadow))
        .count()
        + context
            .scene_point_lights()
            .iter()
            .filter(|light| matches!(light.shadow, Some(shadow) if shadow.casts_shadow))
            .count()
        + context
            .scene_spot_lights()
            .iter()
            .filter(|light| matches!(light.shadow, Some(shadow) if shadow.casts_shadow))
            .count()
}

fn count_executor_prefix(executor_ids: &[String], prefix: &str) -> usize {
    executor_ids
        .iter()
        .filter(|executor_id| executor_id.starts_with(prefix))
        .count()
}

fn update_visibility_stats(stats: &mut RenderStats, frame_visibility: &FrameVisibility) {
    stats.last_visibility_view_count = frame_visibility.views.len();
    stats.last_visibility_input_count = frame_visibility
        .views
        .iter()
        .map(|view| view.stats.input_count)
        .sum();
    stats.last_visibility_layer_filtered_count = frame_visibility
        .views
        .iter()
        .map(|view| view.stats.layer_filtered_count)
        .sum();
    stats.last_visibility_frustum_culled_count = frame_visibility
        .views
        .iter()
        .map(|view| view.stats.frustum_culled_count)
        .sum();
    stats.last_visibility_occlusion_culled_count = frame_visibility
        .views
        .iter()
        .map(|view| view.stats.occlusion_culled_count)
        .sum();
    stats.last_visibility_visible_count = frame_visibility
        .views
        .iter()
        .map(|view| view.stats.visible_count)
        .sum();
}

fn update_visibility_static_index_stats(
    stats: &mut RenderStats,
    report: &VisibilityStaticIndexReport,
) {
    stats.last_visibility_static_index_full_rebuild_count =
        report.frame_full_rebuild_count as usize;
    stats.last_visibility_static_index_incremental_update_count =
        report.frame_incremental_update_count as usize;
    stats.last_visibility_static_index_inserted_count = report.inserted_count;
    stats.last_visibility_static_index_updated_count = report.updated_count;
    stats.last_visibility_static_index_removed_count = report.removed_count;
    stats.last_visibility_static_index_indexed_entity_count = report.indexed_entity_count;
    stats.last_visibility_static_index_occupied_cell_count = report.occupied_cell_count;
    stats.last_visibility_static_index_main_view_prefilter_used = report.main_view_prefilter_used;
    stats.last_visibility_static_index_main_view_static_input_count =
        report.main_view_static_input_count;
    stats.last_visibility_static_index_main_view_static_candidate_count =
        report.main_view_static_candidate_count;
}

fn update_hzb_occlusion_stats(stats: &mut RenderStats, report: Option<HzbOcclusionCullReport>) {
    let Some(report) = report else {
        stats.last_hzb_occlusion_reported = false;
        stats.last_hzb_occlusion_candidate_arg_count = 0;
        stats.last_hzb_occlusion_candidate_instance_count = 0;
        stats.last_hzb_occlusion_dispatch_group_count = 0;
        stats.last_hzb_occlusion_dispatched_phase_count = 0;
        stats.last_hzb_occlusion_history_available = false;
        stats.last_hzb_occlusion_readback_available = false;
        stats.last_hzb_occlusion_tested_arg_count = 0;
        stats.last_hzb_occlusion_tested_instance_count = 0;
        stats.last_hzb_occlusion_culled_arg_count = 0;
        stats.last_hzb_occlusion_culled_instance_count = 0;
        stats.last_hzb_occlusion_indirect_args_readback_available = false;
        stats.last_hzb_occlusion_readback_arg_count = 0;
        stats.last_hzb_occlusion_compacted_draw_count = 0;
        stats.last_hzb_occlusion_zero_instance_arg_count = 0;
        stats.last_hzb_occlusion_remaining_instance_count = 0;
        return;
    };

    stats.last_hzb_occlusion_reported = true;
    stats.last_hzb_occlusion_candidate_arg_count = report.candidate_arg_count as usize;
    stats.last_hzb_occlusion_candidate_instance_count = report.candidate_instance_count as usize;
    stats.last_hzb_occlusion_dispatch_group_count = report.dispatch_group_count as usize;
    stats.last_hzb_occlusion_dispatched_phase_count = report.dispatched_phase_count as usize;
    stats.last_hzb_occlusion_history_available = report.history_available;
    if let Some(indirect_args_readback) = report.indirect_args_readback {
        stats.last_hzb_occlusion_indirect_args_readback_available = true;
        stats.last_hzb_occlusion_readback_arg_count =
            indirect_args_readback.readback_arg_count as usize;
        stats.last_hzb_occlusion_compacted_draw_count =
            indirect_args_readback.compacted_draw_count as usize;
        stats.last_hzb_occlusion_zero_instance_arg_count =
            indirect_args_readback.zero_instance_arg_count as usize;
        stats.last_hzb_occlusion_remaining_instance_count =
            indirect_args_readback.remaining_instance_count as usize;
    } else {
        stats.last_hzb_occlusion_indirect_args_readback_available = false;
        stats.last_hzb_occlusion_readback_arg_count = 0;
        stats.last_hzb_occlusion_compacted_draw_count = 0;
        stats.last_hzb_occlusion_zero_instance_arg_count = 0;
        stats.last_hzb_occlusion_remaining_instance_count = 0;
    }
    let Some(readback_stats) = report.readback_stats else {
        stats.last_hzb_occlusion_readback_available = false;
        stats.last_hzb_occlusion_tested_arg_count = 0;
        stats.last_hzb_occlusion_tested_instance_count = 0;
        stats.last_hzb_occlusion_culled_arg_count = 0;
        stats.last_hzb_occlusion_culled_instance_count = 0;
        return;
    };

    stats.last_hzb_occlusion_readback_available = true;
    stats.last_hzb_occlusion_tested_arg_count = readback_stats.tested_arg_count as usize;
    stats.last_hzb_occlusion_tested_instance_count = readback_stats.tested_instance_count as usize;
    stats.last_hzb_occlusion_culled_arg_count = readback_stats.culled_arg_count as usize;
    stats.last_hzb_occlusion_culled_instance_count = readback_stats.culled_instance_count as usize;
    stats.last_visibility_occlusion_culled_count = readback_stats.culled_instance_count as usize;
}

fn graph_execution_coverage_report(
    compiled_pipeline: &crate::graphics::pipeline::CompiledRenderPipeline,
    executed_passes: &[String],
) -> RenderGraphExecutionCoverageReport {
    graph_execution_coverage_report_from_names(
        compiled_pipeline
            .graph
            .passes()
            .iter()
            .filter(|pass| !pass.culled)
            .map(|pass| pass.name.as_str()),
        executed_passes,
    )
}

fn graph_execution_coverage_report_from_names<'a>(
    planned_live_passes: impl IntoIterator<Item = &'a str>,
    executed_passes: &[String],
) -> RenderGraphExecutionCoverageReport {
    let planned_live_passes = planned_live_passes
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let mut executed_unique_passes = BTreeSet::new();
    let mut duplicate_executed_pass_count = 0;
    for pass_name in executed_passes {
        if !executed_unique_passes.insert(pass_name.clone()) {
            duplicate_executed_pass_count += 1;
        }
    }

    let matched_planned_pass_count = planned_live_passes
        .intersection(&executed_unique_passes)
        .count();
    let missing_planned_pass_count = planned_live_passes
        .len()
        .saturating_sub(matched_planned_pass_count);
    let unexpected_executed_pass_count = executed_unique_passes
        .difference(&planned_live_passes)
        .count();

    RenderGraphExecutionCoverageReport::new(
        planned_live_passes.len(),
        executed_passes.len(),
        matched_planned_pass_count,
        missing_planned_pass_count,
        unexpected_executed_pass_count,
        duplicate_executed_pass_count,
    )
}

fn effect_stack_resource_status(
    post_process_graph: &PostProcessPassGraph,
    executed_executor_ids: &[String],
    motion_vector_camera_status: crate::core::framework::render::MotionVectorCameraStatus,
) -> RenderPostProcessEffectStackResourceStatus {
    let ssr_normal_available = effect_stack_uses_resource(
        post_process_graph,
        PostProcessGraphResourceNames::GBUFFER_NORMAL,
    );
    let ssr_temporal_history_available = effect_stack_uses_resource(
        post_process_graph,
        PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION,
    );
    let motion_vector_available = effect_stack_uses_resource(
        post_process_graph,
        PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX,
    );
    let motion_vector_camera_available = executed_executor_ids
        .iter()
        .any(|executor_id| executor_id == "temporal.velocity-camera");
    let motion_vector_object_available = executed_executor_ids
        .iter()
        .any(|executor_id| executor_id == "temporal.velocity-object");
    let motion_vector_tile_max_available = executed_executor_ids
        .iter()
        .any(|executor_id| executor_id == "post.motion-vector-tile-max");
    let motion_vector_tile_max_coarse_available = executed_executor_ids
        .iter()
        .any(|executor_id| executor_id == "post.motion-vector-tile-max-coarse");
    let motion_vector_neighbor_max_available = executed_executor_ids
        .iter()
        .any(|executor_id| executor_id == "post.motion-vector-neighbor-max");
    RenderPostProcessEffectStackResourceStatus {
        ssr_normal_available,
        ssr_temporal_history_available,
        motion_vector_available,
        motion_vector_camera_available,
        motion_vector_object_available,
        motion_vector_tile_max_available,
        motion_vector_tile_max_coarse_available,
        motion_vector_neighbor_max_available,
        motion_vector_camera_status,
        motion_vector_prepass_available: motion_vector_camera_status
            == crate::core::framework::render::MotionVectorCameraStatus::Ready
            && motion_vector_camera_available
            && motion_vector_object_available
            && motion_vector_tile_max_available
            && motion_vector_tile_max_coarse_available
            && motion_vector_neighbor_max_available,
    }
}

fn effect_stack_uses_resource(
    post_process_graph: &PostProcessPassGraph,
    resource_name: &str,
) -> bool {
    post_process_graph.nodes.iter().any(|node| {
        node.required_inputs
            .iter()
            .any(|resource| resource == resource_name)
    })
}

fn particle_velocity_missing_sprite_count(
    effect_stack: RenderPostProcessEffectStackSettings,
    executed_executor_ids: &[String],
    particle_sprite_count: usize,
    particle_previous_state_sprite_count: usize,
) -> usize {
    if particle_velocity_diagnostics_enabled(
        effect_stack,
        executed_executor_ids,
        particle_sprite_count,
    ) {
        particle_sprite_count.saturating_sub(particle_previous_state_sprite_count)
    } else {
        0
    }
}

fn particle_velocity_anonymous_stream_ambiguity_count(
    effect_stack: RenderPostProcessEffectStackSettings,
    executed_executor_ids: &[String],
    particle_sprite_count: usize,
    particle_anonymous_stream_ambiguity_sprite_count: usize,
) -> usize {
    if particle_velocity_diagnostics_enabled(
        effect_stack,
        executed_executor_ids,
        particle_sprite_count,
    ) {
        particle_anonymous_stream_ambiguity_sprite_count
    } else {
        0
    }
}

fn particle_velocity_diagnostics_enabled(
    effect_stack: RenderPostProcessEffectStackSettings,
    executed_executor_ids: &[String],
    particle_sprite_count: usize,
) -> bool {
    if particle_sprite_count == 0 {
        return false;
    }
    let reconstructed_velocity_requested =
        effect_stack.motion_blur.is_enabled() || effect_stack.screen_space_reflection.is_enabled();
    let particle_transparent_executed = executed_executor_ids
        .iter()
        .any(|executor_id| executor_id == "particle.transparent");
    reconstructed_velocity_requested && particle_transparent_executed
}

#[cfg(test)]
mod tests {
    use super::{
        effect_stack_resource_status, graph_execution_coverage_report_from_names,
        particle_velocity_anonymous_stream_ambiguity_count, particle_velocity_missing_sprite_count,
        update_hzb_occlusion_stats, update_visibility_static_index_stats, update_visibility_stats,
    };
    use crate::core::framework::render::{
        MotionVectorCameraStatus, PostProcessEffectKind, PostProcessGraphResourceNames,
        PostProcessPassGraph, PostProcessPassNode, RenderPostProcessEffectStackSettings,
        RenderStats,
    };
    use crate::graphics::visibility::{
        FrameVisibility, HzbOcclusionCullReadbackStats, HzbOcclusionCullReport, ViewCullingStats,
        ViewVisibilityContext, VisibilityStaticIndexReport,
    };

    fn effect_stack_graph(required_inputs: Vec<String>) -> PostProcessPassGraph {
        PostProcessPassGraph::from_ordered_nodes(
            vec![
                PostProcessPassNode::new("uber", PostProcessEffectKind::Uber)
                    .with_required_inputs(required_inputs),
            ],
            Vec::new(),
            None,
        )
    }

    #[test]
    fn graph_execution_coverage_report_counts_missing_unexpected_and_duplicate_passes() {
        let executed_passes = vec![
            "depth-prepass".to_string(),
            "depth-prepass".to_string(),
            "uber".to_string(),
            "unexpected-pass".to_string(),
        ];

        let report = graph_execution_coverage_report_from_names(
            ["depth-prepass", "opaque", "uber"],
            &executed_passes,
        );

        assert_eq!(report.planned_live_pass_count, 3);
        assert_eq!(report.executed_pass_count, 4);
        assert_eq!(report.matched_planned_pass_count, 2);
        assert_eq!(report.missing_planned_pass_count, 1);
        assert_eq!(report.unexpected_executed_pass_count, 1);
        assert_eq!(report.duplicate_executed_pass_count, 1);
    }

    #[test]
    fn update_visibility_stats_sums_per_view_culling_counts() {
        let frame_visibility = FrameVisibility {
            views: vec![
                ViewVisibilityContext {
                    stats: ViewCullingStats {
                        input_count: 4,
                        layer_filtered_count: 1,
                        frustum_culled_count: 1,
                        occlusion_culled_count: 0,
                        visible_count: 2,
                    },
                    ..ViewVisibilityContext::default()
                },
                ViewVisibilityContext {
                    stats: ViewCullingStats {
                        input_count: 4,
                        layer_filtered_count: 0,
                        frustum_culled_count: 2,
                        occlusion_culled_count: 1,
                        visible_count: 1,
                    },
                    ..ViewVisibilityContext::default()
                },
            ],
            ..FrameVisibility::default()
        };
        let mut stats = RenderStats::default();

        update_visibility_stats(&mut stats, &frame_visibility);

        assert_eq!(stats.last_visibility_view_count, 2);
        assert_eq!(stats.last_visibility_input_count, 8);
        assert_eq!(stats.last_visibility_layer_filtered_count, 1);
        assert_eq!(stats.last_visibility_frustum_culled_count, 3);
        assert_eq!(stats.last_visibility_occlusion_culled_count, 1);
        assert_eq!(stats.last_visibility_visible_count, 3);
    }

    #[test]
    fn update_visibility_static_index_stats_records_latest_report() {
        let mut stats = RenderStats::default();
        let report = VisibilityStaticIndexReport {
            frame_full_rebuild_count: 0,
            frame_incremental_update_count: 1,
            inserted_count: 2,
            updated_count: 3,
            removed_count: 4,
            indexed_entity_count: 10,
            occupied_cell_count: 7,
            main_view_prefilter_used: true,
            main_view_static_input_count: 12,
            main_view_static_candidate_count: 5,
            ..VisibilityStaticIndexReport::default()
        };

        update_visibility_static_index_stats(&mut stats, &report);

        assert_eq!(stats.last_visibility_static_index_full_rebuild_count, 0);
        assert_eq!(
            stats.last_visibility_static_index_incremental_update_count,
            1
        );
        assert_eq!(stats.last_visibility_static_index_inserted_count, 2);
        assert_eq!(stats.last_visibility_static_index_updated_count, 3);
        assert_eq!(stats.last_visibility_static_index_removed_count, 4);
        assert_eq!(stats.last_visibility_static_index_indexed_entity_count, 10);
        assert_eq!(stats.last_visibility_static_index_occupied_cell_count, 7);
        assert!(stats.last_visibility_static_index_main_view_prefilter_used);
        assert_eq!(
            stats.last_visibility_static_index_main_view_static_input_count,
            12
        );
        assert_eq!(
            stats.last_visibility_static_index_main_view_static_candidate_count,
            5
        );
    }

    #[test]
    fn update_hzb_occlusion_stats_records_latest_cull_report() {
        let mut stats = RenderStats::default();
        let report = HzbOcclusionCullReport::single_frame_reproject(6, 42, 2, 1, true);

        update_hzb_occlusion_stats(&mut stats, Some(report));

        assert!(stats.last_hzb_occlusion_reported);
        assert_eq!(stats.last_hzb_occlusion_candidate_arg_count, 6);
        assert_eq!(stats.last_hzb_occlusion_candidate_instance_count, 42);
        assert_eq!(stats.last_hzb_occlusion_dispatch_group_count, 2);
        assert_eq!(stats.last_hzb_occlusion_dispatched_phase_count, 1);
        assert!(stats.last_hzb_occlusion_history_available);
        assert!(!stats.last_hzb_occlusion_readback_available);
    }

    #[test]
    fn update_hzb_occlusion_stats_records_readback_and_overrides_visibility_occlusion_count() {
        let mut stats = RenderStats {
            last_visibility_occlusion_culled_count: 3,
            ..RenderStats::default()
        };
        let report = HzbOcclusionCullReport::single_frame_reproject(6, 42, 2, 1, true)
            .with_readback_stats(HzbOcclusionCullReadbackStats::new(6, 42, 2, 18))
            .with_indirect_args_readback(
                crate::graphics::visibility::HzbOcclusionIndirectArgsReadbackSummary::new(
                    6, 4, 2, 24,
                ),
            );

        update_hzb_occlusion_stats(&mut stats, Some(report));

        assert!(stats.last_hzb_occlusion_readback_available);
        assert_eq!(stats.last_hzb_occlusion_tested_arg_count, 6);
        assert_eq!(stats.last_hzb_occlusion_tested_instance_count, 42);
        assert_eq!(stats.last_hzb_occlusion_culled_arg_count, 2);
        assert_eq!(stats.last_hzb_occlusion_culled_instance_count, 18);
        assert!(stats.last_hzb_occlusion_indirect_args_readback_available);
        assert_eq!(stats.last_hzb_occlusion_readback_arg_count, 6);
        assert_eq!(stats.last_hzb_occlusion_compacted_draw_count, 4);
        assert_eq!(stats.last_hzb_occlusion_zero_instance_arg_count, 2);
        assert_eq!(stats.last_hzb_occlusion_remaining_instance_count, 24);
        assert_eq!(stats.last_visibility_occlusion_culled_count, 18);
    }

    #[test]
    fn update_hzb_occlusion_stats_resets_when_no_report() {
        let mut stats = RenderStats {
            last_hzb_occlusion_reported: true,
            last_hzb_occlusion_candidate_arg_count: 6,
            last_hzb_occlusion_candidate_instance_count: 42,
            last_hzb_occlusion_dispatch_group_count: 2,
            last_hzb_occlusion_dispatched_phase_count: 1,
            last_hzb_occlusion_history_available: true,
            last_hzb_occlusion_readback_available: true,
            last_hzb_occlusion_tested_arg_count: 6,
            last_hzb_occlusion_tested_instance_count: 42,
            last_hzb_occlusion_culled_arg_count: 2,
            last_hzb_occlusion_culled_instance_count: 18,
            last_hzb_occlusion_indirect_args_readback_available: true,
            last_hzb_occlusion_readback_arg_count: 6,
            last_hzb_occlusion_compacted_draw_count: 4,
            last_hzb_occlusion_zero_instance_arg_count: 2,
            last_hzb_occlusion_remaining_instance_count: 24,
            ..RenderStats::default()
        };

        update_hzb_occlusion_stats(&mut stats, None);

        assert!(!stats.last_hzb_occlusion_reported);
        assert_eq!(stats.last_hzb_occlusion_candidate_arg_count, 0);
        assert_eq!(stats.last_hzb_occlusion_candidate_instance_count, 0);
        assert_eq!(stats.last_hzb_occlusion_dispatch_group_count, 0);
        assert_eq!(stats.last_hzb_occlusion_dispatched_phase_count, 0);
        assert!(!stats.last_hzb_occlusion_history_available);
        assert!(!stats.last_hzb_occlusion_readback_available);
        assert_eq!(stats.last_hzb_occlusion_tested_arg_count, 0);
        assert_eq!(stats.last_hzb_occlusion_tested_instance_count, 0);
        assert_eq!(stats.last_hzb_occlusion_culled_arg_count, 0);
        assert_eq!(stats.last_hzb_occlusion_culled_instance_count, 0);
        assert!(!stats.last_hzb_occlusion_indirect_args_readback_available);
        assert_eq!(stats.last_hzb_occlusion_readback_arg_count, 0);
        assert_eq!(stats.last_hzb_occlusion_compacted_draw_count, 0);
        assert_eq!(stats.last_hzb_occlusion_zero_instance_arg_count, 0);
        assert_eq!(stats.last_hzb_occlusion_remaining_instance_count, 0);
    }

    #[test]
    fn effect_stack_resource_status_detects_graph_bound_ssr_normal() {
        let graph = effect_stack_graph(vec![
            PostProcessGraphResourceNames::GBUFFER_NORMAL.to_string()
        ]);

        assert!(
            effect_stack_resource_status(&graph, &[], MotionVectorCameraStatus::NotRequested)
                .ssr_normal_available
        );
    }

    #[test]
    fn effect_stack_resource_status_detects_graph_bound_ssr_temporal_history_without_prepass() {
        let graph = effect_stack_graph(vec![
            PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION.to_string(),
            PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string(),
        ]);

        let status =
            effect_stack_resource_status(&graph, &[], MotionVectorCameraStatus::NotRequested);

        assert!(status.ssr_temporal_history_available);
        assert!(status.motion_vector_available);
        assert!(!status.motion_vector_prepass_available);
    }

    #[test]
    fn effect_stack_resource_status_detects_graph_bound_motion_vector_neighbor_max_without_prepass()
    {
        let graph = effect_stack_graph(vec![
            PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string(),
        ]);

        let status =
            effect_stack_resource_status(&graph, &[], MotionVectorCameraStatus::NotRequested);

        assert!(status.motion_vector_available);
        assert!(!status.motion_vector_camera_available);
        assert!(!status.motion_vector_object_available);
        assert!(!status.motion_vector_tile_max_available);
        assert!(!status.motion_vector_tile_max_coarse_available);
        assert!(!status.motion_vector_neighbor_max_available);
        assert!(!status.motion_vector_prepass_available);
    }

    #[test]
    fn effect_stack_resource_status_detects_split_motion_blur_node_motion_vectors() {
        let graph = PostProcessPassGraph::from_ordered_nodes(
            vec![
                PostProcessPassNode::new("motion-blur", PostProcessEffectKind::MotionBlur)
                    .with_required_inputs(vec![
                        PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string(),
                    ]),
                PostProcessPassNode::new("uber", PostProcessEffectKind::Uber),
            ],
            Vec::new(),
            None,
        );
        let executed = vec![
            "temporal.velocity-camera".to_string(),
            "temporal.velocity-object".to_string(),
            "post.motion-vector-tile-max".to_string(),
            "post.motion-vector-tile-max-coarse".to_string(),
            "post.motion-vector-neighbor-max".to_string(),
        ];

        let status =
            effect_stack_resource_status(&graph, &executed, MotionVectorCameraStatus::Ready);

        assert!(status.motion_vector_available);
        assert!(status.motion_vector_prepass_available);
    }

    #[test]
    fn effect_stack_resource_status_detects_executed_motion_vector_prepass_chain() {
        let graph = effect_stack_graph(vec![
            PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string(),
        ]);
        let executed = vec![
            "temporal.velocity-camera".to_string(),
            "temporal.velocity-object".to_string(),
            "post.motion-vector-tile-max".to_string(),
            "post.motion-vector-tile-max-coarse".to_string(),
            "post.motion-vector-neighbor-max".to_string(),
        ];

        let status =
            effect_stack_resource_status(&graph, &executed, MotionVectorCameraStatus::Ready);

        assert!(status.motion_vector_available);
        assert!(status.motion_vector_camera_available);
        assert!(status.motion_vector_object_available);
        assert!(status.motion_vector_tile_max_available);
        assert!(status.motion_vector_tile_max_coarse_available);
        assert!(status.motion_vector_neighbor_max_available);
        assert_eq!(
            status.motion_vector_camera_status,
            MotionVectorCameraStatus::Ready
        );
        assert!(status.motion_vector_prepass_available);
    }

    #[test]
    fn effect_stack_resource_status_keeps_prepass_unavailable_without_object_vectors() {
        let graph = effect_stack_graph(vec![
            PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string(),
        ]);
        let executed = vec![
            "temporal.velocity-camera".to_string(),
            "post.motion-vector-tile-max".to_string(),
            "post.motion-vector-tile-max-coarse".to_string(),
            "post.motion-vector-neighbor-max".to_string(),
        ];

        let status =
            effect_stack_resource_status(&graph, &executed, MotionVectorCameraStatus::Ready);

        assert!(status.motion_vector_available);
        assert!(status.motion_vector_camera_available);
        assert!(!status.motion_vector_object_available);
        assert!(status.motion_vector_tile_max_available);
        assert!(status.motion_vector_tile_max_coarse_available);
        assert!(status.motion_vector_neighbor_max_available);
        assert!(!status.motion_vector_prepass_available);
    }

    #[test]
    fn effect_stack_resource_status_keeps_prepass_unavailable_when_camera_vectors_are_cut() {
        let graph = effect_stack_graph(vec![
            PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string(),
        ]);
        let executed = vec![
            "temporal.velocity-camera".to_string(),
            "temporal.velocity-object".to_string(),
            "post.motion-vector-tile-max".to_string(),
            "post.motion-vector-tile-max-coarse".to_string(),
            "post.motion-vector-neighbor-max".to_string(),
        ];

        let status = effect_stack_resource_status(
            &graph,
            &executed,
            MotionVectorCameraStatus::CameraCutOrInvalid,
        );

        assert!(status.motion_vector_camera_available);
        assert!(status.motion_vector_object_available);
        assert_eq!(
            status.motion_vector_camera_status,
            MotionVectorCameraStatus::CameraCutOrInvalid
        );
        assert!(!status.motion_vector_prepass_available);
    }

    #[test]
    fn particle_velocity_gap_counts_sprites_only_when_reconstructed_velocity_is_requested() {
        let mut effect_stack = RenderPostProcessEffectStackSettings::default();
        let executed = vec!["particle.transparent".to_string()];

        assert_eq!(
            particle_velocity_missing_sprite_count(effect_stack, &executed, 3, 0),
            0
        );

        effect_stack.motion_blur.shutter_angle = 0.5;

        assert_eq!(
            particle_velocity_missing_sprite_count(effect_stack, &executed, 3, 0),
            3
        );
        assert_eq!(
            particle_velocity_missing_sprite_count(effect_stack, &[], 3, 0),
            0
        );
        assert_eq!(
            particle_velocity_missing_sprite_count(effect_stack, &executed, 0, 0),
            0
        );
    }

    #[test]
    fn particle_velocity_gap_excludes_sprites_with_previous_state() {
        let mut effect_stack = RenderPostProcessEffectStackSettings::default();
        effect_stack.motion_blur.shutter_angle = 0.5;
        let executed = vec!["particle.transparent".to_string()];

        assert_eq!(
            particle_velocity_missing_sprite_count(effect_stack, &executed, 3, 2),
            1
        );
        assert_eq!(
            particle_velocity_missing_sprite_count(effect_stack, &executed, 3, 8),
            0
        );
    }

    #[test]
    fn particle_velocity_anonymous_stream_ambiguity_requires_velocity_diagnostics() {
        let mut effect_stack = RenderPostProcessEffectStackSettings::default();
        let executed = vec!["particle.transparent".to_string()];

        assert_eq!(
            particle_velocity_anonymous_stream_ambiguity_count(effect_stack, &executed, 2, 2),
            0
        );

        effect_stack.motion_blur.shutter_angle = 0.5;

        assert_eq!(
            particle_velocity_anonymous_stream_ambiguity_count(effect_stack, &executed, 2, 2),
            2
        );
        assert_eq!(
            particle_velocity_anonymous_stream_ambiguity_count(effect_stack, &[], 2, 2),
            0
        );
        assert_eq!(
            particle_velocity_anonymous_stream_ambiguity_count(effect_stack, &executed, 0, 2),
            0
        );
    }
}
