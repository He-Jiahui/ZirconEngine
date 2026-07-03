pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 14 Cargo 验证窗口探测" {
        Some("cargo_deferred_active_lane")
    } else if slice == "Runtime 14 animation Cargo gate 尝试" {
        Some("cargo_blocked_external_compile_drift")
    } else if slice == "Runtime 14 animation Cargo gate 修复与复验阻塞" {
        Some("cargo_recheck_blocked_external_ui_compile_drift")
    } else if slice == "Runtime 14 animation runtime-status focused recheck timeout" {
        Some("cargo_recheck_timeout_no_result")
    } else if slice == "Runtime 14 animation family 28-file audit sync" {
        Some("module_family_source_count_static_passed_cargo_pending")
    } else if slice == "Runtime 14 navigation fallback runtime owner split" {
        Some("navigation_runtime_owner_split_static_passed_cargo_pending")
    } else if slice == "Runtime 14 module family current audit recheck" {
        Some("module_family_current_audit_static_passed_cargo_pending")
    } else if slice == "Runtime 14 module family 2026-07-01 current audit recheck" {
        Some("module_family_20260701_current_audit_static_passed_cargo_deferred")
    } else if slice == "Runtime 14 module family markdown renderer split" {
        Some("module_family_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 13 Gameplay Host Owner Split" {
        Some("folder_split_static_passed_script_vm_cargo_broader_gate_pending")
    } else if slice == "Runtime 13 Script binding current audit recheck" {
        Some("script_binding_current_audit_static_passed_cargo_pending")
    } else if slice == "Runtime 13 script binding Markdown renderer split" {
        Some("script_binding_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 13 Script binding 2026-07-01 current audit recheck" {
        Some("script_binding_20260701_current_audit_static_passed_cargo_deferred")
    } else if slice == "Runtime 11 graphics frustum rayon cutover" {
        Some("runtime_11_m2_1_graphics_frustum_rayon_cutover_static_passed_cargo_pending")
    } else if slice == "Runtime 11 scheduler wait_all 同步点" {
        Some("wait_all_static_passed_cargo_pending")
    } else if slice == "Runtime 11 panic-safe handle completion" {
        Some("panic_safe_completion_static_passed_cargo_deferred")
    } else if slice == "Runtime 11 JobSystem 2026-06-20 验证窗口探测" {
        Some("cargo_recheck_timeout_static_guards_passed")
    } else if slice == "Runtime 11 JobSystem core-min 验证窗口探测" {
        Some("core_min_cargo_recheck_timeout_static_guards_passed")
    } else if slice == "Runtime 11 JobSystem current audit recheck" {
        Some("job_system_current_audit_static_passed_cargo_pending")
    } else if slice == "Runtime 11 JobSystem 2026-07-01 current audit recheck" {
        Some("job_system_20260701_current_audit_static_passed_cargo_deferred")
    } else if slice == "Runtime 11 JobSystem inventory split" {
        Some("job_system_inventory_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 11 JobSystem Markdown renderer split" {
        Some("job_system_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 11 worker wait-assist" {
        Some("worker_wait_assist_static_passed_cargo_deferred")
    } else if slice == "Runtime 11 worker wait-assist core-min 验证窗口探测" {
        Some("worker_wait_assist_core_min_cargo_timeout_no_result_residual_stopped")
    } else if slice == "Runtime 11 worker wait-assist core-min test binary 验证" {
        Some("worker_wait_assist_core_min_test_binary_passed_cargo_gate_pending")
    } else if slice == "Runtime 11 core-min test binary task/guard batch" {
        Some("runtime_11_core_min_test_binary_task_guard_batch_passed_cargo_gate_pending")
    } else if slice == "Runtime 11 ecs_schedule source-guard lifetime anchor repair" {
        Some("runtime_11_ecs_schedule_lifetime_guard_anchor_static_passed_rebuild_pending")
    } else if slice == "Runtime 11 ecs_schedule core-min Cargo 复验" {
        Some("runtime_11_core_min_ecs_schedule_cargo_passed_remaining_gates_pending")
    } else if slice == "Runtime 11 tasks core-min Cargo 复验" {
        Some("runtime_11_core_min_tasks_cargo_passed_remaining_gates_pending")
    } else if slice == "Runtime 11 worker_pool core-min Cargo 复验" {
        Some("runtime_11_core_min_worker_pool_cargo_passed_remaining_gates_pending")
    } else if slice == "Runtime 11 rayon core-min Cargo 复验" {
        Some("runtime_11_core_min_rayon_cargo_passed_broader_gates_pending")
    } else if slice == "Runtime 11 tasks default Cargo 复验" {
        Some("runtime_11_default_tasks_cargo_passed_remaining_default_gates_pending")
    } else if slice == "Runtime 11 worker_pool default Cargo 复验" {
        Some("runtime_11_default_worker_pool_cargo_passed_remaining_default_gates_pending")
    } else if slice == "Runtime 11 rayon default Cargo 复验" {
        Some("runtime_11_default_rayon_cargo_passed_full_lib_gate_pending")
    } else if slice == "Runtime 11 ecs_schedule default Cargo 复验" {
        Some("runtime_11_default_ecs_schedule_cargo_passed_full_lib_gate_pending")
    } else if slice == "Runtime 11 full-lib default Cargo closeout attempt" {
        Some("runtime_11_full_lib_cargo_timeout_with_broader_failures_observed")
    } else if slice == "Runtime 11 core runtime full-lib triage recheck" {
        Some("runtime_11_core_runtime_tests_passed_full_lib_gate_broader_failures_pending")
    } else if slice == "Runtime 11 asset broader failure triage core-min 复验" {
        Some("runtime_11_asset_tests_passed_full_lib_gate_dynamic_graphics_pending")
    } else if slice == "Runtime 11 full-lib default after asset triage recheck" {
        Some("runtime_11_full_lib_after_asset_recheck_blocked_graphics_compile_timeout")
    } else if slice == "Runtime 11 full-lib default after graphics exposure retry" {
        Some("runtime_11_full_lib_after_graphics_exposure_retry_timeout_104_broader_failures")
    } else if slice == "Runtime 13 Gameplay host predicate functions for real ZR VM" {
        Some("focused_behavior_passed_broader_script_gate_pending")
    } else if slice == "Runtime 12 action context routing" {
        Some("action_context_static_passed_cargo_pending")
    } else if slice == "Runtime 12 gamepad event-owner 漂移同步" {
        Some("input_boundary_static_passed_cargo_pending")
    } else if slice == "Runtime 12 gamepad bridge source guard event-owner sync" {
        Some("gamepad_bridge_source_guard_static_passed_cargo_timeout")
    } else if slice == "Runtime 12 action axis value bindings" {
        Some("action_axis_value_static_passed_cargo_deferred")
    } else if slice == "Runtime 12 gamepad axis transition edges" {
        Some("action_axis_transition_static_passed_cargo_deferred")
    } else if slice == "Runtime 12 consumed gamepad axis arbitration" {
        Some("action_axis_consumption_static_passed_cargo_deferred")
    } else if slice == "Runtime 12 input recording/replay" {
        Some("input_recording_replay_static_passed_cargo_deferred")
    } else if slice == "Runtime 12 action map config source" {
        Some("action_config_static_passed_cargo_deferred")
    } else if slice == "Runtime 12 action manager registration path" {
        Some("action_manager_registration_static_passed_cargo_deferred")
    } else if slice == "Runtime 12 cursor host requests" {
        Some("cursor_host_request_static_passed_cargo_deferred")
    } else if slice == "Runtime 12 input validation window recheck" {
        Some("cargo_recheck_timeout_static_guards_passed")
    } else if slice == "Runtime 12 Input stack current audit recheck" {
        Some("input_stack_current_audit_static_passed_cargo_pending")
    } else if slice == "Runtime 12 Input stack 2026-07-01 current audit recheck" {
        Some("input_stack_20260701_current_audit_static_passed_cargo_deferred")
    } else if slice == "Runtime 12 Input stack inventory split" {
        Some("input_stack_inventory_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 12 Input stack Markdown renderer split" {
        Some("input_stack_markdown_split_static_passed_cargo_deferred_tests_deferred")
    } else if slice == "Runtime 12 input boundary grouped manager import guard repair" {
        Some("input_boundary_grouped_manager_import_guard_passed_broader_input_failures_pending")
    } else if slice == "Runtime 12 input_manager child test owner audit sync" {
        Some("runtime_12_input_manager_child_test_owner_audit_sync_static_passed_cargo_deferred")
    } else {
        None
    }
}
