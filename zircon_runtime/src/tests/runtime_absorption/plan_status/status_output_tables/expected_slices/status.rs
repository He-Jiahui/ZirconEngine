pub(super) fn expected_status_for_slice(slice: &str) -> &'static str {
    if slice == "Runtime 14 Cargo 验证窗口探测" {
        "cargo_deferred_active_lane"
    } else if slice == "Runtime 14 animation Cargo gate 尝试" {
        "cargo_blocked_external_compile_drift"
    } else if slice == "Runtime 14 animation Cargo gate 修复与复验阻塞" {
        "cargo_recheck_blocked_external_ui_compile_drift"
    } else if slice == "Runtime 14 animation runtime-status focused recheck timeout" {
        "cargo_recheck_timeout_no_result"
    } else if slice == "Runtime 14 animation family 28-file audit sync" {
        "module_family_source_count_static_passed_cargo_pending"
    } else if slice == "Runtime 14 navigation fallback runtime owner split" {
        "navigation_runtime_owner_split_static_passed_cargo_pending"
    } else if slice == "Runtime 14 module family current audit recheck" {
        "module_family_current_audit_static_passed_cargo_pending"
    } else if slice == "Runtime 05 plan-status Cargo attempt 状态审计" {
        "cargo_attempt_status_static_passed_cargo_pending"
    } else if slice == "Runtime 05 plan-status Cargo timeout 状态审计" {
        "cargo_attempt_timeout_status_static_passed_cargo_pending"
    } else if slice == "Runtime 01 export build-plan directory materialization boundary" {
        "export_materialize_owner_split_static_passed_cargo_deferred_tests_deferred"
    } else if slice == "Runtime 01 NativeDynamic materialization symlink boundary" {
        "export_materialize_symlink_boundary_static_passed_cargo_deferred_tests_deferred"
    } else if slice == "Runtime 01 export materialization dry-run preview" {
        "export_materialize_preview_static_passed_cargo_deferred_tests_deferred"
    } else if slice == "Runtime 01 export materialization fatal preflight gate" {
        "export_materialize_fatal_gate_static_passed_cargo_deferred_tests_deferred"
    } else if slice == "Runtime 01 editor native-aware fatal export early exit" {
        "editor_native_aware_export_fatal_gate_static_passed_cargo_deferred_tests_deferred"
    } else if slice == "Runtime 01 editor native-aware discovery reuse" {
        "editor_native_aware_export_discovery_reuse_static_passed_cargo_deferred_tests_deferred"
    } else if slice == "Runtime 01 export ZIP archive materialization" {
        "export_archive_zip_materialization_static_passed_cargo_deferred_tests_deferred"
    } else if slice == "Runtime 01 Tech-stack current audit recheck" {
        "tech_stack_current_audit_static_passed_cargo_pending"
    } else if slice == "Runtime 05 full scene closeout failed evidence" {
        "cargo_recheck_failed_full_scene_gate"
    } else if slice == "Runtime 05 full scene compile-pass graphics-scene blocker" {
        "runtime_05_full_scene_gate_compile_passed_graphics_scene_tests_failed_scene_gate_pending"
    } else if slice == "Runtime 05 full scene closeout no-result recheck" {
        "cargo_recheck_no_result_external_editor_lane"
    } else if slice == "Runtime 05 scene:: failure support-first triage" {
        "support_first_triage_static_passed_cargo_pending"
    } else if slice == "Runtime 05 scene:: lower-layer diagnostic matrix" {
        "support_first_matrix_static_passed_cargo_pending"
    } else if slice == "Runtime 05 scene:: diagnostic matrix source anchors" {
        "support_first_matrix_source_anchors_static_passed_cargo_pending"
    } else if slice == "Runtime 05 serialization source folder-split guard sync" {
        "source_guard_static_passed_cargo_deferred_active_lanes"
    } else if slice == "Runtime 05 editor_projection residual guard verdict" {
        "editor_projection_residual_guard_static_passed_cargo_pending"
    } else if slice == "Runtime 05 dynamic scene session owner-tree guard" {
        "dynamic_scene_session_owner_tree_guard_static_passed_cargo_pending"
    } else if slice == "Runtime 05 dynamic scene root owner-tree guard" {
        "dynamic_scene_root_owner_tree_guard_static_passed_cargo_pending"
    } else if slice == "Runtime 05 dynamic scene root scene owner split" {
        "dynamic_scene_scene_owner_split_static_passed_cargo_deferred_tests_deferred"
    } else if slice == "Runtime 05 dynamic scene spawn task owner split" {
        "dynamic_scene_spawn_task_owner_split_static_passed_cargo_deferred_tests_deferred"
    } else if slice == "Runtime 05 dynamic scene value conversion owner split" {
        "dynamic_scene_value_owner_split_static_passed_cargo_deferred_tests_deferred"
    } else if slice == "Runtime 05 dynamic scene entity declaration owner split" {
        "dynamic_scene_entity_owner_split_static_passed_cargo_deferred_tests_deferred"
    } else if slice == "Runtime 05 dynamic scene scene-asset bridge owner split" {
        "dynamic_scene_scene_asset_bridge_owner_split_static_passed_cargo_deferred_tests_deferred"
    } else if slice == "Runtime 05 dynamic scene document serialization owner split" {
        "dynamic_scene_document_owner_split_static_passed_cargo_deferred_tests_deferred"
    } else if slice == "Runtime 05 dynamic scene patch preview API" {
        "dynamic_scene_patch_preview_api_static_passed_cargo_timeout_no_result_tests_deferred"
    } else if slice == "Runtime 05 dynamic scene patch preview status guard" {
        "dynamic_scene_patch_preview_status_guard_static_passed_cargo_pending"
    } else if slice
        == "Runtime 05 dynamic scene patch preview resource preflight details status guard"
    {
        "dynamic_scene_patch_preview_resource_preflight_details_status_guard_static_passed_cargo_pending"
    } else if slice
        == "Runtime 05 dynamic scene patch preview resource ensure creation status guard"
    {
        "dynamic_scene_patch_preview_resource_ensure_creation_status_guard_static_passed_cargo_pending"
    } else if slice
        == "Runtime 05 dynamic scene patch preview component type install details status guard"
    {
        "dynamic_scene_patch_preview_component_type_install_details_status_guard_static_passed_cargo_pending"
    } else if slice
        == "Runtime 05 dynamic scene patch preview component type install counts status guard"
    {
        "dynamic_scene_patch_preview_component_type_install_counts_status_guard_static_passed_cargo_pending"
    } else if slice == "Runtime 05 dynamic scene patch preview reflection preflight status guard" {
        "dynamic_scene_patch_preview_reflection_preflight_status_guard_static_passed_cargo_pending"
    } else if slice == "Runtime 05 dynamic scene patch preview component workload status guard" {
        "dynamic_scene_patch_preview_component_workload_status_guard_static_passed_cargo_pending"
    } else if slice == "Runtime 05 dynamic scene patch preview remap status guard" {
        "dynamic_scene_patch_preview_remap_status_guard_static_passed_cargo_pending"
    } else if slice == "Runtime 03 world bootstrap fixed-loop stage guard sync" {
        "guard_sync_static_passed_cargo_pending"
    } else if slice == "Runtime 03 Schedule/frame-loop current audit recheck" {
        "schedule_frame_loop_current_audit_static_passed_cargo_pending"
    } else if slice == "Runtime 07 scene asset split-drift repair" {
        "split_drift_static_passed_cargo_deferred_active_lanes"
    } else if slice == "Runtime 07 scene asset folder-split public-surface guard" {
        "folder_split_guard_static_passed_cargo_deferred_active_lanes"
    } else if slice == "Runtime 07 scene asset guard 纳入 performance_hotpath_boundary" {
        "boundary_guard_anchor_static_passed_cargo_deferred_active_lanes"
    } else if slice == "Runtime 07 project_io folder split"
        || slice == "Runtime 10 Dynamic Session Event Split"
        || slice == "Runtime 10 Dynamic Session Test Owner Split"
    {
        "folder_split_static_passed_cargo_deferred_active_lanes"
    } else if slice == "Runtime 07 ECS frame diagnostics aggregation" {
        "ecs_frame_diagnostics_static_passed_cargo_deferred"
    } else if slice == "Runtime 07 QueryState frame auto-collection" {
        "system_query_frame_telemetry_static_passed_cargo_deferred"
    } else if slice == "Runtime 07 ChangeDetection frame auto-collection" {
        "system_change_detection_frame_telemetry_static_passed_cargo_deferred"
    } else if slice == "Runtime 07 QueryState iterator lifetime guard" {
        "query_state_iterator_lifetime_static_passed_cargo_timeout_no_result"
    } else if slice == "Runtime 07 FPS gate support unblock" {
        "fps_gate_support_unblocked_timeout_no_result"
    } else if slice == "Runtime 07 profiling build tooling" {
        "profiling_build_tooling_static_passed_cargo_deferred_active_lanes"
    } else if slice == "Runtime 07 extract rebuild cache" {
        "extract_rebuild_cache_static_passed_cargo_deferred"
    } else if slice == "Runtime 07 extract cache hit/miss diagnostics" {
        "extract_cache_hit_miss_static_passed_cargo_deferred"
    } else if slice == "Runtime 07 asset worker frame sampler" {
        "asset_worker_frame_sampler_static_passed_cargo_deferred"
    } else if slice == "Runtime 04 worker-pool manager frame sampler entry"
        || slice == "Runtime 07 asset worker manager sampler entry"
    {
        "asset_worker_manager_sampler_static_passed_cargo_deferred"
    } else if slice == "Runtime 04 asset worker request entry hard-cutover" {
        "asset_worker_request_sender_hard_cutover_static_passed_cargo_deferred"
    } else if slice == "Runtime 04 Asset pipeline current audit recheck" {
        "asset_pipeline_current_audit_static_passed_cargo_pending"
    } else if slice == "Runtime 07 artifact cache payload owner split" {
        "artifact_cache_payload_owner_split_static_passed_cargo_deferred"
    } else if slice == "Runtime 07 render product diagnostics owner split" {
        "render_product_diagnostics_owner_split_static_passed_cargo_deferred"
    } else if slice == "Runtime 07 animation scene frame diagnostics" {
        "animation_scene_frame_diagnostics_static_passed_cargo_deferred"
    } else if slice == "Runtime 07 profile counter hotspot export" {
        "profiling_counter_hotspot_export_static_passed_cargo_deferred"
    } else if slice == "Runtime 07 QueryState cache owner performance audit sync" {
        "query_state_cache_owner_perf_audit_sync_static_passed_cargo_pending"
    } else if slice == "Runtime 07 virtual geometry debug snapshot owner split" {
        "virtual_geometry_debug_snapshot_owner_split_static_passed_cargo_deferred"
    } else if slice == "Runtime 07 owner-budget current doc mirror fix" {
        "mirror_docs_static_passed_cargo_pending"
    } else if slice == "Runtime 07 owner-budget 36-hotspot navigation split sync"
        || slice == "Runtime 07 owner-budget 30-hotspot current audit sync"
    {
        "mirror_docs_static_passed_cargo_pending"
    } else if slice == "Runtime 13 Gameplay Host Owner Split" {
        "folder_split_static_passed_script_vm_cargo_broader_gate_pending"
    } else if slice == "Runtime 13 Script binding current audit recheck" {
        "script_binding_current_audit_static_passed_cargo_pending"
    } else if slice == "Runtime 02 generated template count 审计同步" {
        "structure_audit_static_passed_cargo_pending"
    } else if slice == "Runtime 02 root graphics alias block removal" {
        "graphics_alias_block_removed_static_passed_cargo_pending"
    } else if slice == "Runtime 02 rhi_wgpu root backend private cutover" {
        "rhi_wgpu_root_backend_private_static_passed_cargo_pending"
    } else if slice == "Runtime 02 builtin root facade cutover" {
        "builtin_root_facade_removed_static_passed_cargo_pending"
    } else if slice == "Runtime 02 core/root/generated current audit recheck" {
        "core_root_generated_current_audit_static_passed_cargo_pending"
    } else if slice == "Runtime 11 graphics frustum rayon cutover" {
        "runtime_11_m2_1_graphics_frustum_rayon_cutover_static_passed_cargo_pending"
    } else if slice == "Runtime 11 scheduler wait_all 同步点" {
        "wait_all_static_passed_cargo_pending"
    } else if slice == "Runtime 11 panic-safe handle completion" {
        "panic_safe_completion_static_passed_cargo_deferred"
    } else if slice == "Runtime 11 JobSystem 2026-06-20 验证窗口探测" {
        "cargo_recheck_timeout_static_guards_passed"
    } else if slice == "Runtime 11 JobSystem core-min 验证窗口探测" {
        "core_min_cargo_recheck_timeout_static_guards_passed"
    } else if slice == "Runtime 11 JobSystem current audit recheck" {
        "job_system_current_audit_static_passed_cargo_pending"
    } else if slice == "Runtime 09 UI input route authority" {
        "runtime_09_m1_1_ui_input_route_authority_static_passed_cargo_pending"
    } else if slice == "Runtime 09 navigation legacy reply rename" {
        "runtime_09_m1_2_navigation_legacy_reply_renamed_static_passed_cargo_pending"
    } else if slice == "Runtime 09 pointer legacy reply rename" {
        "runtime_09_m1_2_pointer_legacy_reply_renamed_static_passed_cargo_pending"
    } else if slice == "Runtime 09 pointer capture fallback rename" {
        "runtime_09_m1_2_pointer_capture_fallback_renamed_static_passed_cargo_pending"
    } else if slice == "Runtime 09 table row label fallback rename" {
        "runtime_09_m1_2_table_row_label_fallback_renamed_static_passed_cargo_pending"
    } else if slice == "Runtime 09 template component-name fallback rename" {
        "runtime_09_m1_2_template_component_name_fallback_renamed_static_passed_cargo_pending"
    } else if slice == "Runtime 09 property visibility flag rename" {
        "runtime_09_m1_2_property_visibility_flag_renamed_static_passed_cargo_pending"
    } else if slice == "Runtime 09 responsive MUI visibility flag rename" {
        "runtime_09_m1_2_responsive_mui_visibility_flag_renamed_static_passed_cargo_pending"
    } else if slice == "Runtime 09 accessibility open-state fallback rename" {
        "runtime_09_m1_2_accessibility_open_state_fallback_renamed_static_passed_cargo_pending"
    } else if slice == "Runtime 09 layout engine backend name cutover" {
        "runtime_09_m1_2_layout_engine_backend_name_cutover_static_passed_cargo_pending"
    } else if slice == "Runtime 09 surface default interaction fallback rename" {
        "runtime_09_m1_2_surface_default_interaction_fallback_renamed_static_passed_cargo_pending"
    } else if slice == "Runtime 09 taffy bridge pass order" {
        "runtime_09_m2_1_taffy_bridge_pass_order_static_passed_cargo_pending"
    } else if slice == "Runtime 09 virtualization scroll boundary" {
        "runtime_09_m2_2_virtualization_scroll_boundary_static_passed_cargo_pending"
    } else if slice == "Runtime 09 template pipeline boundary" {
        "runtime_09_m3_1_template_compile_instance_validate_boundary_static_passed_cargo_pending"
    } else if slice == "Runtime 10 UI contract duplicate public types cleanup" {
        "runtime_10_m2_1_ui_contract_duplicate_public_types_removed_static_passed_cargo_pending"
    } else if slice == "Runtime 10 UI v2 contract sync" {
        "runtime_10_m2_2_ui_v2_contract_sync_static_passed_cargo_pending"
    } else if slice == "Runtime 10 host-request payload ABI boundary" {
        "host_request_payload_boundary_static_passed_cargo_pending"
    } else if slice == "Runtime 10 Dynamic API current audit recheck" {
        "dynamic_api_current_audit_static_passed_cargo_pending"
    } else if slice == "Runtime 10 dynamic_api_session Cargo 验证窗口探测" {
        "cargo_recheck_timeout_static_guards_passed"
    } else if slice == "Runtime 10 runtime diagnostics profile-control snapshot" {
        "runtime_diagnostics_profile_control_static_passed_cargo_deferred_tests_deferred"
    } else if slice == "Runtime 10 diagnostics inventory split" {
        "runtime_10_dynamic_api_diagnostics_inventory_split_static_passed_cargo_deferred_tests_deferred"
    } else if slice == "Runtime 10 host-request inventory split" {
        "runtime_10_host_request_payload_inventory_split_static_passed_cargo_deferred_tests_deferred"
    } else if slice == "Runtime 10 UI contract inventory split" {
        "runtime_10_ui_contract_inventory_split_static_passed_cargo_deferred_tests_deferred"
    } else if slice == "Runtime 10 validation inventory split" {
        "runtime_10_dynamic_api_validation_inventory_split_static_passed_cargo_deferred_tests_deferred"
    } else if slice == "Runtime 10 session lifecycle inventory split" {
        "runtime_10_session_lifecycle_inventory_split_static_passed_cargo_deferred_tests_deferred"
    } else if slice == "Runtime 10 failure boundary inventory split" {
        "runtime_10_failure_boundary_inventory_split_static_passed_cargo_deferred_tests_deferred"
    } else if slice == "Runtime 10 ABI source inventory split" {
        "runtime_10_dynamic_api_abi_inventory_split_static_passed_cargo_timeout_no_result_tests_deferred"
    } else if slice == "Runtime 06 plugin::native hard-cutover" {
        "code_static_passed_cargo_pending"
    } else if slice == "Runtime 06 fallback lifecycle failure tests" {
        "code_static_passed_real_backend_pending"
    } else if slice == "Runtime 06 fallback lifecycle Cargo 验证" {
        "fallback_cargo_passed_real_backend_pending"
    } else if slice == "Runtime 06 shader artifact cache real-backend unblock" {
        "asset_cache_fixed_vampire_session_pending"
    } else if slice == "Runtime 06 Vampire real-backend menu/retry focused validation" {
        "vampire_real_backend_focused_passed_full_gate_pending"
    } else if slice == "Runtime 06 Vampire HUD real-backend capture validation" {
        "vampire_hud_real_backend_focused_passed_full_gate_pending"
    } else if slice == "Runtime 06 native loader test namespace migration" {
        "code_static_passed_cargo_pending"
    } else if slice == "Runtime 06 V1/V2 ABI hard-cutover" {
        "code_static_passed_cargo_pending"
    } else if slice == "Runtime 06 hot reload failure injection" {
        "code_static_passed_cargo_pending"
    } else if matches!(
        slice,
        "Runtime 05 recent-static Runtime 02/07 status metadata guard"
            | "Runtime 05 status-output recent-static metadata row"
            | "Runtime 05 status-output Runtime 12 gamepad event-owner row"
            | "Runtime 05 status-output Runtime 12 behavior-test row"
            | "Runtime 05 status-output Runtime 04 behavior-test row"
            | "Runtime 05 status-output Runtime 08 behavior-test row"
            | "Runtime 05 status-output Runtime 10 behavior-test row"
            | "Runtime 05 plan-status output-anchor module split"
            | "Runtime 05 plan-status output-anchor budget guard"
            | "Runtime 05 status-output status/date helper split"
            | "Runtime 05 status-output expected anchor split"
            | "Runtime 05 plan-status root module split"
            | "Runtime 05 plan-status support inventory split"
            | "Runtime 05 plan-status anchor inventory split"
            | "Runtime 05 plan-status markdown renderer split"
            | "Runtime 05 plan-status source helper split"
            | "Runtime 05 status-output expected row data split"
            | "Runtime 05 status-output Runtime 05 row-data family split"
            | "Runtime 05 status-output audit-metadata owner split"
            | "Runtime 05 status-output Runtime 14 row-data family split"
            | "Runtime 05 status-output Runtime 07 row-data family split"
            | "Runtime 05 status-output Runtime 09 row-data family split"
            | "Runtime 05 status-output Runtime 10 row-data family split"
            | "Runtime 05 status-output Runtime 12 row-data family split"
            | "Runtime 05 status-output support-structure owner split"
            | "Runtime 05 status-output scene-closeout owner split"
            | "Runtime 05 status-output cargo-gates owner split"
            | "Runtime 05 status-output status/date owner split"
            | "Runtime 05 cargo-gates early Runtime 03 split"
            | "Runtime 05 cargo-gates early Runtime 01 split"
            | "Runtime 05 cargo-gates early Runtime 02 split"
            | "Runtime 05 cargo-gates early Runtime 04 split"
            | "Runtime 05 cargo-gates early Runtime 06 split"
            | "Runtime 05 cargo-gates early Runtime 08 split"
            | "Runtime 05 cargo-gates early Runtime 07 split"
            | "Runtime 05 cargo-gates late Runtime 10 split"
            | "Runtime 05 cargo-gates late Runtime 11 split"
            | "Runtime 05 cargo-gates late Runtime 12 split"
            | "Runtime 05 cargo-gates late Runtime 13 split"
            | "Runtime 05 cargo-gates late Runtime 14 split"
            | "Runtime 05 status-output Runtime 01-04 row-data group split"
            | "Runtime 05 status-output Runtime 06-09 row-data group split"
            | "Runtime 05 status-output Runtime 10-13 row-data group split"
            | "Runtime 05 plan-status 输出表守卫"
            | "Runtime 05 plan-status 审计元数据守卫"
            | "Runtime 05 status-output Runtime 07 scene asset rows"
            | "Runtime 05 Runtime 07 scene status 审计元数据"
            | "Runtime 05 status-output Runtime 02 generated template row"
            | "Runtime 05 Runtime 02 generated status 审计元数据"
            | "Runtime 05 status-output current anchor fix"
            | "Runtime 05 Runtime 02 root_entries count 状态表闭环"
            | "Runtime 05 status-output Runtime 07 owner-budget row"
            | "Runtime 05 plan-status owner-budget current mirror fix"
            | "Runtime 05 Runtime 07 owner-budget status 审计元数据"
            | "Runtime 05 status-output Runtime 03 module-doc row"
            | "Runtime 05 status-output Runtime 03 behavior-test row"
            | "Runtime 05 status-output all-index-row coverage guard"
            | "Runtime 05 status-output non-network server allowlist row"
    ) {
        "status_table_static_passed_cargo_pending"
    } else if slice == "Runtime 05 M0 absorption guard coverage sync" {
        "static_docs_passed_cargo_pending"
    } else if slice == "Runtime 05 non-network server UI sortingMode allowlist" {
        "static_audit_passed_cargo_pending"
    } else if slice == "Runtime 05 naming_boundary non-network server Rust guard" {
        "naming_guard_static_passed_cargo_pending"
    } else if slice == "Runtime 05 texture importer DDS caps policy wording" {
        "hard_cutover_dds_caps_policy_static_passed_cargo_pending"
    } else if slice == "Runtime 05 status-output row-data group split" {
        "status_table_static_passed_cargo_pending"
    } else if slice == "Runtime 10 dynamic_api_session 吸收守卫拆分" {
        "focused_cargo_passed_broader_gates_pending"
    } else if slice == "Runtime 13 Gameplay host predicate functions for real ZR VM" {
        "focused_behavior_passed_broader_script_gate_pending"
    } else if slice == "Runtime 08 First-stage event update guard" {
        "mirror_docs_static_passed_cargo_pending"
    } else if slice == "Runtime 08 ECS 数据面 current audit recheck" {
        "ecs_kernel_data_current_audit_static_passed_cargo_pending"
    } else if slice == "Runtime 08 ECS source/test inventory split" {
        "ecs_kernel_data_source_inventory_split_static_passed_cargo_deferred_tests_deferred"
    } else if slice == "Runtime 08 ECS anchor inventory split" {
        "ecs_kernel_data_anchor_inventory_split_static_passed_cargo_deferred_tests_deferred"
    } else if slice == "Runtime 08 QueryState cache owner split" {
        "query_state_cache_owner_split_static_passed_cargo_pending"
    } else if slice == "Runtime 08 ECS event owner folder split" {
        "ecs_events_folder_split_static_passed_cargo_deferred"
    } else if slice == "Runtime 08 ECS message owner folder split" {
        "ecs_messages_folder_split_static_passed_cargo_deferred"
    } else if slice == "Runtime 08 ECS resource store owner folder split" {
        "ecs_resource_store_folder_split_static_passed_cargo_deferred"
    } else if slice == "Runtime 08 ECS resource identity owner folder split" {
        "ecs_resource_identity_folder_split_static_passed_cargo_deferred"
    } else if slice == "Runtime 08 ECS component identity owner folder split" {
        "ecs_component_identity_folder_split_static_passed_cargo_deferred"
    } else if slice == "Runtime 08 ECS entity identity owner folder split" {
        "ecs_entity_identity_folder_split_static_passed_cargo_deferred"
    } else if slice == "Runtime 08 ECS archetype owner folder split" {
        "ecs_archetype_owner_split_static_passed_cargo_deferred"
    } else if slice == "Runtime 08 ECS component storage owner folder split" {
        "ecs_component_storage_owner_split_static_passed_cargo_deferred"
    } else if slice == "Runtime 08 ECS component storage private re-export cleanup" {
        "ecs_component_storage_private_reexport_cargo_check_passed"
    } else if slice == "Runtime 08 ECS observer owner folder split" {
        "ecs_observer_owner_split_static_passed_cargo_deferred"
    } else if slice == "Runtime 08 ECS commands facade owner split" {
        "ecs_commands_facade_owner_split_static_passed_cargo_deferred"
    } else if slice == "Runtime 08 ECS command Cargo 验证窗口探测" {
        "cargo_recheck_timeout_no_result"
    } else if slice == "Runtime 08 ECS entity Cargo 验证窗口探测" {
        "cargo_recheck_timeout_no_result"
    } else if slice == "Runtime 08 ECS data owner-tree guard" {
        "ecs_data_owner_tree_guard_static_passed_cargo_pending"
    } else if slice == "Runtime 08 ECS change detection owner-tree guard" {
        "ecs_change_detection_owner_tree_guard_static_passed_cargo_pending"
    } else if slice == "Runtime 08 ECS root leaf owner guard" {
        "ecs_root_leaf_owner_guard_static_passed_cargo_pending"
    } else if slice == "Runtime 08 ecs_events_messages Cargo 验证窗口探测" {
        "cargo_recheck_timeout_no_result"
    } else if slice == "Runtime 12 action context routing" {
        "action_context_static_passed_cargo_pending"
    } else if slice == "Runtime 12 gamepad event-owner 漂移同步" {
        "input_boundary_static_passed_cargo_pending"
    } else if slice == "Runtime 12 gamepad bridge source guard event-owner sync" {
        "gamepad_bridge_source_guard_static_passed_cargo_timeout"
    } else if slice == "Runtime 12 action axis value bindings" {
        "action_axis_value_static_passed_cargo_deferred"
    } else if slice == "Runtime 12 gamepad axis transition edges" {
        "action_axis_transition_static_passed_cargo_deferred"
    } else if slice == "Runtime 12 consumed gamepad axis arbitration" {
        "action_axis_consumption_static_passed_cargo_deferred"
    } else if slice == "Runtime 12 input recording/replay" {
        "input_recording_replay_static_passed_cargo_deferred"
    } else if slice == "Runtime 12 action map config source" {
        "action_config_static_passed_cargo_deferred"
    } else if slice == "Runtime 12 action manager registration path" {
        "action_manager_registration_static_passed_cargo_deferred"
    } else if slice == "Runtime 12 cursor host requests" {
        "cursor_host_request_static_passed_cargo_deferred"
    } else if slice == "Runtime 12 input validation window recheck" {
        "cargo_recheck_timeout_static_guards_passed"
    } else if slice == "Runtime 12 Input stack current audit recheck" {
        "input_stack_current_audit_static_passed_cargo_pending"
    } else {
        "mirror_docs_static_passed_cargo_pending"
    }
}
