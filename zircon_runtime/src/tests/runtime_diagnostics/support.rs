use std::sync::Arc;

use crate::core::framework::render::{
    AdvancedProviderAvailability, AdvancedProviderReport, AdvancedProviderStatus,
    AdvancedRenderDegradation, AdvancedRenderFeature, AntiAliasFallbackReason,
    AntiAliasFallbackReport, AntiAliasMode, CapturedFrame, FrameHistoryHandle,
    FrameHistoryInvalidationReason, FrameHistoryStatus, MotionVectorCameraStatus,
    RenderFrameExtract, RenderFramework, RenderFrameworkError, RenderGpuSceneUploadPath,
    RenderGraphExecutionAliasRecord, RenderGraphExecutionAliasReport,
    RenderGraphExecutionCoverageReport, RenderGraphExecutionProfileReport,
    RenderGraphExecutionResourceReport, RenderGraphMaterializationReport,
    RenderGraphPassProfileRecord, RenderGraphStageExecutionReport, RenderGraphTransientPoolReport,
    RenderHistoryCopyReport, RenderHybridGiPayloadSource, RenderPipelineHandle,
    RenderPostProcessEffectStackReport, RenderQualityProfile, RenderQueueCapability, RenderStats,
    RenderViewportDescriptor, RenderViewportHandle,
    RenderVirtualGeometryClusterSelectionInputSource, RenderVirtualGeometryDebugSnapshot,
    RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryNodeAndClusterCullSource, RenderVirtualGeometryPayloadSource,
    RenderVirtualGeometrySelectedClusterSource, RenderVirtualGeometryVisBuffer64Source,
    SolariRuntimeDegradation, SolariRuntimeReport, SolariRuntimeStatus, SolariSettings,
};
use crate::core::runtime::ServiceObject;
use crate::core::{ManagerDescriptor, ModuleDescriptor, RegistryName, StartupMode};
use crate::engine_module::factory;
use zircon_runtime_interface::ui::surface::UiRenderExtract;

pub(super) const DIAGNOSTICS_TEST_MODULE: &str = crate::graphics::GRAPHICS_MODULE_NAME;

pub(super) fn fake_render_module() -> ModuleDescriptor {
    ModuleDescriptor::new(
        DIAGNOSTICS_TEST_MODULE,
        "runtime diagnostics fake render services",
    )
    .with_manager(ManagerDescriptor::new(
        RegistryName::new(crate::core::manager::RENDER_FRAMEWORK_NAME).unwrap(),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| {
            Ok(
                Arc::new(crate::core::manager::RenderFrameworkHandle::new(Arc::new(
                    FakeRenderFramework,
                ))) as ServiceObject,
            )
        }),
    ))
}

struct FakeRenderFramework;

impl RenderFramework for FakeRenderFramework {
    fn create_viewport(
        &self,
        _descriptor: RenderViewportDescriptor,
    ) -> Result<RenderViewportHandle, RenderFrameworkError> {
        Ok(RenderViewportHandle::new(1))
    }

    fn destroy_viewport(
        &self,
        _viewport: RenderViewportHandle,
    ) -> Result<(), RenderFrameworkError> {
        Ok(())
    }

    fn submit_frame_extract(
        &self,
        _viewport: RenderViewportHandle,
        _extract: RenderFrameExtract,
    ) -> Result<(), RenderFrameworkError> {
        Ok(())
    }

    fn submit_frame_extract_with_ui(
        &self,
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
        _ui: Option<UiRenderExtract>,
    ) -> Result<(), RenderFrameworkError> {
        self.submit_frame_extract(viewport, extract)
    }

    fn set_pipeline_asset(
        &self,
        _viewport: RenderViewportHandle,
        _pipeline: RenderPipelineHandle,
    ) -> Result<(), RenderFrameworkError> {
        Ok(())
    }

    fn reload_pipeline(&self, _pipeline: RenderPipelineHandle) -> Result<(), RenderFrameworkError> {
        Ok(())
    }

    fn query_stats(&self) -> Result<RenderStats, RenderFrameworkError> {
        Ok(RenderStats {
            active_viewports: 2,
            submitted_frames: 7,
            last_frame_history_status: FrameHistoryStatus::new(
                Some(FrameHistoryHandle::new(41)),
                Some(FrameHistoryHandle::new(40)),
                false,
                Some(FrameHistoryInvalidationReason::RenderSizeChanged),
                crate::core::math::UVec2::new(1280, 720),
                crate::core::math::UVec2::new(960, 540),
            ),
            last_frame_history_copy_report: RenderHistoryCopyReport::new(
                true,
                crate::core::math::UVec2::new(960, 540),
                5,
                true,
                true,
                true,
                false,
                true,
                true,
                false,
            ),
            last_frame_target_size: Some(crate::core::math::UVec2::new(1280, 720)),
            last_frame_render_size: Some(crate::core::math::UVec2::new(960, 540)),
            last_camera_target_resolution:
                crate::core::framework::render::RenderCameraTargetResolutionReport::new(
                    crate::core::framework::render::RenderCameraTargetKind::Headless,
                    crate::core::math::UVec2::new(1280, 720),
                    crate::core::math::UVec2::new(640, 360),
                    crate::core::math::UVec2::new(320, 180),
                    crate::core::math::UVec2::new(160, 90),
                ),
            last_camera_target_graph_import:
                crate::core::framework::render::RenderCameraTargetGraphImportReport::not_requested(
                    crate::core::framework::render::RenderCameraTargetKind::Headless,
                ),
            last_camera_target_writeback:
                crate::core::framework::render::RenderCameraTargetWritebackReport::not_requested(
                    crate::core::framework::render::RenderCameraTargetKind::Headless,
                ),
            last_camera_loop_submission_count: 4,
            last_scene_camera_scheduled_count: 3,
            last_scene_camera_order_ambiguity_count: 1,
            last_visibility_view_count: 2,
            last_visibility_input_count: 8,
            last_visibility_layer_filtered_count: 1,
            last_visibility_frustum_culled_count: 3,
            last_visibility_occlusion_culled_count: 18,
            last_visibility_visible_count: 3,
            last_visibility_static_index_full_rebuild_count: 0,
            last_visibility_static_index_incremental_update_count: 1,
            last_visibility_static_index_inserted_count: 2,
            last_visibility_static_index_updated_count: 3,
            last_visibility_static_index_removed_count: 4,
            last_visibility_static_index_indexed_entity_count: 10,
            last_visibility_static_index_occupied_cell_count: 7,
            last_visibility_static_index_main_view_prefilter_used: true,
            last_visibility_static_index_main_view_static_input_count: 12,
            last_visibility_static_index_main_view_static_candidate_count: 5,
            last_hzb_mip_count: 10,
            last_hzb_graph_executed_pass_count: 1,
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
            last_light_grid_reported: true,
            last_light_grid_light_count: 9,
            last_light_grid_tile_count: 64,
            last_light_grid_zbin_count: 32,
            last_light_grid_non_empty_tile_count: 11,
            last_light_grid_non_empty_zbin_count: 7,
            last_light_grid_non_empty_cluster_count: 23,
            last_light_grid_peak_lights_per_cluster: 5,
            last_light_grid_average_lights_per_cluster_milli: 375,
            last_graph_pass_count: 18,
            last_graph_culled_pass_count: 4,
            last_graph_queue_fallback_pass_count: 2,
            last_graph_resource_lifetime_count: 6,
            last_graph_sparse_texture_lifetime_count: 1,
            last_graph_planned_resource_access_count: 22,
            last_graph_planned_dependency_count: 9,
            last_graph_transient_texture_slot_count: 3,
            last_graph_sparse_texture_slot_count: 1,
            last_graph_transient_buffer_slot_count: 2,
            last_graph_transient_texture_bytes_reserved: 4_194_304,
            last_graph_transient_buffer_bytes_reserved: 65_536,
            last_graph_transient_dense_bytes_reserved: 4_259_840,
            last_graph_sparse_texture_virtual_bytes: 16_777_216,
            last_graph_executed_pass_count: 14,
            last_graph_executed_debug_markers: (0..14)
                .map(|index| format!("zircon::RenderGraphPass::diagnostics-pass-{index}"))
                .collect(),
            last_graph_executed_resource_access_count: 19,
            last_graph_executed_dependency_count: 8,
            last_graph_compute_dispatch_count: 2,
            last_graph_compute_dispatch_group_count: 1234,
            last_graph_compute_storage_write_resource_count: 2,
            last_graph_compute_planned_workload_count: 2,
            last_graph_compute_matched_workload_count: 1,
            last_graph_compute_missing_dispatch_count: 1,
            last_graph_compute_workload_mismatch_count: 0,
            last_graph_compute_unexpected_dispatch_count: 0,
            last_volumetric_fog_compute_dispatch_count: 3,
            last_volumetric_fog_compute_dispatch_group_count: 44_400,
            last_volumetric_fog_uploaded_bytes: 624,
            last_graph_execution_resource_report: RenderGraphExecutionResourceReport::new(
                18, 14, 4, 3,
            )
            .with_transient_pool_report(
                RenderGraphTransientPoolReport::new(6, 5, 7, 2, 3, 4, 1, 8, 9)
                    .with_retained_bytes(4_096, 512)
                    .with_budget_bytes(1_048_576, 65_536)
                    .with_budget_evictions(10, 11),
            ),
            last_graph_materialization_report: RenderGraphMaterializationReport {
                required_texture_count: 4,
                bound_texture_count: 4,
                missing_texture_count: 0,
                required_buffer_count: 3,
                bound_buffer_count: 3,
                missing_buffer_count: 0,
                required_external_count: 2,
                bound_required_external_count: 2,
                missing_required_external_count: 0,
                report_only_external_count: 3,
                bound_report_only_external_count: 2,
                missing_report_only_external_count: 1,
                stale_texture_binding_count: 0,
                stale_buffer_binding_count: 0,
                sparse_texture_reservation_count: 1,
            },
            last_graph_execution_alias_report: RenderGraphExecutionAliasReport::new(
                vec![
                    RenderGraphExecutionAliasRecord::new("hzb-furthest", "hzb-furthest"),
                    RenderGraphExecutionAliasRecord::new(
                        "scene-color",
                        "rg-transient-texture-bucket-0123456789abcdef-slot-0",
                    ),
                    RenderGraphExecutionAliasRecord::new(
                        "scene-normal",
                        "rg-transient-texture-bucket-0123456789abcdef-slot-0",
                    ),
                ],
                vec![
                    RenderGraphExecutionAliasRecord::new("light-list", "light-list"),
                    RenderGraphExecutionAliasRecord::new(
                        "mesh.compacted-args",
                        "rg-transient-buffer-bucket-fedcba9876543210-slot-0",
                    ),
                    RenderGraphExecutionAliasRecord::new(
                        "mesh.indirect-args",
                        "rg-transient-buffer-bucket-fedcba9876543210-slot-0",
                    ),
                ],
            ),
            last_graph_execution_coverage_report: RenderGraphExecutionCoverageReport::new(
                14, 14, 14, 0, 0, 0,
            ),
            last_graph_execution_profile_report: RenderGraphExecutionProfileReport::new(vec![
                RenderGraphPassProfileRecord::new("depth-prepass", "mesh.depth", 150),
                RenderGraphPassProfileRecord::new("lighting", "lighting.light-grid", 275),
                RenderGraphPassProfileRecord::new("post-stack", "post.uber", 0),
            ]),
            last_graph_stage_execution_report: RenderGraphStageExecutionReport::new(14, 1, 7, 6, 0),
            last_post_process_graph_node_count: 5,
            last_post_process_graph_skipped_node_count: 1,
            last_post_process_output_transfer_node: Some("output-transfer".to_string()),
            last_post_process_graph_executed_nodes: vec![
                "bloom".to_string(),
                "uber".to_string(),
                "output-transfer".to_string(),
            ],
            last_post_process_effect_stack_report: RenderPostProcessEffectStackReport {
                enabled: true,
                active_family_count: 3,
                active_families: vec![
                    "vignette".to_string(),
                    "depth-of-field".to_string(),
                    "screen-space-reflection".to_string(),
                ],
                approximated_family_count: 2,
                approximated_families: vec![
                    "depth-of-field".to_string(),
                    "screen-space-reflection".to_string(),
                ],
                missing_resource_count: 1,
                missing_resources: vec!["effect-stack.ssr.normal".to_string()],
            },
            capabilities: crate::core::framework::render::RenderCapabilitySummary {
                backend_name: "diagnostics-test-renderer".to_string(),
                queue_classes: vec![
                    RenderQueueCapability::Graphics,
                    RenderQueueCapability::Compute,
                    RenderQueueCapability::Copy,
                ],
                supports_surface: true,
                supports_offscreen: true,
                supports_async_compute: true,
                supports_async_copy: false,
                supports_pipeline_cache: true,
                supports_storage_buffers: true,
                max_storage_buffers_per_shader_stage: 10,
                supports_indirect_draw: true,
                supports_buffer_readback: false,
                acceleration_structures_supported: false,
                inline_ray_query: true,
                ray_tracing_pipeline: false,
                supports_buffer_binding_array: true,
                supports_texture_binding_array: true,
                supports_non_uniform_resource_indexing: true,
                supports_partially_bound_binding_array: false,
                supports_fxaa: true,
                supports_smaa: true,
                supports_taa: false,
                supports_cas: true,
                supports_dlss: false,
                supports_neural_compute: true,
                max_supported_msaa_samples: 8,
                virtual_geometry_supported: true,
                hybrid_global_illumination_supported: true,
                ..Default::default()
            },
            last_anti_alias_fallback: AntiAliasFallbackReport::fallback(
                AntiAliasMode::Taa,
                AntiAliasMode::Fxaa,
                AntiAliasFallbackReason::MissingHistory,
            )
            .with_graph_sample_counts(4, 1),
            last_anti_alias_graph_executed_pass_count: 1,
            last_virtual_geometry_graph_executed_pass_count: 2,
            last_hybrid_gi_graph_executed_pass_count: 3,
            last_particle_graph_executed_pass_count: 1,
            last_shadow_graph_executed_pass_count: 1,
            last_transparent_graph_executed_pass_count: 4,
            last_async_compute_pass_count: 2,
            last_particle_velocity_missing_sprite_count: 5,
            last_particle_gpu_alive_count: 31,
            last_particle_gpu_spawned_total: 44,
            last_particle_gpu_emitter_readback_count: 2,
            last_particle_gpu_indirect_instance_count: 29,
            last_mesh_draw_count: 12,
            last_mesh_opaque_draw_count: 6,
            last_mesh_alpha_mask_draw_count: 2,
            last_mesh_transparent_draw_count: 4,
            last_mesh_early_z_draw_count: 8,
            last_mesh_shadow_caster_draw_count: 8,
            last_mesh_alpha_mask_shadow_caster_draw_count: 2,
            last_mesh_prepared_geometry_draw_count: 5,
            last_mesh_dynamic_geometry_draw_count: 7,
            last_mesh_gpu_morphed_source_draw_count: 2,
            last_mesh_gpu_skinned_morphed_source_draw_count: 1,
            last_mesh_skinned_draw_count: 3,
            last_mesh_skinned_palette_upload_count: 2,
            last_mesh_skinned_previous_palette_upload_count: 1,
            last_mesh_skinned_gpu_source_candidate_count: 1,
            last_mesh_skinned_gpu_cpu_morphed_source_candidate_count: 1,
            last_mesh_skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count: 1,
            last_mesh_skinned_gpu_skinning_draw_count: 1,
            last_mesh_skinned_gpu_velocity_draw_count: 1,
            last_mesh_indirect_draw_count: 3,
            last_mesh_lod_draw_count: 2,
            last_mesh_previous_velocity_transform_draw_count: 5,
            last_mesh_missing_velocity_transform_draw_count: 2,
            last_mesh_taa_reactive_mask_command_count: 3,
            last_mesh_static_batch_candidate_group_count: 2,
            last_mesh_static_batch_candidate_draw_count: 5,
            last_mesh_dynamic_batch_candidate_group_count: 3,
            last_mesh_dynamic_batch_candidate_draw_count: 6,
            last_mesh_gpu_instancing_candidate_group_count: 4,
            last_mesh_gpu_instancing_candidate_draw_count: 9,
            last_mesh_pending_static_command_cache_draw_candidate_count: 3,
            last_mesh_pending_static_command_cache_phase_candidate_count: 7,
            last_mesh_pending_static_command_cache_depth_prepass_candidate_count: 3,
            last_mesh_pending_static_command_cache_shadow_candidate_count: 2,
            last_mesh_pending_static_command_cache_opaque_candidate_count: 1,
            last_mesh_pending_static_command_cache_alpha_mask_candidate_count: 1,
            last_mesh_pre_mesh_draw_static_command_cache_skipped_draw_count: 2,
            last_mesh_pre_mesh_draw_static_command_cache_skipped_phase_count: 5,
            last_mesh_pre_mesh_draw_static_command_cache_visibility_pruned_draw_count: 1,
            last_mesh_pre_mesh_draw_static_command_cache_residual_material_phase_draw_count: 3,
            last_mesh_pre_mesh_draw_static_command_cache_residual_rebuild_input_missing_draw_count:
                4,
            last_mesh_pre_mesh_draw_static_command_cache_residual_rebuild_rejected_draw_count: 6,
            last_indirect_batch_count: 2,
            last_indirect_batched_draw_count: 5,
            last_indirect_fallback_draw_count: 4,
            last_indirect_args_count: 5,
            last_gpu_scene_primitive_count: 5,
            last_gpu_scene_instance_count: 7,
            last_gpu_scene_dirty_entry_count: 3,
            last_gpu_scene_uploaded_bytes: 128,
            last_gpu_scene_upload_path: RenderGpuSceneUploadPath::DirectQueueWrite,
            last_gpu_scene_free_span_count: 2,
            last_gpu_scene_primitive_upload_range_count: 1,
            last_gpu_scene_instance_upload_range_count: 4,
            last_material_count: 13,
            last_material_ready_count: 10,
            last_material_fallback_count: 2,
            last_material_validation_error_count: 1,
            last_material_diagnostic_count: 4,
            last_post_process_lut_request_count: 1,
            last_post_process_lut_ready_count: 0,
            last_post_process_lut_fallback_count: 1,
            last_post_process_lut_2d_strip_ready_count: 0,
            last_post_process_lut_3d_request_count: 1,
            last_post_process_lut_unsupported_shape_count: 0,
            last_motion_vector_camera_status: MotionVectorCameraStatus::Ready,
            last_directional_light_count: 3,
            last_directional_light_ready_count: 3,
            last_directional_light_degraded_count: 0,
            last_point_light_count: 4,
            last_point_light_ready_count: 4,
            last_point_light_degraded_count: 0,
            last_spot_light_count: 5,
            last_spot_light_ready_count: 5,
            last_spot_light_degraded_count: 0,
            last_ambient_light_count: 2,
            last_ambient_light_ready_count: 2,
            last_ambient_light_degraded_count: 0,
            last_rect_light_count: 1,
            last_rect_light_ready_count: 0,
            last_rect_light_degraded_count: 1,
            last_sprite_count: 11,
            last_sprite_ready_count: 9,
            last_sprite_texture_fallback_count: 2,
            last_sprite_graph_executed_pass_count: 3,
            last_sprite_draw_batch_count: 4,
            last_sprite_batched_sprite_count: 10,
            last_sprite_image_slice_count: 14,
            last_sprite_expanded_image_slice_count: 4,
            last_sprite_vertex_count: 60,
            last_sprite_opaque_draw_batch_count: 1,
            last_sprite_alpha_mask_draw_batch_count: 1,
            last_sprite_transparent_draw_batch_count: 2,
            last_ui_command_count: 17,
            last_ui_quad_count: 8,
            last_ui_text_payload_count: 5,
            last_ui_image_payload_count: 2,
            last_ui_clipped_command_count: 3,
            last_ui_graph_executed_pass_count: 1,
            last_virtual_geometry_cluster_budget: 128,
            last_virtual_geometry_page_budget: 64,
            last_virtual_geometry_input_cluster_count: 40,
            last_virtual_geometry_input_page_count: 12,
            last_virtual_geometry_visible_cluster_count: 10,
            last_virtual_geometry_visible_entity_count: 5,
            last_virtual_geometry_instance_count: 3,
            last_virtual_geometry_payload_source: RenderVirtualGeometryPayloadSource::Authored,
            last_virtual_geometry_requested_page_count: 7,
            last_virtual_geometry_dirty_page_count: 2,
            last_virtual_geometry_forced_mip: Some(3),
            last_virtual_geometry_freeze_cull: true,
            last_virtual_geometry_visualize_bvh: false,
            last_virtual_geometry_visualize_visbuffer: true,
            last_virtual_geometry_print_leaf_clusters: false,
            last_virtual_geometry_page_table_entry_count: 50,
            last_virtual_geometry_resident_page_count: 20,
            last_virtual_geometry_pending_request_count: 6,
            last_virtual_geometry_page_dependency_count: 8,
            last_virtual_geometry_completed_page_count: 4,
            last_virtual_geometry_replaced_page_count: 1,
            last_virtual_geometry_indirect_draw_count: 9,
            last_virtual_geometry_indirect_buffer_count: 2,
            last_virtual_geometry_indirect_args_count: 18,
            last_virtual_geometry_indirect_segment_count: 11,
            last_virtual_geometry_execution_segment_count: 11,
            last_virtual_geometry_execution_page_count: 7,
            last_virtual_geometry_execution_resident_segment_count: 5,
            last_virtual_geometry_execution_pending_segment_count: 4,
            last_virtual_geometry_execution_missing_segment_count: 2,
            last_virtual_geometry_execution_repeated_draw_count: 1,
            last_virtual_geometry_cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
            last_virtual_geometry_node_and_cluster_cull_source:
                RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput,
            last_virtual_geometry_node_and_cluster_cull_record_count: 16,
            last_virtual_geometry_node_and_cluster_cull_dispatch_group_count: [3, 4, 5],
            last_virtual_geometry_node_and_cluster_cull_instance_seed_count: 6,
            last_virtual_geometry_node_and_cluster_cull_instance_work_item_count: 7,
            last_virtual_geometry_node_and_cluster_cull_cluster_work_item_count: 8,
            last_virtual_geometry_node_and_cluster_cull_hierarchy_child_id_count: 9,
            last_virtual_geometry_node_and_cluster_cull_child_work_item_count: 10,
            last_virtual_geometry_node_and_cluster_cull_traversal_record_count: 11,
            last_virtual_geometry_node_and_cluster_cull_page_request_count: 12,
            last_virtual_geometry_selected_cluster_source:
                RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections,
            last_virtual_geometry_selected_cluster_count: 13,
            last_virtual_geometry_visbuffer64_source:
                RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
            last_virtual_geometry_visbuffer64_entry_count: 14,
            last_virtual_geometry_hardware_rasterization_source:
                RenderVirtualGeometryHardwareRasterizationSource::RenderPathExecutionSelections,
            last_virtual_geometry_hardware_rasterization_record_count: 15,
            last_hybrid_gi_active_probe_count: 5,
            last_hybrid_gi_requested_probe_count: 6,
            last_hybrid_gi_dirty_probe_count: 2,
            last_hybrid_gi_cache_entry_count: 8,
            last_hybrid_gi_resident_probe_count: 4,
            last_hybrid_gi_pending_update_count: 3,
            last_hybrid_gi_scheduled_trace_region_count: 1,
            last_hybrid_gi_scene_card_count: 7,
            last_hybrid_gi_scene_screen_probe_count: 9,
            last_hybrid_gi_scene_radiance_cache_entry_count: 10,
            last_hybrid_gi_surface_cache_resident_page_count: 11,
            last_hybrid_gi_surface_cache_dirty_page_count: 12,
            last_hybrid_gi_surface_cache_feedback_card_count: 13,
            last_hybrid_gi_surface_cache_capture_slot_count: 14,
            last_hybrid_gi_surface_cache_invalidated_page_count: 15,
            last_hybrid_gi_surface_cache_depth_sample_count: 16,
            last_hybrid_gi_probe_trace_tile_count: 17,
            last_hybrid_gi_probe_trace_dispatch_group_count: [18, 19, 20],
            last_hybrid_gi_voxel_resident_clipmap_count: 21,
            last_hybrid_gi_voxel_dirty_clipmap_count: 22,
            last_hybrid_gi_voxel_invalidated_clipmap_count: 23,
            last_hybrid_gi_payload_source: RenderHybridGiPayloadSource::SceneRepresentation,
            advanced_provider_availability: AdvancedProviderAvailability::new()
                .with_virtual_geometry_provider("diagnostics-vg-provider"),
            last_advanced_provider_reports: vec![
                AdvancedProviderReport {
                    feature: AdvancedRenderFeature::VirtualGeometry,
                    requested: true,
                    provider_id: Some("diagnostics-vg-provider".to_string()),
                    status: AdvancedProviderStatus::Ready,
                    degradations: Vec::new(),
                },
                AdvancedProviderReport {
                    feature: AdvancedRenderFeature::HybridGlobalIllumination,
                    requested: true,
                    provider_id: None,
                    status: AdvancedProviderStatus::Degraded,
                    degradations: vec![AdvancedRenderDegradation::missing_provider(
                        AdvancedRenderFeature::HybridGlobalIllumination,
                    )],
                },
            ],
            last_solari_runtime_report: SolariRuntimeReport {
                requested: true,
                provider_id: Some("diagnostics-solari-provider".to_string()),
                status: SolariRuntimeStatus::ExperimentalDisabled,
                settings: SolariSettings::new(),
                degradations: vec![SolariRuntimeDegradation::experimental_disabled()],
            },
            ..Default::default()
        })
    }

    fn query_virtual_geometry_debug_snapshot(
        &self,
    ) -> Result<Option<RenderVirtualGeometryDebugSnapshot>, RenderFrameworkError> {
        Ok(None)
    }

    fn capture_frame(
        &self,
        _viewport: RenderViewportHandle,
    ) -> Result<Option<CapturedFrame>, RenderFrameworkError> {
        Ok(None)
    }

    fn set_quality_profile(
        &self,
        _viewport: RenderViewportHandle,
        _profile: RenderQualityProfile,
    ) -> Result<(), RenderFrameworkError> {
        Ok(())
    }
}

pub(super) fn assert_series_current(
    store: &crate::core::diagnostics::DiagnosticStoreSnapshot,
    path: &str,
    expected: f64,
    expected_unit: &str,
) {
    let series = store
        .series
        .iter()
        .find(|series| series.path.as_str() == path)
        .unwrap_or_else(|| panic!("missing diagnostic series {path}"));
    assert_eq!(
        series.current,
        Some(expected),
        "unexpected current value for {path}"
    );
    assert_eq!(
        series.unit.as_deref(),
        Some(expected_unit),
        "unexpected unit for {path}"
    );
    assert!(series.subsystem_tags.contains(&"render".to_string()));
    assert!(series.subsystem_tags.contains(&"effect_stack".to_string()));
}

pub(super) fn assert_render_count_series(
    store: &crate::core::diagnostics::DiagnosticStoreSnapshot,
    path: &str,
    expected: f64,
    expected_tags: &[&str],
) {
    let series = store
        .series
        .iter()
        .find(|series| series.path.as_str() == path)
        .unwrap_or_else(|| panic!("missing diagnostic series {path}"));
    assert_eq!(
        series.current,
        Some(expected),
        "unexpected current value for {path}"
    );
    assert_eq!(
        series.unit.as_deref(),
        Some("count"),
        "unexpected unit for {path}"
    );
    assert!(series.subsystem_tags.contains(&"render".to_string()));
    for expected_tag in expected_tags {
        assert!(series.subsystem_tags.contains(&expected_tag.to_string()));
    }
}

pub(super) fn assert_render_byte_series(
    store: &crate::core::diagnostics::DiagnosticStoreSnapshot,
    path: &str,
    expected: f64,
    expected_tags: &[&str],
) {
    let series = store
        .series
        .iter()
        .find(|series| series.path.as_str() == path)
        .unwrap_or_else(|| panic!("missing diagnostic series {path}"));
    assert_eq!(
        series.current,
        Some(expected),
        "unexpected current value for {path}"
    );
    assert_eq!(series.unit.as_deref(), Some("bytes"));
    assert!(series.subsystem_tags.contains(&"render".to_string()));
    for expected_tag in expected_tags {
        assert!(series.subsystem_tags.contains(&expected_tag.to_string()));
    }
}

pub(super) fn assert_render_microsecond_series(
    store: &crate::core::diagnostics::DiagnosticStoreSnapshot,
    path: &str,
    expected: f64,
    expected_tags: &[&str],
) {
    let series = store
        .series
        .iter()
        .find(|series| series.path.as_str() == path)
        .unwrap_or_else(|| panic!("missing diagnostic series {path}"));
    assert_eq!(
        series.current,
        Some(expected),
        "unexpected current value for {path}"
    );
    assert_eq!(series.unit.as_deref(), Some("microseconds"));
    assert!(series.subsystem_tags.contains(&"render".to_string()));
    for expected_tag in expected_tags {
        assert!(series.subsystem_tags.contains(&expected_tag.to_string()));
    }
}

pub(super) fn assert_render_bool_series(
    store: &crate::core::diagnostics::DiagnosticStoreSnapshot,
    path: &str,
    expected: bool,
    expected_tags: &[&str],
) {
    let series = store
        .series
        .iter()
        .find(|series| series.path.as_str() == path)
        .unwrap_or_else(|| panic!("missing diagnostic series {path}"));
    assert_eq!(series.current, Some(u8::from(expected) as f64));
    assert_eq!(series.unit.as_deref(), Some("bool"));
    assert!(series.subsystem_tags.contains(&"render".to_string()));
    for expected_tag in expected_tags {
        assert!(series.subsystem_tags.contains(&expected_tag.to_string()));
    }
}

pub(super) fn assert_light_family_series(
    store: &crate::core::diagnostics::DiagnosticStoreSnapshot,
    family: &str,
    total: f64,
    ready: f64,
    degraded: f64,
) {
    assert_render_count_series(
        store,
        &format!("render.light.{family}.count"),
        total,
        &["light", family],
    );
    assert_render_count_series(
        store,
        &format!("render.light.{family}.ready_count"),
        ready,
        &["light", family, "ready"],
    );
    assert_render_count_series(
        store,
        &format!("render.light.{family}.degraded_count"),
        degraded,
        &["light", family, "degraded"],
    );
}
