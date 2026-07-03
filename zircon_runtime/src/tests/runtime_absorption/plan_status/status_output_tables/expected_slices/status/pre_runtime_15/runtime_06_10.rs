pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 10 F18 asset manager resolution return shape" {
        Some("runtime_10_asset_manager_resolution_handle_shape_coremin_check_passed")
    } else if slice == "Runtime 08 F17 entity path lookup verb rename" {
        Some("runtime_08_entity_path_lookup_getter_rename_coremin_check_passed")
    } else if slice == "Runtime 07 scene asset split-drift repair" {
        Some("split_drift_static_passed_cargo_deferred_active_lanes")
    } else if slice == "Runtime 07 scene asset folder-split public-surface guard" {
        Some("folder_split_guard_static_passed_cargo_deferred_active_lanes")
    } else if slice == "Runtime 07 scene asset guard 纳入 performance_hotpath_boundary" {
        Some("boundary_guard_anchor_static_passed_cargo_deferred_active_lanes")
    } else if slice == "Runtime 07 project_io folder split"
        || slice == "Runtime 10 Dynamic Session Event Split"
        || slice == "Runtime 10 Dynamic Session Test Owner Split"
    {
        Some("folder_split_static_passed_cargo_deferred_active_lanes")
    } else if slice == "Runtime 07 ECS frame diagnostics aggregation" {
        Some("ecs_frame_diagnostics_static_passed_cargo_deferred")
    } else if slice == "Runtime 07 QueryState frame auto-collection" {
        Some("system_query_frame_telemetry_static_passed_cargo_deferred")
    } else if slice == "Runtime 07 ChangeDetection frame auto-collection" {
        Some("system_change_detection_frame_telemetry_static_passed_cargo_deferred")
    } else if slice == "Runtime 07 QueryState iterator lifetime guard" {
        Some("query_state_iterator_lifetime_static_passed_cargo_timeout_no_result")
    } else if slice == "Runtime 07 FPS gate support unblock" {
        Some("fps_gate_support_unblocked_timeout_no_result")
    } else if slice == "Runtime 07 profiling build tooling" {
        Some("profiling_build_tooling_static_passed_cargo_deferred_active_lanes")
    } else if slice == "Runtime 07 Performance hotpath Markdown renderer split" {
        Some("performance_hotpath_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 07 Performance hotpath inventory split" {
        Some("performance_hotpath_inventory_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 07 scene/EventBus poison-safe locks" {
        Some("scene_level_poison_recovery_coremin_passed_eventbus_guard_timeout")
    } else if slice == "Runtime 07 render submit source-extract sharing" {
        Some("render_submit_source_extract_shared_coremin_check_passed_partial")
    } else if slice == "Runtime 07 render submit viewport/provider errors" {
        Some("render_submit_viewport_provider_errors_review_guard_static_passed_cargo_timeout_no_result_full_runtime07_pending")
    } else if slice == "Runtime 07 render camera-loop descriptor submissions" {
        Some("render_camera_loop_descriptor_submissions_coremin_check_passed_partial")
    } else if slice == "Runtime 07 render camera-loop borrowed sequence resolution" {
        Some("render_camera_loop_borrowed_sequence_resolution_static_passed_cargo_deferred")
    } else if slice == "Runtime 07 render camera-loop source view restore narrowing" {
        Some("render_camera_loop_source_view_restore_narrowed_static_passed_cargo_deferred")
    } else if slice == "Runtime 07 render camera-loop post-process source restore narrowing" {
        Some("render_camera_loop_post_process_restore_narrowed_static_passed_cargo_deferred")
    } else if slice == "Runtime 07 render camera-loop VG/HGI conditional source restore" {
        Some("render_camera_loop_vg_hgi_conditional_restore_static_passed_cargo_deferred")
    } else if slice == "Runtime 07 render camera-loop single-child source-state capture skip" {
        Some("render_camera_loop_single_child_source_state_capture_skipped_static_passed_cargo_deferred")
    } else if slice == "Runtime 07 render camera-loop source payload slot ownership" {
        Some("render_camera_loop_source_payload_slot_owned_static_passed_cargo_deferred")
    } else if slice == "Runtime 07 render camera-loop frame terminal move" {
        Some("render_camera_loop_frame_terminal_move_coremin_check_passed_partial")
    } else if slice == "Runtime 07 render submit feedback sideband owned merge" {
        Some("render_submit_feedback_sidebands_owned_merge_coremin_check_passed_partial")
    } else if slice == "Runtime 07 render prepared sideband frame owner move" {
        Some("render_prepared_sideband_frame_owner_move_coremin_check_passed_partial")
    } else if slice == "Runtime 07 render direct runtime-frame streaming camera loop" {
        Some("render_direct_runtime_frame_streaming_camera_loop_coremin_check_passed_partial")
    } else if slice == "Runtime 07 render generated camera-loop shared extract" {
        Some("render_generated_camera_loop_shared_extract_static_passed_cargo_locked_blocked")
    } else if slice == "Runtime 07 render shared effective extract frame source" {
        Some("render_shared_effective_extract_frame_source_coremin_check_passed_partial")
    } else if slice == "Runtime 07 render direct runtime-frame shared context extract" {
        Some("render_direct_runtime_frame_shared_context_extract_coremin_check_passed_partial")
    } else if slice == "Runtime 07 render VG debug overlay frame override" {
        Some("render_vg_debug_overlay_frame_override_coremin_check_passed_partial")
    } else if slice == "Runtime 07 render direct runtime-frame trace export" {
        Some("render_direct_runtime_frame_trace_export_static_passed_profile_timeout_fps_pending")
    } else if slice == "Runtime 07 render submit effective extract projection" {
        Some("render_submit_effective_extract_projection_coremin_check_passed_partial")
    } else if slice == "Runtime 07 F16 compiled-scene split status guard" {
        Some("compiled_scene_render_split_review_guard_static_passed_cargo_deferred")
    } else if slice == "Runtime 07 extract rebuild cache" {
        Some("extract_rebuild_cache_static_passed_cargo_deferred")
    } else if slice == "Runtime 07 extract cache hit/miss diagnostics" {
        Some("extract_cache_hit_miss_static_passed_cargo_deferred")
    } else if slice == "Runtime 07 asset worker frame sampler" {
        Some("asset_worker_frame_sampler_static_passed_cargo_deferred")
    } else if slice == "Runtime 07 artifact cache payload owner split" {
        Some("artifact_cache_payload_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 07 render product diagnostics owner split" {
        Some("render_product_diagnostics_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 07 animation scene frame diagnostics" {
        Some("animation_scene_frame_diagnostics_static_passed_cargo_deferred")
    } else if slice == "Runtime 07 profile counter hotspot export" {
        Some("profiling_counter_hotspot_export_static_passed_cargo_deferred")
    } else if slice == "Runtime 07 QueryState cache owner performance audit sync" {
        Some("query_state_cache_owner_perf_audit_sync_static_passed_cargo_pending")
    } else if slice == "Runtime 07 virtual geometry debug snapshot owner split" {
        Some("virtual_geometry_debug_snapshot_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 07 owner-budget current doc mirror fix" {
        Some("mirror_docs_static_passed_cargo_pending")
    } else if slice == "Runtime 07 owner-budget 36-hotspot navigation split sync"
        || slice == "Runtime 07 owner-budget 30-hotspot current audit sync"
    {
        Some("mirror_docs_static_passed_cargo_pending")
    } else if slice == "Runtime 07 owner-budget 0-hotspot current audit sync" {
        Some("mirror_docs_static_passed_cargo_deferred")
    } else if slice == "Runtime 09 UI input route authority" {
        Some("runtime_09_m1_1_ui_input_route_authority_static_passed_cargo_pending")
    } else if slice == "Runtime 09 navigation legacy reply rename" {
        Some("runtime_09_m1_2_navigation_legacy_reply_renamed_static_passed_cargo_pending")
    } else if slice == "Runtime 09 pointer legacy reply rename" {
        Some("runtime_09_m1_2_pointer_legacy_reply_renamed_static_passed_cargo_pending")
    } else if slice == "Runtime 09 pointer capture fallback rename" {
        Some("runtime_09_m1_2_pointer_capture_fallback_renamed_static_passed_cargo_pending")
    } else if slice == "Runtime 09 table row label fallback rename" {
        Some("runtime_09_m1_2_table_row_label_fallback_renamed_static_passed_cargo_pending")
    } else if slice == "Runtime 09 template component-name fallback rename" {
        Some("runtime_09_m1_2_template_component_name_fallback_renamed_static_passed_cargo_pending")
    } else if slice == "Runtime 09 property visibility flag rename" {
        Some("runtime_09_m1_2_property_visibility_flag_renamed_static_passed_cargo_pending")
    } else if slice == "Runtime 09 responsive MUI visibility flag rename" {
        Some("runtime_09_m1_2_responsive_mui_visibility_flag_renamed_static_passed_cargo_pending")
    } else if slice == "Runtime 09 accessibility open-state fallback rename" {
        Some(
            "runtime_09_m1_2_accessibility_open_state_fallback_renamed_static_passed_cargo_pending",
        )
    } else if slice == "Runtime 09 layout engine backend name cutover" {
        Some("runtime_09_m1_2_layout_engine_backend_name_cutover_static_passed_cargo_pending")
    } else if slice == "Runtime 09 surface default interaction fallback rename" {
        Some("runtime_09_m1_2_surface_default_interaction_fallback_renamed_static_passed_cargo_pending")
    } else if slice == "Runtime 09 UI architecture Markdown renderer split" {
        Some("ui_architecture_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 09 UI entry map audit sync" {
        Some("runtime_09_ui_entry_map_audit_sync_static_passed_cargo_deferred")
    } else if slice == "Runtime 09 taffy bridge pass order" {
        Some("runtime_09_m2_1_taffy_bridge_pass_order_static_passed_cargo_pending")
    } else if slice == "Runtime 09 virtualization scroll boundary" {
        Some("runtime_09_m2_2_virtualization_scroll_boundary_static_passed_cargo_pending")
    } else if slice == "Runtime 09 template pipeline boundary" {
        Some("runtime_09_m3_1_template_compile_instance_validate_boundary_static_passed_cargo_pending")
    } else if slice == "Runtime 10 UI contract duplicate public types cleanup" {
        Some("runtime_10_m2_1_ui_contract_duplicate_public_types_removed_static_passed_cargo_pending")
    } else if slice == "Runtime 10 UI v2 contract sync" {
        Some("runtime_10_m2_2_ui_v2_contract_sync_static_passed_cargo_pending")
    } else if slice == "Runtime 10 host-request payload ABI boundary" {
        Some("host_request_payload_boundary_static_passed_cargo_pending")
    } else if slice == "Runtime 10 Dynamic API current audit recheck" {
        Some("dynamic_api_current_audit_static_passed_cargo_pending")
    } else if slice == "Runtime 10 Dynamic API 2026-07-01 current audit recheck" {
        Some("dynamic_api_20260701_current_audit_static_passed_cargo_deferred")
    } else if slice == "Runtime 10 Dynamic API test boundary Markdown renderer split" {
        Some("dynamic_api_test_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 10 dynamic_api_session Cargo 验证窗口探测" {
        Some("cargo_recheck_timeout_static_guards_passed")
    } else if slice == "Runtime 10 runtime diagnostics profile-control snapshot" {
        Some("runtime_diagnostics_profile_control_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 10 diagnostics inventory split" {
        Some("runtime_10_dynamic_api_diagnostics_inventory_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 10 host-request inventory split" {
        Some("runtime_10_host_request_payload_inventory_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 10 UI contract inventory split" {
        Some("runtime_10_ui_contract_inventory_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 10 validation inventory split" {
        Some("runtime_10_dynamic_api_validation_inventory_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 10 session lifecycle inventory split" {
        Some("runtime_10_session_lifecycle_inventory_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 10 session profile owner audit sync" {
        Some("runtime_10_session_profile_owner_audit_sync_static_passed_cargo_deferred")
    } else if slice == "Runtime 10 host-request payload test owner split" {
        Some("runtime_10_host_request_payload_test_owner_split_static_passed_cargo_timeout_no_result")
    } else if slice == "Runtime 10 failure boundary inventory split" {
        Some("runtime_10_failure_boundary_inventory_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 10 ABI source inventory split" {
        Some("runtime_10_dynamic_api_abi_inventory_split_static_passed_cargo_timeout_no_result_tests_deferred")
    } else if slice == "Runtime 10 runtime API Markdown renderer split" {
        Some("runtime_api_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 10 dynamic runtime API Markdown renderer split" {
        Some("dynamic_runtime_api_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 10 dynamic input mouse-wheel event owner guard" {
        Some("dynamic_input_mouse_wheel_event_owner_guard_focused_cargo_passed_broader_input_pending")
    } else if slice == "Runtime 10 Vampire W input real-backend gate" {
        Some("dynamic_vampire_w_input_real_backend_gate_ignored_without_zr_vm_remaining_ui_input_pending")
    } else if slice == "Runtime 06 plugin surface/lifecycle Markdown renderer split" {
        Some("plugin_surface_lifecycle_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 06 native plugin public-surface Markdown renderer split" {
        Some("native_plugin_public_surface_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 06 native hot-update/replay public-surface audit sync" {
        Some("runtime_06_native_hot_update_replay_public_surface_audit_sync_static_passed_cargo_deferred")
    } else if slice == "Runtime 06 F8 RuntimePluginDescriptor builder scaffold" {
        Some("runtime_plugin_descriptor_builder_scaffold_coremin_check_passed")
    } else if slice == "Runtime 06 F8 first-party RuntimePluginDescriptor builder migration" {
        Some("runtime_plugin_descriptor_first_party_builder_migration_coremin_check_passed")
    } else if slice == "Runtime 06 F8 RuntimePluginDescriptor test fixture builder migration" {
        Some("runtime_plugin_descriptor_test_fixture_builder_migration_coremin_check_passed")
    } else if slice == "Runtime 06 F8 RuntimePluginDescriptor public-field convergence" {
        Some("runtime_plugin_descriptor_public_field_convergence_coremin_check_passed")
    } else if slice == "Runtime 06 F8 RuntimePluginDescriptor public constructor retired" {
        Some("runtime_plugin_descriptor_public_constructor_retired_coremin_check_passed")
    } else if slice == "Runtime 06 plugin::native hard-cutover" {
        Some("code_static_passed_cargo_pending")
    } else if slice == "Runtime 06 fallback lifecycle failure tests" {
        Some("code_static_passed_real_backend_pending")
    } else if slice == "Runtime 06 fallback lifecycle Cargo 验证" {
        Some("fallback_cargo_passed_real_backend_pending")
    } else if slice == "Runtime 06 shader artifact cache real-backend unblock" {
        Some("asset_cache_fixed_vampire_session_pending")
    } else if slice == "Runtime 06 Vampire real-backend menu/retry focused validation" {
        Some("vampire_real_backend_focused_passed_full_gate_pending")
    } else if slice == "Runtime 06 Vampire HUD real-backend capture validation" {
        Some("vampire_hud_real_backend_focused_passed_full_gate_pending")
    } else if slice == "Runtime 06 native loader test namespace migration" {
        Some("code_static_passed_cargo_pending")
    } else if slice == "Runtime 06 V1/V2 ABI hard-cutover" {
        Some("code_static_passed_cargo_pending")
    } else if slice == "Runtime 06 hot reload failure injection" {
        Some("code_static_passed_cargo_pending")
    } else if slice == "Runtime 10 dynamic_api_session 吸收守卫拆分" {
        Some("focused_cargo_passed_broader_gates_pending")
    } else if slice == "Runtime 08 First-stage event update guard" {
        Some("mirror_docs_static_passed_cargo_pending")
    } else if slice == "Runtime 08 ECS 数据面 current audit recheck" {
        Some("ecs_kernel_data_current_audit_static_passed_cargo_pending")
    } else if slice == "Runtime 08 ECS 数据面 2026-07-01 current audit recheck" {
        Some("ecs_kernel_data_20260701_current_audit_static_passed_cargo_deferred")
    } else if slice == "Runtime 08 ECS source/test inventory split" {
        Some("ecs_kernel_data_source_inventory_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 08 ECS anchor inventory split" {
        Some("ecs_kernel_data_anchor_inventory_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 08 ECS markdown renderer split" {
        Some("ecs_kernel_data_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 08 QueryState Markdown renderer split" {
        Some("ecs_query_state_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 08 QueryState many_item_array audit sync" {
        Some("runtime_08_query_state_many_item_array_audit_sync_static_passed_cargo_deferred")
    } else if slice == "Runtime 08 ECS hard-cutover owner inventory sync" {
        Some("runtime_08_ecs_hard_cutover_owner_inventory_sync_static_passed_cargo_deferred")
    } else if slice == "Runtime 08 F5 world typed mutation errors" {
        Some("world_typed_mutation_errors_coremin_check_passed_partial")
    } else if slice == "Runtime 08 F5 dynamic component typed errors" {
        Some("dynamic_component_typed_errors_coremin_check_passed")
    } else if slice == "Runtime 08 QueryState cache owner split" {
        Some("query_state_cache_owner_split_static_passed_cargo_pending")
    } else if slice == "Runtime 08 ECS event owner folder split" {
        Some("ecs_events_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 08 ECS message owner folder split" {
        Some("ecs_messages_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 08 ECS resource store owner folder split" {
        Some("ecs_resource_store_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 08 ECS resource identity owner folder split" {
        Some("ecs_resource_identity_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 08 ECS component identity owner folder split" {
        Some("ecs_component_identity_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 08 ECS entity identity owner folder split" {
        Some("ecs_entity_identity_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 08 ECS archetype owner folder split" {
        Some("ecs_archetype_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 08 ECS component storage owner folder split" {
        Some("ecs_component_storage_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 08 ECS component storage private re-export cleanup" {
        Some("ecs_component_storage_private_reexport_cargo_check_passed")
    } else if slice == "Runtime 08 ECS observer owner folder split" {
        Some("ecs_observer_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 08 ECS commands facade owner split" {
        Some("ecs_commands_facade_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 08 ECS command Cargo 验证窗口探测" {
        Some("cargo_recheck_timeout_no_result")
    } else if slice == "Runtime 08 ECS entity Cargo 验证窗口探测" {
        Some("cargo_recheck_timeout_no_result")
    } else if slice == "Runtime 08 ECS data owner-tree guard" {
        Some("ecs_data_owner_tree_guard_static_passed_cargo_pending")
    } else if slice == "Runtime 08 ECS change detection owner-tree guard" {
        Some("ecs_change_detection_owner_tree_guard_static_passed_cargo_pending")
    } else if slice == "Runtime 08 ECS root leaf owner guard" {
        Some("ecs_root_leaf_owner_guard_static_passed_cargo_pending")
    } else if slice == "Runtime 08 ecs_events_messages Cargo 验证窗口探测" {
        Some("cargo_recheck_timeout_no_result")
    } else {
        None
    }
}
