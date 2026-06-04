use std::sync::Arc;

use crate::core::framework::render::{
    AdvancedProviderAvailability, AdvancedProviderReport, AdvancedProviderStatus,
    AdvancedRenderDegradation, AdvancedRenderFeature, AntiAliasFallbackReason,
    AntiAliasFallbackReport, AntiAliasMode, CapturedFrame, FrameHistoryHandle,
    FrameHistoryInvalidationReason, FrameHistoryStatus, RenderFrameExtract, RenderFramework,
    RenderFrameworkError, RenderHybridGiPayloadSource, RenderPipelineHandle,
    RenderPostProcessEffectStackReport, RenderQualityProfile, RenderQueueCapability, RenderStats,
    RenderViewportDescriptor, RenderViewportHandle,
    RenderVirtualGeometryClusterSelectionInputSource, RenderVirtualGeometryDebugSnapshot,
    RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryNodeAndClusterCullSource, RenderVirtualGeometryPayloadSource,
    RenderVirtualGeometrySelectedClusterSource, RenderVirtualGeometryVisBuffer64Source,
    SolariRuntimeDegradation, SolariRuntimeReport, SolariRuntimeStatus, SolariSettings,
};
use crate::core::{ManagerDescriptor, ModuleDescriptor, RegistryName, ServiceObject, StartupMode};
use crate::engine_module::factory;
use crate::graphics::RenderPipelineAsset;
use zircon_runtime_interface::ui::surface::UiRenderExtract;

pub(super) const DIAGNOSTICS_TEST_MODULE: &str = "DiagnosticsTestModule";

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

    fn register_pipeline_asset(
        &self,
        pipeline: RenderPipelineAsset,
    ) -> Result<RenderPipelineHandle, RenderFrameworkError> {
        Ok(pipeline.handle)
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
            last_frame_target_size: Some(crate::core::math::UVec2::new(1280, 720)),
            last_frame_render_size: Some(crate::core::math::UVec2::new(960, 540)),
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
            last_post_process_graph_node_count: 5,
            last_post_process_graph_skipped_node_count: 1,
            last_post_process_final_composite_node: Some("final-composite".to_string()),
            last_post_process_graph_executed_nodes: vec![
                "bloom".to_string(),
                "effect-stack".to_string(),
                "final-composite".to_string(),
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
            ),
            last_anti_alias_graph_executed_pass_count: 1,
            last_virtual_geometry_graph_executed_pass_count: 2,
            last_hybrid_gi_graph_executed_pass_count: 3,
            last_particle_graph_executed_pass_count: 1,
            last_shadow_graph_executed_pass_count: 1,
            last_transparent_graph_executed_pass_count: 4,
            last_async_compute_pass_count: 2,
            last_particle_gpu_alive_count: 31,
            last_particle_gpu_spawned_total: 44,
            last_particle_gpu_emitter_readback_count: 2,
            last_particle_gpu_indirect_instance_count: 29,
            last_mesh_draw_count: 12,
            last_mesh_opaque_draw_count: 6,
            last_mesh_alpha_mask_draw_count: 2,
            last_mesh_transparent_draw_count: 4,
            last_mesh_early_z_draw_count: 8,
            last_mesh_prepared_geometry_draw_count: 5,
            last_mesh_dynamic_geometry_draw_count: 7,
            last_mesh_indirect_draw_count: 3,
            last_mesh_static_batch_candidate_group_count: 2,
            last_mesh_static_batch_candidate_draw_count: 5,
            last_mesh_dynamic_batch_candidate_group_count: 3,
            last_mesh_dynamic_batch_candidate_draw_count: 6,
            last_mesh_gpu_instancing_candidate_group_count: 4,
            last_mesh_gpu_instancing_candidate_draw_count: 9,
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
            last_directional_light_count: 3,
            last_directional_light_ready_count: 1,
            last_directional_light_degraded_count: 2,
            last_point_light_count: 4,
            last_point_light_ready_count: 0,
            last_point_light_degraded_count: 4,
            last_spot_light_count: 5,
            last_spot_light_ready_count: 0,
            last_spot_light_degraded_count: 5,
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
            last_hybrid_gi_voxel_resident_clipmap_count: 16,
            last_hybrid_gi_voxel_dirty_clipmap_count: 17,
            last_hybrid_gi_voxel_invalidated_clipmap_count: 18,
            last_hybrid_gi_payload_source: RenderHybridGiPayloadSource::Authored,
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
    assert_eq!(series.current, Some(expected));
    assert_eq!(series.unit.as_deref(), Some(expected_unit));
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
    assert_eq!(series.current, Some(expected));
    assert_eq!(series.unit.as_deref(), Some("count"));
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
    assert_eq!(series.current, Some(expected));
    assert_eq!(series.unit.as_deref(), Some("bytes"));
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
