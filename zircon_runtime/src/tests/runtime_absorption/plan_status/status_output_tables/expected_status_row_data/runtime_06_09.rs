use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 06 Plugin surface/lifecycle 镜像文档守卫",
        [
            "runtime_06_plugin_surface_lifecycle_mirror_docs_match_structure_audit_counts",
            "plugin_surface_lifecycle_boundary",
            "standalone plugin_surface_lifecycle 1/1",
            "script::vm/vampire_project_session/plugin/native_plugin/app/plugins",
        ],
    ),
    (
        "Runtime 06 native root re-export current mirror fix",
        [
            "plugin_root_symbols.len()",
            "native root re-export 0/0",
            "last_refined = 2026-06-16",
            "standalone plugin_surface_lifecycle 1/1",
        ],
    ),
    (
        "Runtime 06 plugin::native hard-cutover",
        [
            "plugin::native",
            "root_reexport_count = 0",
            "native_namespace_reexport_count = 60",
            "M4 gate `classified-and-clear`",
        ],
    ),
    (
        "Runtime 06 fallback lifecycle failure tests",
        [
            "runtime_06_vm_lifecycle_fallback_failure_tests_are_folder_backed",
            "fallback lifecycle failure tests 4/4",
            "vm_lifecycle_fallback_missing_optional_export_returns_none_not_error",
            "real-backend Cargo remains pending",
        ],
    ),
    (
        "Runtime 06 fallback lifecycle Cargo 验证",
        [
            "fallback_cargo_passed_real_backend_pending",
            "vm_lifecycle_fallback --no-default-features --features core-min",
            "5/5",
            "real-backend Cargo",
        ],
    ),
    (
        "Runtime 06 shader artifact cache real-backend unblock",
        [
            "asset_cache_fixed_vampire_session_pending",
            "ArtifactCacheShaderImportRedirectAsset",
            "project_manager_imports_compound_zshader_package_with_subassets",
            "vampire_project_session_starts_paused_until_start_button_click",
        ],
    ),
    (
        "Runtime 06 Vampire real-backend menu/retry focused validation",
        [
            "vampire_real_backend_focused_passed_full_gate_pending",
            "vampire_project_session_game_over_menu_retries_to_playing",
            "gameplay.script_number_at_most",
            "vampire.spawn_grace",
        ],
    ),
    (
        "Runtime 06 Vampire HUD real-backend capture validation",
        [
            "vampire_hud_real_backend_focused_passed_full_gate_pending",
            "particle-render",
            "vampire_project_session_capture_frame_draws_world_hud_bars",
            "particle_pipeline_keeps_world_hud_billboards_transparent_and_depth_read_only",
        ],
    ),
    (
        "Runtime 06 native loader test namespace migration",
        [
            "runtime_06_native_loader_tests_use_isolated_plugin_native_namespace",
            "native loader test files 3/3",
            "native test namespace import files 2/2",
            "native test root import leaks 0/0",
        ],
    ),
    (
        "Runtime 06 V1/V2 ABI hard-cutover",
        [
            "V3-only native plugin ABI",
            "unknown ABI rejection",
            "native_loader_v1_v2_file_count = 0",
            "plugin_v1_v2_usage_files = 0",
        ],
    ),
    (
        "Runtime 06 hot reload failure injection",
        [
            "hot_reload_missing_symbol_after_reload_rolls_back_to_previous_instance",
            "hot_reload_state_restore_failure_rolls_back_and_reports",
            "hot reload failure injection",
            "Cargo timeout",
        ],
    ),
    (
        "Runtime 07 Performance hotpath 镜像文档守卫",
        [
            "runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts",
            "performance_hotpath_boundary",
            "expected_source_file_count = 26",
            "extract/ecs_query/performance profiling/FPS Cargo gates pending",
        ],
    ),
    (
        "Runtime 07 ECS frame diagnostics aggregation",
        [
            "EcsFramePerformanceDiagnostics",
            "ecs_frame_performance_diagnostics_record_query_and_change_counts",
            "expected_source_file_count = 24",
            "ecs_query` gate",
        ],
    ),
    (
        "Runtime 07 extract rebuild cache",
        [
            "RuntimeFrameExtractCache",
            "extract.rebuild_clones = 0",
            "frame_extract_rebuilds_after_scene_change",
            "extract_counter_anchor_count = 17",
        ],
    ),
    (
        "Runtime 07 asset worker frame sampler",
        [
            "AssetWorkerPoolFrameSampler",
            "asset.worker.frame_completed",
            "asset_worker_anchor_count = 13",
            "worker_diagnostic_count = 7",
        ],
    ),
    (
        "Runtime 07 asset worker manager sampler entry",
        [
            "spawn_worker_pool_with_frame_sampler",
            "AssetWorkerPoolFrameSampler::from_pool(&pool)",
            "asset_worker_anchor_count = 13",
            "expected_source_file_count = 26",
        ],
    ),
    (
        "Runtime 07 artifact cache payload owner split",
        [
            "cache_payload/{json_value,mesh,toml_value}.rs",
            "runtime_07_artifact_cache_payload_owner_split_keeps_wire_types_folder_backed",
            "expected_source_file_count = 22",
            "large_file_hotspot_count = 41",
        ],
    ),
    (
        "Runtime 07 render product diagnostics owner split",
        [
            "render_stats_store/product/{camera,mesh_queue,gpu_scene}.rs",
            "runtime_07_render_product_diagnostics_owner_split_keeps_families_folder_backed",
            "expected_source_file_count = 38",
            "large_file_hotspot_count = 39",
        ],
    ),
    (
        "Runtime 07 animation scene frame diagnostics",
        [
            "AnimationSceneFrameDiagnostics",
            "animation.scene.scanned_entities",
            "animation.scene.output_poses",
            "animation_scene_anchor_count = 19",
        ],
    ),
    (
        "Runtime 07 QueryState cache owner performance audit sync",
        [
            "query_state/cache.rs",
            "expected_source_file_count = 45",
            "missing_query_counter_anchors = []",
            "runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts",
        ],
    ),
    (
        "Runtime 07 virtual geometry debug snapshot owner split",
        [
            "virtual_geometry_debug_snapshot/{cull_input,node_and_cluster_cull,snapshot}.rs",
            "runtime_07_virtual_geometry_debug_snapshot_owner_split_keeps_contracts_folder_backed",
            "Runtime 07 owner-budget 36-hotspot navigation split sync",
            "extract/ecs_query/performance profiling/FPS Cargo gates",
        ],
    ),
    (
        "Runtime 07 extract cache hit/miss diagnostics",
        [
            "EXTRACT_CACHE_HITS_DIAGNOSTIC",
            "extract.cache_hits",
            "extract.cache_misses",
            "extract_counter_anchor_count = 21",
        ],
    ),
    (
        "Runtime 07 QueryState frame auto-collection",
        [
            "QueryState::take_unreported_cache_stats()",
            "SystemParam::record_performance_diagnostics",
            "World::record_ecs_query_cache_stats",
            "system_state_records_query_cache_stats_into_world_frame_diagnostics",
        ],
    ),
    (
        "Runtime 07 ChangeDetection frame auto-collection",
        [
            "matches_component_locations_with_stats",
            "take_unreported_change_detection_stats",
            "World::record_ecs_change_detection_stats",
            "system_state_records_change_detection_stats_into_world_frame_diagnostics",
        ],
    ),
    (
        "Runtime 07 QueryState iterator lifetime guard",
        [
            "NonNull<QueryState<D, F>>",
            "read-only, non-cached iterators",
            "QueryState::single",
            "query_counter_anchor_count = 32",
        ],
    ),
    (
        "Runtime 07 FPS gate support unblock",
        [
            "ZR_VM_RUST_BINDING_LIB_DIR",
            "zircon_runtime_interface::ui::template::UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION",
            "RenderBloomSettings",
            "904s timeout no result",
        ],
    ),
    (
        "Runtime 07 profiling build tooling",
        [
            "--mode profiling",
            "--runtime-features target-client,profiling,profiling-tracy",
            "-CargoProfile profiling",
            "profiling_build_tooling_static_passed_cargo_deferred_active_lanes",
        ],
    ),
    (
        "Runtime 07 scene asset owner split",
        [
            "folder-backed `scene/{mod,animation,asset,camera,defaults,entity,extensions,lighting,management,mesh,physics,post_process,transform}.rs`",
            "`SceneMobilityAsset`",
            "`management.rs` 432 行",
            "38-hotspot / runtime-other=14 state",
        ],
    ),
    (
        "Runtime 07 scene asset split-drift repair",
        [
            "删除拆分后遗留在 `zircon_runtime/src/asset/assets/scene/physics.rs` 的重复 `SceneMobilityAsset` 定义",
            "`scene/mod.rs` 是唯一 owner",
            "`SceneSpotLightAsset` 公开链",
            "`scene_asset` 与 Runtime 07 Cargo gates 继续 pending",
        ],
    ),
    (
        "Runtime 07 scene asset folder-split public-surface guard",
        [
            "runtime_07_scene_asset_folder_split_keeps_public_surface_and_single_owner",
            "`SceneSpotLightAsset` 字段/导出链",
            "standalone `rustc --edition 2021 --test ...performance_hotspots.rs` 通过 6/6",
            "包级 `scene_asset` / Runtime 07 Cargo gates 仍待 active lanes 清空后补跑",
        ],
    ),
    (
        "Runtime 07 scene asset guard 纳入 performance_hotpath_boundary",
        [
            "`performance_hotpath_boundary.py`",
            "`hotspot_guard_anchor_count = 20`",
            "`missing_hotspot_guard_anchors = []`",
            "direct `performance_hotpath_boundary_audit` risks=0 / hotspot_guard_anchor_count=20",
        ],
    ),
    (
        "Runtime 07 project_io folder split",
        [
            "`project_io/{camera,physics,post_process,references,script,transform}.rs`",
            "`runtime_07_project_io_folder_split_keeps_entry_and_converter_owners`",
            "`large_file_hotspot_count = 38`",
            "`runtime-other=14`",
        ],
    ),
    (
        "Runtime 07 owner-budget evidence drift resync",
        [
            "`large_file_ownership_gate`",
            "38 hotspots",
            "runtime-other=14",
            "`runtime_absorption::performance_hotspots`",
        ],
    ),
    (
        "Runtime 07 owner-budget 38-hotspot 漂移同步",
        [
            "`large_file_hotspot_count = 38`",
            "`runtime-framework-render=2`",
            "`runtime-other=14`",
            "standalone `performance_hotspots.rs` 6/6",
        ],
    ),
    (
        "Runtime 07 owner-budget 37-hotspot 漂移同步",
        [
            "`large_file_hotspot_count = 37`",
            "`runtime-other=13`",
            "`hotspot_guard_anchor_count = 20`",
            "standalone `status_output_tables.rs` 2/2",
        ],
    ),
    (
        "Runtime 07 owner-budget 37-hotspot 再同步",
        [
            "`large_file_hotspot_count = 37`",
            "`runtime-other=12`",
            "direct `performance_hotpath_boundary_audit` risks=0 / hotspots=37",
            "standalone `performance_hotspots.rs` 6/6",
        ],
    ),
    (
        "Runtime 07 owner-budget 38-hotspot 回漂同步",
        [
            "`large_file_hotspot_count = 38`",
            "`runtime-framework-render=2`",
            "direct `performance_hotpath_boundary_audit` risks=0 / hotspots=38",
            "standalone `performance_hotspots.rs` 6/6",
        ],
    ),
    (
        "Runtime 07 owner-budget 39-hotspot 漂移同步",
        [
            "`large_file_hotspot_count = 39`",
            "`runtime-framework-render=3`",
            "direct `performance_hotpath_boundary_audit` risks=0 / hotspots=39",
            "standalone `performance_hotspots.rs` 6/6",
        ],
    ),
    (
        "Runtime 07 owner-budget 42-hotspot 漂移同步",
        [
            "`large_file_hotspot_count = 42`",
            "`runtime-other=15`",
            "direct `performance_hotpath_boundary_audit` risks=0 / hotspots=42",
            "standalone `performance_hotspots.rs` 6/6",
        ],
    ),
    (
        "Runtime 07 owner-budget current doc mirror fix",
        [
            "hotspot_inventory.md",
            "M0 review",
            "42 hotspots / 5 migration-debt owner groups / 0 unclassified hotspots",
            "standalone `performance_hotspots.rs` 6/6",
        ],
    ),
    (
        "Runtime 07 owner-budget 36-hotspot navigation split sync",
        [
            "`large_file_hotspot_count = 36`",
            "`runtime-other=12`",
            "runtime_07_navigation_runtime_owner_split_reduces_owner_budget_hotspot_count",
            "standalone `performance_hotspots.rs` 6/6",
        ],
    ),
    (
        "Runtime 08 ECS 数据面镜像文档守卫",
        [
            "runtime_08_ecs_kernel_data_mirror_docs_match_structure_audit_counts",
            "ecs_kernel_data_boundary",
            "standalone rustc 1/1",
            "entity/observer/command/messages/change_tick/ecs Cargo gates pending",
        ],
    ),
    (
        "Runtime 08 First-stage event update guard",
        [
            "first_stage_updates_all_registered_event_channels",
            "event_message_anchors = 12/12",
            "runtime_08_guard_anchors = 18/18",
            "standalone ecs_kernel_data 1/1",
        ],
    ),
    (
        "Runtime 08 ECS 行为测试锚审计同步",
        [
            "behavior_test_anchor_count = 16",
            "missing_behavior_test_anchors = []",
            "runtime_08_guard_anchors = 18/18",
            "standalone ecs_kernel_data 1/1",
        ],
    ),
    (
        "Runtime 08 QueryState cache owner split",
        [
            "query_state/cache.rs",
            "root_non_empty_lines = 84/180",
            "expected_module_count = 9",
            "entity/observer/command/messages/change_tick/ecs Cargo gates",
        ],
    ),
    (
        "Runtime 09 UI architecture 镜像文档守卫",
        [
            "runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts",
            "ui_architecture_boundary",
            "standalone rustc 18/18",
            "ui/input/naming_boundary/layout/template Cargo gates pending",
        ],
    ),
    (
        "Runtime 09 UI input route authority",
        [
            "runtime_09_m1_1_ui_input_route_authority_static_passed_cargo_pending",
            "runtime_09_m1_1_ui_input_route_authority",
            "runtime_09_m1_1_direct_pointer_navigation_routes_are_leaf_owner_helpers",
            "route_authority.rs",
        ],
    ),
    (
        "Runtime 09 navigation legacy reply rename",
        [
            "runtime_09_m1_2_navigation_legacy_reply_renamed_static_passed_cargo_pending",
            "`routed_reply`",
            "`ui_legacy_hits=153`",
            "standalone rustc 6/6",
        ],
    ),
    (
        "Runtime 09 pointer legacy reply rename",
        [
            "runtime_09_m1_2_pointer_legacy_reply_renamed_static_passed_cargo_pending",
            "`routed_result`",
            "`ui_legacy_hits=104`",
            "standalone rustc 10/10",
        ],
    ),
    (
        "Runtime 09 pointer capture fallback rename",
        [
            "runtime_09_m1_2_pointer_capture_fallback_renamed_static_passed_cargo_pending",
            "`has_pointer_capture_or_unindexed_fallback_for_owner`",
            "`ui_legacy_hits=102`",
            "standalone rustc 11/11",
        ],
    ),
    (
        "Runtime 09 table row label fallback rename",
        [
            "runtime_09_m1_2_table_row_label_fallback_renamed_static_passed_cargo_pending",
            "`split_row_label_table_text`",
            "`ui_legacy_hits=100`",
            "standalone rustc 12/12",
        ],
    ),
    (
        "Runtime 09 template component-name fallback rename",
        [
            "runtime_09_m1_2_template_component_name_fallback_renamed_static_passed_cargo_pending",
            "`component_name_interaction_fallback`",
            "`ui_legacy_hits=95`",
            "standalone rustc 13/13",
        ],
    ),
    (
        "Runtime 09 property visibility flag rename",
        [
            "runtime_09_m1_2_property_visibility_flag_renamed_static_passed_cargo_pending",
            "`state_visible_flag`",
            "`ui_legacy_hits=92`",
            "standalone rustc 14/14",
        ],
    ),
    (
        "Runtime 09 responsive MUI visibility flag rename",
        [
            "runtime_09_m1_2_responsive_mui_visibility_flag_renamed_static_passed_cargo_pending",
            "`state_visible_flag`",
            "`ui_legacy_hits=84`",
            "standalone rustc 15/15",
        ],
    ),
    (
        "Runtime 09 accessibility open-state fallback rename",
        [
            "runtime_09_m1_2_accessibility_open_state_fallback_renamed_static_passed_cargo_pending",
            "`fallback_properties`",
            "`ui_legacy_hits=76`",
            "standalone rustc 16/16",
        ],
    ),
    (
        "Runtime 09 layout engine backend name cutover",
        [
            "runtime_09_m1_2_layout_engine_backend_name_cutover_static_passed_cargo_pending",
            "UiLayoutEngineBackend::Zircon",
            "`zircon_selected_count`",
            "standalone rustc 17/17",
        ],
    ),
    (
        "Runtime 09 surface default interaction fallback rename",
        [
            "runtime_09_m1_2_surface_default_interaction_fallback_renamed_static_passed_cargo_pending",
            "`default_open_boolean_value`",
            "`ui_legacy_hits=54`",
            "standalone rustc 18/18",
        ],
    ),
    (
        "Runtime 09 taffy bridge pass order",
        [
            "runtime_09_m2_1_taffy_bridge_pass_order_static_passed_cargo_pending",
            "UI_LAYOUT_PASS_ORDER",
            "compute_taffy_child_frames",
            "cargo check -p zircon_runtime",
        ],
    ),
    (
        "Runtime 09 virtualization scroll boundary",
        [
            "runtime_09_m2_2_virtualization_scroll_boundary_static_passed_cargo_pending",
            "UiScrollVirtualizationPlan",
            "plan_scrollable_virtual_window",
            "scroll_virtualization.rs",
        ],
    ),
    (
        "Runtime 09 template pipeline boundary",
        [
            "runtime_09_m3_1_template_compile_instance_validate_boundary_static_passed_cargo_pending",
            "UiTemplateRuntimePipeline",
            "UI_TEMPLATE_RUNTIME_PIPELINE_STAGES",
            "template_pipeline.rs",
        ],
    ),
];
