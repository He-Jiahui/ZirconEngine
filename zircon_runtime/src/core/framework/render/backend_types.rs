mod ambient_occlusion;
mod backend_status;
mod camera_target;
mod capability;
mod command;
mod graph_reports;
mod handles;
mod history;
mod quality;

#[cfg(test)]
mod tests;

pub use ambient_occlusion::{
    RenderAmbientOcclusionExecutionFailureFlags, RenderAmbientOcclusionExecutionReport,
    RenderAmbientOcclusionExecutionStatus,
};
pub use backend_status::{
    GraphicsDebuggerStatus, RenderDeviceDiagnostics, RenderDeviceLimitDiagnostics,
    RenderingBackendInfo,
};
pub use camera_target::{
    RenderCameraTargetGraphImportReport, RenderCameraTargetGraphImportStatus,
    RenderCameraTargetResolutionReport, RenderCameraTargetWritebackReport,
    RenderCameraTargetWritebackStatus,
};
pub use capability::{
    RenderCapabilityClass, RenderCapabilityClassReport, RenderCapabilityKind,
    RenderCapabilityMismatchDetail, RenderCapabilitySummary, RenderQueueCapability,
};
pub use command::{
    RenderCommand, RenderHybridGiPayloadSource, RenderQuery, RenderViewportDescriptor,
    RenderVirtualGeometryPayloadSource,
};
pub use graph_reports::{
    MotionVectorCameraStatus, RenderGraphExecutionAccessBindingReport,
    RenderGraphExecutionAliasRecord, RenderGraphExecutionAliasReport,
    RenderGraphExecutionBatchReport, RenderGraphExecutionCoverageReport,
    RenderGraphExecutionProfileReport, RenderGraphExecutionResourceReport,
    RenderGraphMaterializationReport, RenderGraphParallelRecordingReport,
    RenderGraphPassProfileMetrics, RenderGraphPassProfileRecord, RenderGraphStageExecutionReport,
    RenderGraphTransientPoolReport, RenderSceneVelocityReadbackReport,
};
pub use handles::{FrameHistoryHandle, RenderPipelineHandle, RenderViewportHandle};
pub(crate) use history::RenderFrameHistoryInput;
pub use history::{
    FrameHistoryInvalidationReason, FrameHistoryStatus, RenderHistoryCopyReport,
    RenderHistoryDomain, RenderHistoryDomainResetReason, RenderHistoryDomainStatus,
    RenderHistoryDomainsReport,
};
pub(crate) use quality::normalize_texture_max_anisotropy;
pub use quality::{
    RenderFeatureQualitySettings, RenderQualityProfile, DEFAULT_HALF_RES_TRANSPARENCY_DEPTH_SIGMA,
};

use std::sync::Arc;

use crate::core::math::UVec2;

use super::{
    AdvancedProviderAvailability, AdvancedProviderReport, AntiAliasFallbackReport,
    RenderColorLutReadbackReport, RenderExposureReadbackReport, RenderHybridGiResolvedSettings,
    RenderPostProcessEffectStackReport, RenderShadowExecutionReport,
    RenderVirtualGeometryClusterSelectionInputSource,
    RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryNodeAndClusterCullSource, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometryVisBuffer64Source, SolariRuntimeReport,
    RENDER_HYBRID_GI_RADIANCE_CACHE_GPU_STAGE_COUNT,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderGpuSceneUploadPath {
    #[default]
    DirectQueueWrite,
    StagingCopy,
}

impl RenderGpuSceneUploadPath {
    pub const fn label(self) -> &'static str {
        match self {
            Self::DirectQueueWrite => "direct_queue_write",
            Self::StagingCopy => "staging_copy",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderReflectionProbeWorkloadReport {
    /// Probes present in the frame extract, including disabled and non-resident entries.
    pub extracted_probe_count: usize,
    /// Extracted probes intersecting the camera layers, before cubemap and intensity gates.
    pub camera_layer_candidate_count: usize,
    /// Eligible probes that entered asset and slot resolution.
    pub attempted_candidate_count: usize,
    /// Resident probes published to the shader for this frame.
    pub active_probe_count: usize,
    /// Eligible probes not attempted because the fixed probe capacity was already filled.
    pub capacity_dropped_candidate_count: usize,
    /// New cubemap payloads scheduled in the frame upload batch.
    pub scheduled_cubemap_upload_count: usize,
    /// Bytes in newly scheduled cubemap payloads; this does not imply queue submission.
    pub scheduled_cubemap_upload_bytes: u64,
    /// Native texture writes scheduled by the probe upload owner, one per PMREM mip.
    pub scheduled_texture_write_count: usize,
    /// Synchronous texture-asset load calls made after a probe slot miss.
    pub asset_load_call_count: usize,
    /// CPU microseconds spent in those synchronous texture-asset load calls.
    pub asset_load_cpu_time_us: u64,
    /// Attempted cubemaps rejected by asset lookup or PMREM validation.
    pub rejected_cubemap_count: usize,
    /// Upper bound for the current full-resolution shader loop, not measured shaded fragments.
    pub full_resolution_fragment_probe_visit_upper_bound: u64,
}

impl RenderReflectionProbeWorkloadReport {
    pub fn with_render_size(mut self, render_size: UVec2) -> Self {
        let pixel_count = u64::from(render_size.x).saturating_mul(u64::from(render_size.y));
        let active_probe_count = u64::try_from(self.active_probe_count).unwrap_or(u64::MAX);
        self.full_resolution_fragment_probe_visit_upper_bound =
            pixel_count.saturating_mul(active_probe_count);
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderStats {
    pub active_viewports: usize,
    pub submitted_frames: u64,
    pub captured_frames: u64,
    pub last_generation: Option<u64>,
    pub last_ambient_occlusion_execution_report: RenderAmbientOcclusionExecutionReport,
    pub last_scene_submission_completion_report: super::RenderSceneSubmissionCompletionReport,
    pub last_frame_submission_receipt: Option<super::RenderFrameSubmissionReceipt>,
    pub last_pipeline: Option<RenderPipelineHandle>,
    pub last_frame_target_size: Option<UVec2>,
    pub last_frame_render_size: Option<UVec2>,
    pub last_frame_history: Option<FrameHistoryHandle>,
    pub last_frame_history_status: FrameHistoryStatus,
    pub last_frame_history_copy_report: RenderHistoryCopyReport,
    pub last_frame_history_domains_report: RenderHistoryDomainsReport,
    pub last_camera_target_resolution: RenderCameraTargetResolutionReport,
    pub last_camera_target_graph_import: RenderCameraTargetGraphImportReport,
    pub last_camera_target_writeback: RenderCameraTargetWritebackReport,
    pub last_capture_report: super::RenderCaptureReport,
    /// Current-frame CPU/graph profile; it always shares a generation with the flat `last_*` data.
    pub last_frame_profile: Arc<super::RenderFrameProfile>,
    /// Most recently resolved asynchronous GPU timestamp profile, which can lag the current frame.
    pub last_resolved_gpu_frame_profile: Option<Arc<super::RenderFrameProfile>>,
    pub last_budget_warning_count: u32,
    pub last_store_lint_count: u32,
    pub last_pipeline_async_pending_count: u32,
    pub last_variant_first_frame_miss_count: u32,
    pub last_readback_in_flight_count: usize,
    pub last_readback_bytes: u64,
    pub last_readback_completed_count: usize,
    pub last_readback_completed_bytes: u64,
    pub last_readback_slot_reuse_rejection_count: u32,
    pub last_camera_loop_submission_count: usize,
    pub last_scene_camera_scheduled_count: usize,
    pub last_scene_camera_order_ambiguity_count: usize,
    pub last_visibility_view_count: usize,
    pub last_visibility_input_count: usize,
    pub last_visibility_layer_filtered_count: usize,
    pub last_visibility_frustum_culled_count: usize,
    pub last_visibility_occlusion_culled_count: usize,
    pub last_visibility_visible_count: usize,
    pub last_visibility_static_index_full_rebuild_count: usize,
    pub last_visibility_static_index_incremental_update_count: usize,
    pub last_visibility_static_index_inserted_count: usize,
    pub last_visibility_static_index_updated_count: usize,
    pub last_visibility_static_index_removed_count: usize,
    pub last_visibility_static_index_indexed_entity_count: usize,
    pub last_visibility_static_index_occupied_cell_count: usize,
    pub last_visibility_static_index_main_view_prefilter_used: bool,
    pub last_visibility_static_index_main_view_static_input_count: usize,
    pub last_visibility_static_index_main_view_static_candidate_count: usize,
    pub last_hzb_mip_count: usize,
    pub last_hzb_graph_executed_pass_count: usize,
    pub last_hzb_occlusion_reported: bool,
    pub last_hzb_occlusion_candidate_arg_count: usize,
    pub last_hzb_occlusion_candidate_instance_count: usize,
    pub last_hzb_occlusion_dispatch_group_count: usize,
    pub last_hzb_occlusion_dispatched_phase_count: usize,
    pub last_hzb_occlusion_params_buffer_create_count: usize,
    pub last_hzb_occlusion_params_upload_byte_count: u64,
    pub last_hzb_occlusion_bind_group_create_count: usize,
    pub last_hzb_occlusion_history_available: bool,
    pub last_hzb_occlusion_readback_available: bool,
    /// Source frame for the asynchronously completed HZB stats readback.
    pub last_hzb_occlusion_readback_source_frame_index: Option<u64>,
    /// Number of HZB stats requests that remain pending in the bounded async queue.
    pub last_hzb_occlusion_readback_pending_count: usize,
    /// Lifetime count of HZB stats requests dropped by the bounded async queue.
    pub last_hzb_occlusion_readback_dropped_count: usize,
    /// Age of the oldest pending HZB stats request when the current frame is known.
    pub last_hzb_occlusion_readback_oldest_pending_age_frames: Option<u64>,
    pub last_hzb_occlusion_tested_arg_count: usize,
    pub last_hzb_occlusion_tested_instance_count: usize,
    pub last_hzb_occlusion_culled_arg_count: usize,
    pub last_hzb_occlusion_culled_instance_count: usize,
    pub last_hzb_occlusion_indirect_args_readback_available: bool,
    /// Source frame for the asynchronously completed HZB indirect-args diagnostic.
    pub last_hzb_occlusion_indirect_args_readback_source_frame_index: Option<u64>,
    pub last_hzb_occlusion_readback_arg_count: usize,
    pub last_hzb_occlusion_compacted_draw_count: usize,
    pub last_hzb_occlusion_zero_instance_arg_count: usize,
    pub last_hzb_occlusion_remaining_instance_count: usize,
    pub last_light_grid_reported: bool,
    pub last_light_grid_light_count: usize,
    pub last_light_grid_tile_count: usize,
    pub last_light_grid_zbin_count: usize,
    pub last_light_grid_non_empty_tile_count: usize,
    pub last_light_grid_non_empty_zbin_count: usize,
    pub last_light_grid_non_empty_cluster_count: usize,
    pub last_light_grid_peak_lights_per_cluster: usize,
    pub last_light_grid_average_lights_per_cluster_milli: usize,
    pub last_reflection_probe_workload: RenderReflectionProbeWorkloadReport,
    pub last_quality_profile: Option<String>,
    pub last_effective_features: Vec<String>,
    pub last_graph_pass_count: usize,
    pub last_graph_culled_pass_count: usize,
    pub last_graph_queue_fallback_pass_count: usize,
    pub last_graph_resource_lifetime_count: usize,
    pub last_graph_sparse_texture_lifetime_count: usize,
    pub last_graph_planned_resource_access_count: usize,
    pub last_graph_planned_dependency_count: usize,
    pub last_graph_transient_texture_slot_count: usize,
    pub last_graph_sparse_texture_slot_count: usize,
    pub last_graph_transient_buffer_slot_count: usize,
    pub last_graph_transient_texture_bytes_reserved: u64,
    pub last_graph_transient_buffer_bytes_reserved: u64,
    pub last_graph_transient_dense_bytes_reserved: u64,
    pub last_graph_sparse_texture_virtual_bytes: u64,
    pub last_graph_compiled_cache_hit_count: usize,
    pub last_graph_compiled_cache_miss_count: usize,
    pub last_graph_compiled_cache_eviction_count: usize,
    pub last_graph_compiled_cache_entry_count: usize,
    pub last_graph_executed_pass_count: usize,
    pub last_graph_executed_passes: Vec<String>,
    pub last_graph_executed_executor_ids: Vec<String>,
    pub last_graph_executed_debug_markers: Vec<String>,
    pub last_graph_executed_resource_access_count: usize,
    pub last_graph_executed_dependency_count: usize,
    pub last_graph_compute_dispatch_count: usize,
    pub last_graph_compute_dispatch_group_count: usize,
    pub last_graph_compute_storage_write_resource_count: usize,
    pub last_graph_compute_planned_workload_count: usize,
    pub last_graph_compute_matched_workload_count: usize,
    pub last_graph_compute_missing_dispatch_count: usize,
    pub last_graph_compute_workload_mismatch_count: usize,
    pub last_graph_compute_unexpected_dispatch_count: usize,
    pub last_volumetric_fog_compute_dispatch_count: usize,
    pub last_volumetric_fog_compute_dispatch_group_count: usize,
    pub last_volumetric_fog_uploaded_bytes: u64,
    pub last_graph_execution_resource_report: RenderGraphExecutionResourceReport,
    pub last_graph_materialization_report: RenderGraphMaterializationReport,
    pub last_graph_execution_alias_report: RenderGraphExecutionAliasReport,
    pub last_graph_execution_coverage_report: RenderGraphExecutionCoverageReport,
    pub last_graph_execution_profile_report: RenderGraphExecutionProfileReport,
    pub last_graph_execution_batch_report: RenderGraphExecutionBatchReport,
    pub last_graph_parallel_recording_report: RenderGraphParallelRecordingReport,
    pub last_graph_stage_execution_report: RenderGraphStageExecutionReport,
    pub last_scene_velocity_readback_report: RenderSceneVelocityReadbackReport,
    pub last_exposure_readback_report: RenderExposureReadbackReport,
    pub last_color_lut_readback_report: RenderColorLutReadbackReport,
    pub last_post_process_graph_node_count: usize,
    pub last_post_process_graph_skipped_node_count: usize,
    pub last_post_process_output_transfer_node: Option<String>,
    pub last_post_process_graph_executed_nodes: Vec<String>,
    pub last_post_process_effect_stack_report: RenderPostProcessEffectStackReport,
    pub last_post_process_lut_request_count: usize,
    pub last_post_process_lut_ready_count: usize,
    pub last_post_process_lut_fallback_count: usize,
    pub last_post_process_lut_2d_strip_ready_count: usize,
    pub last_post_process_lut_3d_request_count: usize,
    pub last_post_process_lut_unsupported_shape_count: usize,
    pub last_motion_vector_camera_status: MotionVectorCameraStatus,
    pub last_anti_alias_fallback: AntiAliasFallbackReport,
    pub last_graph_requested_msaa_sample_count: u32,
    pub last_graph_effective_msaa_sample_count: u32,
    pub last_anti_alias_graph_executed_pass_count: usize,
    /// Actual WGPU TAA reactive-mask passes, excluding the retained logical node for empty streams.
    pub last_taa_reactive_mask_encoded_pass_count: usize,
    /// R8 bytes written by actual TAA reactive-mask passes in the latest frame.
    pub last_taa_reactive_mask_encoded_write_bytes: u64,
    /// Number of TAA resolve bind groups created before a stable resource-generation cache exists.
    pub last_taa_resolve_bind_group_create_count: usize,
    pub last_virtual_geometry_graph_executed_pass_count: usize,
    pub last_hybrid_gi_graph_executed_pass_count: usize,
    pub last_particle_graph_executed_pass_count: usize,
    pub last_shadow_graph_executed_pass_count: usize,
    pub last_shadow_execution_report: RenderShadowExecutionReport,
    pub last_transparent_graph_executed_pass_count: usize,
    pub last_particle_velocity_missing_sprite_count: usize,
    pub last_particle_velocity_anonymous_stream_ambiguity_count: usize,
    pub last_particle_gpu_alive_count: usize,
    pub last_particle_gpu_spawned_total: usize,
    pub last_particle_gpu_emitter_readback_count: usize,
    pub last_particle_gpu_indirect_instance_count: usize,
    pub last_async_compute_pass_count: usize,
    pub last_ui_command_count: usize,
    pub last_ui_quad_count: usize,
    pub last_ui_text_payload_count: usize,
    pub last_ui_text_glyph_count: usize,
    pub last_ui_text_unmapped_glyph_count: usize,
    pub last_ui_text_visible_raster_glyph_count: usize,
    pub last_ui_text_raster_source_image_count: usize,
    /// Unique persistent physical raster keys bound to live native source-cache entries.
    pub last_ui_text_raster_persistent_key_count: usize,
    /// Exact native bitmap-source cache misses observed while preparing the last UI frame.
    pub last_ui_text_raster_source_cache_miss_count: usize,
    pub last_ui_text_missing_raster_image_count: usize,
    pub last_ui_text_visible_missing_raster_image_count: usize,
    pub last_ui_text_visible_raster_placeholder_count: usize,
    pub last_ui_text_raster_worker_pending_count: usize,
    pub last_ui_text_raster_worker_failed_count: usize,
    pub last_ui_text_raster_renderer_upload_requeued_count: usize,
    pub last_ui_text_raster_renderer_upload_failure_count: usize,
    /// SDF/MSDF generation batches still owned by the bounded text scheduler.
    pub last_ui_text_sdf_generation_pending_batch_count: usize,
    /// Completed SDF/MSDF batches not yet applied to the prepared atlas.
    pub last_ui_text_sdf_generation_completion_backlog_count: usize,
    /// Current-frame SDF/MSDF glyph generation failures, including deferred work.
    pub last_ui_text_sdf_generation_failure_count: usize,
    pub last_ui_text_layout_fallback_count: u64,
    pub last_ui_text_invalid_font_size_count: u64,
    pub last_ui_text_invalid_language_count: u64,
    pub last_ui_text_other_layout_error_count: u64,
    pub last_ui_image_payload_count: usize,
    pub last_ui_clipped_command_count: usize,
    pub last_ui_graph_executed_pass_count: usize,
    pub last_ui_target_size: Option<UVec2>,
    pub last_ui_graph_pass_order: Option<String>,
    pub last_material_count: usize,
    pub last_material_ready_count: usize,
    pub last_material_fallback_count: usize,
    pub last_material_validation_error_count: usize,
    pub last_material_diagnostic_count: usize,
    pub last_shader_variant_miss_report: super::ShaderVariantMissReport,
    pub last_mesh_draw_count: usize,
    pub last_mesh_opaque_draw_count: usize,
    pub last_mesh_alpha_mask_draw_count: usize,
    pub last_mesh_transparent_draw_count: usize,
    pub last_mesh_early_z_draw_count: usize,
    pub last_mesh_shadow_caster_draw_count: usize,
    pub last_mesh_alpha_mask_shadow_caster_draw_count: usize,
    pub last_mesh_prepared_geometry_draw_count: usize,
    pub last_mesh_dynamic_geometry_draw_count: usize,
    pub last_mesh_gpu_morphed_source_draw_count: usize,
    pub last_mesh_gpu_skinned_morphed_source_draw_count: usize,
    pub last_mesh_skinned_draw_count: usize,
    pub last_mesh_skinned_palette_upload_count: usize,
    pub last_mesh_skinned_previous_palette_upload_count: usize,
    pub last_mesh_skinned_gpu_source_candidate_count: usize,
    pub last_mesh_skinned_gpu_cpu_morphed_source_candidate_count: usize,
    pub last_mesh_skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count: usize,
    pub last_mesh_skinned_gpu_skinning_draw_count: usize,
    pub last_mesh_skinned_gpu_velocity_draw_count: usize,
    pub last_mesh_indirect_draw_count: usize,
    pub last_mesh_lod_draw_count: usize,
    pub last_mesh_previous_velocity_transform_draw_count: usize,
    pub last_mesh_missing_velocity_transform_draw_count: usize,
    pub last_mesh_taa_reactive_mask_command_count: usize,
    pub last_mesh_static_batch_candidate_group_count: usize,
    pub last_mesh_static_batch_candidate_draw_count: usize,
    pub last_mesh_dynamic_batch_candidate_group_count: usize,
    pub last_mesh_dynamic_batch_candidate_draw_count: usize,
    pub last_mesh_gpu_instancing_candidate_group_count: usize,
    pub last_mesh_gpu_instancing_candidate_draw_count: usize,
    pub last_mesh_command_count: usize,
    /// Commands routed through the ordinary opaque path.
    pub last_mesh_opaque_command_count: usize,
    /// Commands routed through the late forward path for opaque advanced PBR materials.
    pub last_mesh_advanced_pbr_opaque_command_count: usize,
    pub last_mesh_cached_command_hit_count: usize,
    pub last_mesh_command_rebuild_count: usize,
    pub last_mesh_dynamic_command_count: usize,
    pub last_mesh_pending_static_command_cache_draw_candidate_count: usize,
    pub last_mesh_pending_static_command_cache_phase_candidate_count: usize,
    pub last_mesh_pending_static_command_cache_depth_prepass_candidate_count: usize,
    pub last_mesh_pending_static_command_cache_shadow_candidate_count: usize,
    pub last_mesh_pending_static_command_cache_opaque_candidate_count: usize,
    pub last_mesh_pending_static_command_cache_alpha_mask_candidate_count: usize,
    pub last_mesh_pre_mesh_draw_static_command_cache_skipped_draw_count: usize,
    pub last_mesh_pre_mesh_draw_static_command_cache_skipped_phase_count: usize,
    pub last_mesh_pre_mesh_draw_static_command_cache_visibility_pruned_draw_count: usize,
    pub last_mesh_pre_mesh_draw_static_command_cache_residual_material_phase_draw_count: usize,
    pub last_mesh_pre_mesh_draw_static_command_cache_residual_rebuild_input_missing_draw_count:
        usize,
    pub last_mesh_pre_mesh_draw_static_command_cache_residual_rebuild_rejected_draw_count: usize,
    pub last_mesh_command_cache_miss_count: usize,
    pub last_mesh_command_cache_invalidated_transform_count: usize,
    pub last_mesh_command_cache_invalidated_geometry_count: usize,
    pub last_mesh_command_cache_invalidated_material_count: usize,
    pub last_mesh_replay_state_change_count: usize,
    pub last_mesh_replay_bind_skip_count: usize,
    /// Actual group-2 material `set_bind_group` calls encoded during mesh replay.
    pub last_mesh_replay_material_bind_group_set_count: usize,
    /// Group-2 material bind requests skipped because the currently bound table matched.
    pub last_mesh_replay_material_bind_group_skip_count: usize,
    pub last_indirect_batch_count: usize,
    pub last_indirect_batched_draw_count: usize,
    pub last_indirect_fallback_draw_count: usize,
    pub last_indirect_args_count: usize,
    pub last_indirect_workspace_created_buffer_count: usize,
    pub last_indirect_workspace_uploaded_byte_count: u64,
    pub last_indirect_workspace_upload_range_count: usize,
    pub last_gpu_scene_primitive_count: u32,
    pub last_gpu_scene_instance_count: u32,
    pub last_gpu_scene_dirty_entry_count: usize,
    pub last_gpu_scene_uploaded_bytes: u64,
    pub last_gpu_scene_upload_path: RenderGpuSceneUploadPath,
    pub last_gpu_scene_free_span_count: usize,
    pub last_gpu_scene_primitive_upload_range_count: usize,
    pub last_gpu_scene_instance_upload_range_count: usize,
    pub last_sprite_count: usize,
    pub last_sprite_ready_count: usize,
    pub last_sprite_texture_fallback_count: usize,
    pub last_sprite_graph_executed_pass_count: usize,
    pub last_sprite_draw_batch_count: usize,
    pub last_sprite_batched_sprite_count: usize,
    pub last_sprite_image_slice_count: usize,
    pub last_sprite_expanded_image_slice_count: usize,
    pub last_sprite_vertex_count: usize,
    pub last_sprite_opaque_draw_batch_count: usize,
    pub last_sprite_alpha_mask_draw_batch_count: usize,
    pub last_sprite_transparent_draw_batch_count: usize,
    pub last_directional_light_count: usize,
    pub last_directional_light_ready_count: usize,
    pub last_directional_light_degraded_count: usize,
    pub last_point_light_count: usize,
    pub last_point_light_ready_count: usize,
    pub last_point_light_degraded_count: usize,
    pub last_spot_light_count: usize,
    pub last_spot_light_ready_count: usize,
    pub last_spot_light_degraded_count: usize,
    pub last_ambient_light_count: usize,
    pub last_ambient_light_ready_count: usize,
    pub last_ambient_light_degraded_count: usize,
    pub last_rect_light_count: usize,
    pub last_rect_light_ready_count: usize,
    pub last_rect_light_degraded_count: usize,
    pub last_virtual_geometry_cluster_budget: usize,
    pub last_virtual_geometry_page_budget: usize,
    pub last_virtual_geometry_input_cluster_count: usize,
    pub last_virtual_geometry_input_page_count: usize,
    pub last_virtual_geometry_visible_cluster_count: usize,
    pub last_virtual_geometry_visible_entity_count: usize,
    pub last_virtual_geometry_instance_count: usize,
    pub last_virtual_geometry_payload_source: RenderVirtualGeometryPayloadSource,
    pub last_virtual_geometry_requested_page_count: usize,
    pub last_virtual_geometry_dirty_page_count: usize,
    pub last_virtual_geometry_forced_mip: Option<u8>,
    pub last_virtual_geometry_freeze_cull: bool,
    pub last_virtual_geometry_visualize_bvh: bool,
    pub last_virtual_geometry_visualize_visbuffer: bool,
    pub last_virtual_geometry_print_leaf_clusters: bool,
    pub last_virtual_geometry_page_table_entry_count: usize,
    pub last_virtual_geometry_resident_page_count: usize,
    pub last_virtual_geometry_pending_request_count: usize,
    pub last_virtual_geometry_page_dependency_count: usize,
    pub last_virtual_geometry_completed_page_count: usize,
    pub last_virtual_geometry_replaced_page_count: usize,
    pub last_virtual_geometry_indirect_draw_count: usize,
    pub last_virtual_geometry_indirect_buffer_count: usize,
    pub last_virtual_geometry_indirect_args_count: usize,
    pub last_virtual_geometry_indirect_segment_count: usize,
    pub last_virtual_geometry_execution_segment_count: usize,
    pub last_virtual_geometry_execution_page_count: usize,
    pub last_virtual_geometry_execution_resident_segment_count: usize,
    pub last_virtual_geometry_execution_pending_segment_count: usize,
    pub last_virtual_geometry_execution_missing_segment_count: usize,
    pub last_virtual_geometry_execution_repeated_draw_count: usize,
    pub last_virtual_geometry_cluster_selection_input_source:
        RenderVirtualGeometryClusterSelectionInputSource,
    pub last_virtual_geometry_node_and_cluster_cull_source:
        RenderVirtualGeometryNodeAndClusterCullSource,
    pub last_virtual_geometry_node_and_cluster_cull_record_count: usize,
    pub last_virtual_geometry_node_and_cluster_cull_dispatch_group_count: [usize; 3],
    pub last_virtual_geometry_node_and_cluster_cull_instance_seed_count: usize,
    pub last_virtual_geometry_node_and_cluster_cull_instance_work_item_count: usize,
    pub last_virtual_geometry_node_and_cluster_cull_cluster_work_item_count: usize,
    pub last_virtual_geometry_node_and_cluster_cull_hierarchy_child_id_count: usize,
    pub last_virtual_geometry_node_and_cluster_cull_child_work_item_count: usize,
    pub last_virtual_geometry_node_and_cluster_cull_traversal_record_count: usize,
    pub last_virtual_geometry_node_and_cluster_cull_page_request_count: usize,
    pub last_virtual_geometry_selected_cluster_source: RenderVirtualGeometrySelectedClusterSource,
    pub last_virtual_geometry_selected_cluster_count: usize,
    pub last_virtual_geometry_visbuffer64_source: RenderVirtualGeometryVisBuffer64Source,
    pub last_virtual_geometry_visbuffer64_entry_count: usize,
    pub last_virtual_geometry_hardware_rasterization_source:
        RenderVirtualGeometryHardwareRasterizationSource,
    pub last_virtual_geometry_hardware_rasterization_record_count: usize,
    pub last_hybrid_gi_active_probe_count: usize,
    pub last_hybrid_gi_requested_probe_count: usize,
    pub last_hybrid_gi_dirty_probe_count: usize,
    pub last_hybrid_gi_cache_entry_count: usize,
    pub last_hybrid_gi_resident_probe_count: usize,
    pub last_hybrid_gi_pending_update_count: usize,
    pub last_hybrid_gi_scheduled_trace_region_count: usize,
    pub last_hybrid_gi_scene_card_count: usize,
    pub last_hybrid_gi_scene_screen_probe_count: usize,
    pub last_hybrid_gi_scene_radiance_cache_entry_count: usize,
    pub last_hybrid_gi_radiance_cache_resident_probe_count: usize,
    pub last_hybrid_gi_radiance_cache_update_probe_count: usize,
    pub last_hybrid_gi_radiance_cache_truncated_demand_count: usize,
    pub last_hybrid_gi_radiance_cache_generation: u64,
    pub last_hybrid_gi_radiance_cache_scroll_count: u64,
    pub last_hybrid_gi_radiance_cache_history_clear_count: u64,
    /// GPU-authored item counts for the five update stages, followed by committed consumes.
    pub last_hybrid_gi_radiance_cache_gpu_stage_dispatch_counts:
        [u32; RENDER_HYBRID_GI_RADIANCE_CACHE_GPU_STAGE_COUNT],
    pub last_hybrid_gi_surface_cache_resident_page_count: usize,
    pub last_hybrid_gi_surface_cache_dirty_page_count: usize,
    pub last_hybrid_gi_surface_cache_feedback_card_count: usize,
    pub last_hybrid_gi_surface_cache_capture_slot_count: usize,
    pub last_hybrid_gi_surface_cache_invalidated_page_count: usize,
    pub last_hybrid_gi_surface_cache_depth_sample_count: usize,
    pub last_hybrid_gi_probe_trace_tile_count: usize,
    pub last_hybrid_gi_probe_trace_dispatch_group_count: [usize; 3],
    pub last_hybrid_gi_voxel_resident_clipmap_count: usize,
    pub last_hybrid_gi_voxel_dirty_clipmap_count: usize,
    pub last_hybrid_gi_voxel_invalidated_clipmap_count: usize,
    pub last_hybrid_gi_global_sdf_cpu_prepare_time_us: u64,
    pub last_hybrid_gi_global_sdf_cpu_mesh_object_collection_time_us: u64,
    pub last_hybrid_gi_global_sdf_cpu_mesh_scene_sync_time_us: u64,
    pub last_hybrid_gi_global_sdf_cpu_residency_time_us: u64,
    pub last_hybrid_gi_global_sdf_cpu_influence_update_time_us: u64,
    pub last_hybrid_gi_global_sdf_cpu_candidate_build_time_us: u64,
    pub last_hybrid_gi_global_sdf_mesh_projection_cache_hit: bool,
    pub last_hybrid_gi_global_sdf_object_count: usize,
    pub last_hybrid_gi_global_sdf_resident_page_count: usize,
    pub last_hybrid_gi_global_sdf_sampleable_page_count: usize,
    pub last_hybrid_gi_global_sdf_dirty_page_count: usize,
    pub last_hybrid_gi_global_sdf_dispatched_page_count: usize,
    pub last_hybrid_gi_global_sdf_uploaded_page_count: usize,
    pub last_hybrid_gi_global_sdf_deferred_page_count: usize,
    pub last_hybrid_gi_global_sdf_candidate_overflow_page_count: usize,
    pub last_hybrid_gi_global_sdf_candidate_contributor_count: usize,
    pub last_hybrid_gi_global_sdf_clipmap_fallback_count: usize,
    pub last_hybrid_gi_global_sdf_candidate_bucket_capacity_bytes: u64,
    pub last_hybrid_gi_global_sdf_persistent_resource_byte_count: u64,
    pub last_hybrid_gi_global_sdf_transient_buffer_creation_count: usize,
    pub last_hybrid_gi_global_sdf_transient_bind_group_creation_count: usize,
    pub last_hybrid_gi_global_sdf_transient_parameter_upload_byte_count: u64,
    pub last_hybrid_gi_global_sdf_transient_page_upload_byte_count: u64,
    pub last_hybrid_gi_global_sdf_transient_mesh_upload_byte_count: u64,
    pub last_hybrid_gi_global_sdf_transient_completion_upload_byte_count: u64,
    pub last_hybrid_gi_global_sdf_transient_upload_byte_count: u64,
    pub last_hybrid_gi_payload_source: RenderHybridGiPayloadSource,
    pub last_hybrid_gi_resolved_settings: Option<RenderHybridGiResolvedSettings>,
    pub device_diagnostics: Option<RenderDeviceDiagnostics>,
    pub capabilities: RenderCapabilitySummary,
    pub advanced_provider_availability: AdvancedProviderAvailability,
    pub last_advanced_provider_reports: Vec<AdvancedProviderReport>,
    pub last_solari_runtime_report: SolariRuntimeReport,
}

#[cfg(test)]
mod reflection_probe_workload_tests {
    use super::RenderReflectionProbeWorkloadReport;
    use crate::core::math::UVec2;

    #[test]
    fn reflection_probe_workload_derives_full_resolution_visit_upper_bound() {
        let report = RenderReflectionProbeWorkloadReport {
            active_probe_count: 4,
            ..RenderReflectionProbeWorkloadReport::default()
        }
        .with_render_size(UVec2::new(1_920, 1_080));

        assert_eq!(
            report.full_resolution_fragment_probe_visit_upper_bound,
            8_294_400
        );
    }

    #[test]
    fn reflection_probe_workload_visit_upper_bound_saturates() {
        let report = RenderReflectionProbeWorkloadReport {
            active_probe_count: usize::MAX,
            ..RenderReflectionProbeWorkloadReport::default()
        }
        .with_render_size(UVec2::new(u32::MAX, u32::MAX));

        assert_eq!(
            report.full_resolution_fragment_probe_visit_upper_bound,
            u64::MAX
        );
    }
}
