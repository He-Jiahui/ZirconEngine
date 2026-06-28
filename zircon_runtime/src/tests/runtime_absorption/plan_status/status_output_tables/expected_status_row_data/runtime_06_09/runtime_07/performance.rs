use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 07 scene/EventBus poison-safe locks",
        &[
            "scene_level_poison_recovery_coremin_passed_eventbus_guard_timeout",
            "LevelSystem",
            "EventBus",
            "level_system_recovers_world_lock_after_writer_panic",
        ],
    ),
    (
        "Runtime 07 render submit source-extract sharing",
        &[
            "render_submit_source_extract_shared_coremin_check_passed_partial",
            "source_extract: Arc<RenderFrameExtract>",
            "runtime_07_submit_context_shares_large_extract_payloads",
            "FrameSubmissionContext",
        ],
    ),
    (
        "Runtime 07 render submit viewport/provider errors",
        &[
            "render_submit_viewport_provider_errors_review_guard_static_passed_cargo_timeout_no_result_full_runtime07_pending",
            "viewport_record_mut_after_generation_check",
            "RenderFrameworkError::UnsupportedCapability",
            "review_f4_render_submit_capability_gaps_return_typed_errors",
        ],
    ),
    (
        "Runtime 07 render camera-loop descriptor submissions",
        &[
            "render_camera_loop_descriptor_submissions_coremin_check_passed_partial",
            "camera_loop_submissions",
            "CameraLoopSubmission",
            "camera: CameraRenderDescriptor",
        ],
    ),
    (
        "Runtime 07 render camera-loop borrowed sequence resolution",
        &[
            "render_camera_loop_borrowed_sequence_resolution_static_passed_cargo_deferred",
            "resolve_camera_sequence_borrowed",
            "extract.view.cameras.clone",
            "runtime_07_submit_context_shares_large_extract_payloads",
        ],
    ),
    (
        "Runtime 07 render camera-loop source view restore narrowing",
        &[
            "render_camera_loop_source_view_restore_narrowed_static_passed_cargo_deferred",
            "CameraLoopExtractSourceState",
            "view_target_size: Option<crate::core::math::UVec2>",
            "extract.view.target_size = self.view_target_size",
        ],
    ),
    (
        "Runtime 07 render camera-loop post-process source restore narrowing",
        &[
            "render_camera_loop_post_process_restore_narrowed_static_passed_cargo_deferred",
            "CameraLoopPostProcessSourceState",
            "post_process: CameraLoopPostProcessSourceState",
            "self.post_process.restore_to(&mut extract.post_process)",
        ],
    ),
    (
        "Runtime 07 render camera-loop VG/HGI conditional source restore",
        &[
            "render_camera_loop_vg_hgi_conditional_restore_static_passed_cargo_deferred",
            "restore_optional_payload_source",
            "restore_optional_payload_source(&self.virtual_geometry",
            "restore_optional_payload_source(&self.hybrid_global_illumination",
        ],
    ),
    (
        "Runtime 07 render camera-loop single-child source-state capture skip",
        &[
            "render_camera_loop_single_child_source_state_capture_skipped_static_passed_cargo_deferred",
            "Some(CameraLoopExtractSourceState::capture(source))",
            "(submissions.len() > 1).then(|| CameraLoopFrameSourceState::capture(&mut frame))",
            "if submission_index > 0",
        ],
    ),
    (
        "Runtime 07 render camera-loop source payload slot ownership",
        &[
            "render_camera_loop_source_payload_slot_owned_static_passed_cargo_deferred",
            "FrameSubmissionSourcePayloads",
            "virtual_geometry: extract.geometry.virtual_geometry.take()",
            "FrameHistoryValidationKey::from_extract_with_hybrid_gi",
        ],
    ),
    (
        "Runtime 07 render camera-loop frame terminal move",
        &[
            "render_camera_loop_frame_terminal_move_coremin_check_passed_partial",
            "camera_loop_frame_submissions",
            "source_frame.take()",
            "project_owned_frame_to_selected_camera",
        ],
    ),
    (
        "Runtime 07 render submit feedback sideband owned merge",
        &[
            "render_submit_feedback_sidebands_owned_merge_coremin_check_passed_partial",
            "collect_runtime_feedback",
            "take_hybrid_gi_readback_outputs",
            "RenderVirtualGeometryReadbackOutputs",
        ],
    ),
    (
        "Runtime 07 render prepared sideband frame owner move",
        &[
            "render_prepared_sideband_frame_owner_move_coremin_check_passed_partial",
            "into_prepared_runtime_sidebands",
            "prepared_runtime_sidebands_mut",
            "take_hybrid_gi_evictable_probe_ids",
        ],
    ),
    (
        "Runtime 07 render direct runtime-frame streaming camera loop",
        &[
            "render_direct_runtime_frame_streaming_camera_loop_coremin_check_passed_partial",
            "submit_camera_loop_frame",
            "CameraLoopFrameSourceState",
            "select_camera_descriptor",
        ],
    ),
    (
        "Runtime 07 render generated camera-loop shared extract",
        &[
            "render_generated_camera_loop_shared_extract_static_passed_cargo_locked_blocked",
            "stream_camera_loop_extract_submissions",
            "CameraLoopExtractSourceState",
            "Cargo.lock",
        ],
    ),
    (
        "Runtime 07 render shared effective extract frame source",
        &[
            "render_shared_effective_extract_frame_source_coremin_check_passed_partial",
            "ViewportRenderFrame::from_shared_extract",
            "source_extract()",
            "direct runtime-frame context clone",
        ],
    ),
    (
        "Runtime 07 render direct runtime-frame shared context extract",
        &[
            "render_direct_runtime_frame_shared_context_extract_coremin_check_passed_partial",
            "build_frame_submission_context_from_runtime_frame_extract",
            "Arc::make_mut(extract)",
            "frame.extract.as_ref().clone()",
        ],
    ),
    (
        "Runtime 07 render VG debug overlay frame override",
        &[
            "render_vg_debug_overlay_frame_override_coremin_check_passed_partial",
            "runtime_overlay_override",
            "runtime_virtual_geometry_debug_overlays",
            "Arc::try_unwrap(extract).unwrap_or_else",
        ],
    ),
    (
        "Runtime 07 render direct runtime-frame trace export",
        &[
            "render_direct_runtime_frame_trace_export_static_passed_profile_timeout_fps_pending",
            "render_profiling.rs",
            "direct_runtime_frame_submit_exports_perfetto_trace_artifacts",
            "timeline.perfetto.json",
        ],
    ),
    (
        "Runtime 07 render submit effective extract projection",
        &[
            "render_submit_effective_extract_projection_coremin_check_passed_partial",
            "build_frame_submission_context_from_runtime_frame_extract",
            "Arc::make_mut(extract_source)",
            "Arc::clone(extract_source)",
        ],
    ),
    (
        "Runtime 07 F16 compiled-scene split status guard",
        &[
            "compiled_scene_render_split_review_guard_static_passed_cargo_deferred",
            "review_f16_compiled_scene_render_path_uses_split_owners",
            "bind_compiled_scene_graph_resources.rs",
            "submit_compiled_scene_frame.rs",
        ],
    ),
    (
        "Runtime 07 Performance hotpath Markdown renderer split",
        &[
            "performance_hotpath_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "performance_hotpath_markdown.py",
            "performance_hotpath_boundary.py` remains the 643-line audit/risk owner",
            "Markdown owner is 139 lines",
        ],
    ),
    (
        "Runtime 07 Performance hotpath inventory split",
        &[
            "performance_hotpath_inventory_split_static_passed_cargo_deferred_tests_deferred",
            "performance_hotpath_source_inventory.py",
            "performance_hotpath_anchor_inventory.py",
            "performance_hotpath_boundary.py` is now the 353-line audit reader",
        ],
    ),
    (
        "Runtime 07 Performance hotpath 镜像文档守卫",
        &[
            "runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts",
            "performance_hotpath_boundary",
            "expected_source_file_count = 26",
            "extract/ecs_query/performance profiling/FPS Cargo gates pending",
        ],
    ),
    (
        "Runtime 07 ECS frame diagnostics aggregation",
        &[
            "EcsFramePerformanceDiagnostics",
            "ecs_frame_performance_diagnostics_record_query_and_change_counts",
            "expected_source_file_count = 24",
            "ecs_query` gate",
        ],
    ),
    (
        "Runtime 07 extract rebuild cache",
        &[
            "RuntimeFrameExtractCache",
            "extract.rebuild_clones = 0",
            "frame_extract_rebuilds_after_scene_change",
            "extract_counter_anchor_count = 17",
        ],
    ),
    (
        "Runtime 07 animation scene frame diagnostics",
        &[
            "AnimationSceneFrameDiagnostics",
            "animation.scene.scanned_entities",
            "animation.scene.output_poses",
            "animation_scene_anchor_count = 19",
        ],
    ),
    (
        "Runtime 07 profile counter hotspot export",
        &[
            "CounterHotspotReport",
            "counter_hotspots.json",
            "analyze_counter_hotspots",
            "ProfileControlResponse.counter_hotspot_report",
        ],
    ),
    (
        "Runtime 07 QueryState cache owner performance audit sync",
        &[
            "query_state/cache.rs",
            "expected_source_file_count = 45",
            "missing_query_counter_anchors = []",
            "runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts",
        ],
    ),
    (
        "Runtime 07 extract cache hit/miss diagnostics",
        &[
            "EXTRACT_CACHE_HITS_DIAGNOSTIC",
            "extract.cache_hits",
            "extract.cache_misses",
            "extract_counter_anchor_count = 21",
        ],
    ),
    (
        "Runtime 07 QueryState frame auto-collection",
        &[
            "QueryState::take_unreported_cache_stats()",
            "SystemParam::record_performance_diagnostics",
            "World::record_ecs_query_cache_stats",
            "system_state_records_query_cache_stats_into_world_frame_diagnostics",
        ],
    ),
    (
        "Runtime 07 ChangeDetection frame auto-collection",
        &[
            "matches_component_locations_with_stats",
            "take_unreported_change_detection_stats",
            "World::record_ecs_change_detection_stats",
            "system_state_records_change_detection_stats_into_world_frame_diagnostics",
        ],
    ),
    (
        "Runtime 07 QueryState iterator lifetime guard",
        &[
            "NonNull<QueryState<D, F>>",
            "read-only, non-cached iterators",
            "QueryState::single",
            "query_counter_anchor_count = 32",
        ],
    ),
    (
        "Runtime 07 FPS gate support unblock",
        &[
            "ZR_VM_RUST_BINDING_LIB_DIR",
            "zircon_runtime_interface::ui::template::UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION",
            "RenderBloomSettings",
            "904s timeout no result",
        ],
    ),
    (
        "Runtime 07 profiling build tooling",
        &[
            "--mode profiling",
            "--runtime-features target-client,profiling,profiling-tracy",
            "-CargoProfile profiling",
            "profiling_build_tooling_static_passed_cargo_deferred_active_lanes",
        ],
    ),
];
