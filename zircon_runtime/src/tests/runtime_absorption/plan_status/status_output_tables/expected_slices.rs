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
    } else if slice == "Runtime 05 plan-status Cargo attempt 状态审计" {
        "cargo_attempt_status_static_passed_cargo_pending"
    } else if slice == "Runtime 05 plan-status Cargo timeout 状态审计" {
        "cargo_attempt_timeout_status_static_passed_cargo_pending"
    } else if slice == "Runtime 05 full scene closeout failed evidence" {
        "cargo_recheck_failed_full_scene_gate"
    } else if slice == "Runtime 05 full scene closeout no-result recheck" {
        "cargo_recheck_no_result_external_editor_lane"
    } else if slice == "Runtime 05 scene:: failure support-first triage" {
        "support_first_triage_static_passed_cargo_pending"
    } else if slice == "Runtime 05 scene:: lower-layer diagnostic matrix" {
        "support_first_matrix_static_passed_cargo_pending"
    } else if slice == "Runtime 05 serialization source folder-split guard sync" {
        "source_guard_static_passed_cargo_deferred_active_lanes"
    } else if slice == "Runtime 03 world bootstrap fixed-loop stage guard sync" {
        "guard_sync_static_passed_cargo_pending"
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
    } else if slice == "Runtime 07 artifact cache payload owner split" {
        "artifact_cache_payload_owner_split_static_passed_cargo_deferred"
    } else if slice == "Runtime 07 render product diagnostics owner split" {
        "render_product_diagnostics_owner_split_static_passed_cargo_deferred"
    } else if slice == "Runtime 07 animation scene frame diagnostics" {
        "animation_scene_frame_diagnostics_static_passed_cargo_deferred"
    } else if slice == "Runtime 07 QueryState cache owner performance audit sync" {
        "query_state_cache_owner_perf_audit_sync_static_passed_cargo_pending"
    } else if slice == "Runtime 07 virtual geometry debug snapshot owner split" {
        "virtual_geometry_debug_snapshot_owner_split_static_passed_cargo_deferred"
    } else if slice == "Runtime 07 owner-budget current doc mirror fix" {
        "mirror_docs_static_passed_cargo_pending"
    } else if slice == "Runtime 07 owner-budget 36-hotspot navigation split sync" {
        "mirror_docs_static_passed_cargo_pending"
    } else if slice == "Runtime 13 Gameplay Host Owner Split" {
        "folder_split_static_passed_script_vm_cargo_broader_gate_pending"
    } else if slice == "Runtime 02 generated template count 审计同步" {
        "structure_audit_static_passed_cargo_pending"
    } else if slice == "Runtime 02 root graphics alias block removal" {
        "graphics_alias_block_removed_static_passed_cargo_pending"
    } else if slice == "Runtime 02 rhi_wgpu root backend private cutover" {
        "rhi_wgpu_root_backend_private_static_passed_cargo_pending"
    } else if slice == "Runtime 02 builtin root facade cutover" {
        "builtin_root_facade_removed_static_passed_cargo_pending"
    } else if slice == "Runtime 11 graphics frustum rayon cutover" {
        "runtime_11_m2_1_graphics_frustum_rayon_cutover_static_passed_cargo_pending"
    } else if slice == "Runtime 11 scheduler wait_all 同步点" {
        "wait_all_static_passed_cargo_pending"
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
    } else if slice == "Runtime 08 QueryState cache owner split" {
        "query_state_cache_owner_split_static_passed_cargo_pending"
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
    } else {
        "mirror_docs_static_passed_cargo_pending"
    }
}

pub(super) fn expected_date_for_slice(slice: &str) -> &'static str {
    if matches!(
        slice,
        "Runtime 09 UI architecture 镜像文档守卫"
            | "Runtime 09 surface default interaction fallback rename"
            | "Runtime 07 ECS frame diagnostics aggregation"
            | "Runtime 07 QueryState frame auto-collection"
            | "Runtime 07 ChangeDetection frame auto-collection"
            | "Runtime 07 QueryState iterator lifetime guard"
            | "Runtime 07 FPS gate support unblock"
            | "Runtime 07 profiling build tooling"
            | "Runtime 07 extract rebuild cache"
            | "Runtime 07 extract cache hit/miss diagnostics"
            | "Runtime 07 asset worker frame sampler"
            | "Runtime 04 worker-pool manager frame sampler entry"
            | "Runtime 07 asset worker manager sampler entry"
            | "Runtime 07 artifact cache payload owner split"
            | "Runtime 07 render product diagnostics owner split"
            | "Runtime 07 animation scene frame diagnostics"
            | "Runtime 07 QueryState cache owner performance audit sync"
            | "Runtime 07 virtual geometry debug snapshot owner split"
            | "Runtime 07 owner-budget 42-hotspot 漂移同步"
            | "Runtime 07 owner-budget current doc mirror fix"
            | "Runtime 08 QueryState cache owner split"
            | "Runtime 10 UI contract duplicate public types cleanup"
            | "Runtime 10 UI v2 contract sync"
            | "Runtime 12 action context routing"
            | "Runtime 12 gamepad bridge source guard event-owner sync"
            | "Runtime 12 action axis value bindings"
            | "Runtime 12 gamepad axis transition edges"
            | "Runtime 12 consumed gamepad axis arbitration"
            | "Runtime 12 input recording/replay"
            | "Runtime 12 action map config source"
            | "Runtime 12 action manager registration path"
            | "Runtime 11 scheduler wait_all 同步点"
            | "Runtime 02 root graphics alias block removal"
            | "Runtime 02 rhi_wgpu root backend private cutover"
            | "Runtime 02 builtin root facade cutover"
            | "Runtime 05 texture importer DDS caps policy wording"
    ) {
        "2026-06-17"
    } else if matches!(
        slice,
        "Runtime 02 root_entries guard-count current resync"
            | "Runtime 05 Runtime 02 root_entries count 状态表闭环"
            | "Runtime 06 native root re-export current mirror fix"
            | "Runtime 06 plugin::native hard-cutover"
            | "Runtime 06 fallback lifecycle failure tests"
            | "Runtime 06 fallback lifecycle Cargo 验证"
            | "Runtime 06 shader artifact cache real-backend unblock"
            | "Runtime 06 Vampire real-backend menu/retry focused validation"
            | "Runtime 06 Vampire HUD real-backend capture validation"
            | "Runtime 06 native loader test namespace migration"
            | "Runtime 06 V1/V2 ABI hard-cutover"
            | "Runtime 06 hot reload failure injection"
            | "Runtime 09 UI input route authority"
            | "Runtime 09 navigation legacy reply rename"
            | "Runtime 09 pointer legacy reply rename"
            | "Runtime 09 pointer capture fallback rename"
            | "Runtime 09 table row label fallback rename"
            | "Runtime 09 template component-name fallback rename"
            | "Runtime 09 property visibility flag rename"
            | "Runtime 09 responsive MUI visibility flag rename"
            | "Runtime 09 accessibility open-state fallback rename"
            | "Runtime 09 layout engine backend name cutover"
            | "Runtime 09 taffy bridge pass order"
            | "Runtime 09 virtualization scroll boundary"
            | "Runtime 09 template pipeline boundary"
            | "Runtime 11 graphics frustum rayon cutover"
            | "Runtime 13 Gameplay host predicate functions for real ZR VM"
            | "Runtime 05 status-output current anchor fix"
    ) {
        "2026-06-16"
    } else if matches!(
        slice,
        "Runtime 07 owner-budget 38-hotspot 回漂同步"
            | "Runtime 07 owner-budget 39-hotspot 漂移同步"
            | "Runtime 07 owner-budget 37-hotspot 漂移同步"
            | "Runtime 07 owner-budget 37-hotspot 再同步"
            | "Runtime 05 status-output Runtime 07 owner-budget row"
    ) {
        "2026-06-15"
    } else if slice == "Runtime 14 animation runtime-status focused recheck timeout" {
        "2026-06-15"
    } else if slice == "Runtime 14 animation family 28-file audit sync"
        || slice == "Runtime 14 navigation fallback runtime owner split"
        || slice == "Runtime 07 owner-budget 36-hotspot navigation split sync"
    {
        "2026-06-17"
    } else if slice == "Runtime 14 Module family guard anchors 审计同步" {
        "2026-06-15"
    } else if slice == "Runtime 05 plan-status Cargo timeout 状态审计" {
        "2026-06-15"
    } else if slice == "Runtime 12 gamepad event-owner 漂移同步"
        || slice == "Runtime 01 Tech-stack 行为测试锚审计同步"
        || slice == "Runtime 02 core/root/generated 镜像文档守卫"
        || slice == "Runtime 02 guard-test anchors 审计同步"
        || slice == "Runtime 10 Dynamic API 行为测试锚审计同步"
        || slice == "Runtime 10 dynamic_api_session 吸收守卫拆分"
        || slice == "Runtime 12 Input stack 行为测试锚审计同步"
        || slice == "Runtime 04 Asset pipeline 行为测试锚审计同步"
        || slice == "Runtime 08 ECS 行为测试锚审计同步"
        || slice == "Runtime 05 status-output Runtime 08 behavior-test row"
        || slice == "Runtime 05 status-output Runtime 12 gamepad event-owner row"
        || slice == "Runtime 05 status-output Runtime 12 behavior-test row"
        || slice == "Runtime 05 status-output Runtime 04 behavior-test row"
        || slice == "Runtime 05 status-output Runtime 03 module-doc row"
        || slice == "Runtime 05 status-output Runtime 03 behavior-test row"
        || slice == "Runtime 05 status-output Runtime 10 behavior-test row"
        || slice == "Runtime 05 plan-status owner-budget current mirror fix"
        || slice == "Runtime 05 plan-status output-anchor module split"
        || slice == "Runtime 05 plan-status output-anchor budget guard"
        || slice == "Runtime 05 status-output status/date helper split"
        || slice == "Runtime 05 status-output expected anchor split"
        || slice == "Runtime 05 status-output row-data group split"
        || slice == "Runtime 05 plan-status root module split"
        || slice == "Runtime 05 plan-status support inventory split"
        || slice == "Runtime 05 plan-status anchor inventory split"
        || slice == "Runtime 05 plan-status markdown renderer split"
        || slice == "Runtime 05 plan-status source helper split"
        || slice == "Runtime 05 status-output expected row data split"
        || slice == "Runtime 05 full scene closeout failed evidence"
        || slice == "Runtime 05 full scene closeout no-result recheck"
        || slice == "Runtime 05 scene:: failure support-first triage"
        || slice == "Runtime 05 scene:: lower-layer diagnostic matrix"
        || slice == "Runtime 05 serialization source folder-split guard sync"
        || slice == "Runtime 03 world bootstrap fixed-loop stage guard sync"
        || slice == "Runtime 05 cargo-gates early Runtime 03 split"
        || slice == "Runtime 05 cargo-gates early Runtime 01 split"
        || slice == "Runtime 05 cargo-gates early Runtime 02 split"
        || slice == "Runtime 05 cargo-gates early Runtime 04 split"
        || slice == "Runtime 05 cargo-gates early Runtime 06 split"
        || slice == "Runtime 05 cargo-gates early Runtime 08 split"
        || slice == "Runtime 05 cargo-gates early Runtime 07 split"
        || slice == "Runtime 05 cargo-gates late Runtime 10 split"
        || slice == "Runtime 05 cargo-gates late Runtime 11 split"
        || slice == "Runtime 05 cargo-gates late Runtime 12 split"
        || slice == "Runtime 05 cargo-gates late Runtime 13 split"
        || slice == "Runtime 05 cargo-gates late Runtime 14 split"
        || slice == "Runtime 05 status-output all-index-row coverage guard"
        || slice == "Runtime 03 Schedule/frame-loop 行为测试锚审计同步"
        || slice == "Runtime 08 First-stage event update guard"
    {
        "2026-06-15"
    } else if slice == "Runtime 11 JobSystem 行为测试锚审计同步" {
        "2026-06-17"
    } else {
        "2026-06-14"
    }
}
