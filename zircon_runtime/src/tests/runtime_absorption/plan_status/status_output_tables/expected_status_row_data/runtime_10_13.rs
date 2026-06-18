use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 10 Dynamic API 镜像文档守卫",
        [
            "runtime_10_dynamic_runtime_api_mirror_docs_match_structure_audit_counts",
            "dynamic_runtime_api_boundary",
            "standalone rustc 5/5",
            "dynamic_api/app/UI gates pending",
        ],
    ),
    (
        "Runtime 10 Dynamic API 行为测试锚审计同步",
        [
            "behavior_test_anchor_count = 15",
            "missing_behavior_test_anchors = []",
            "standalone dynamic_api_session 5/5",
            "dynamic_api/app/UI gates pending",
        ],
    ),
    (
        "Runtime 10 dynamic_api_session 吸收守卫拆分",
        [
            "dynamic_api_session/{shared,headless_profiles,event_split,test_owner_split,ffi_panic_boundary,ui_contract,v2_contract,mirror_docs}.rs",
            "expected_source_file_count = 23",
            "cargo test -p zircon_runtime --lib dynamic_api_session",
            "5 passed / 4231 filtered out",
        ],
    ),
    (
        "Runtime 10 UI contract duplicate public types cleanup",
        [
            "runtime_10_ui_contract_types_have_single_definition_across_interface_and_runtime",
            "UiBindingCodec",
            "ui_contract_single_source_anchors = 7/7",
            "ui_contract_duplicate_public_types = 0",
        ],
    ),
    (
        "Runtime 10 UI v2 contract sync",
        [
            "runtime_10_ui_v2_contract_sync_matches_runtime_09_verdict_and_interface_owner",
            "ui_component_api_version_mismatch_is_rejected_with_parse_error",
            "ui_v2_contract_sync_anchors = 9/9",
            "UiComponentApiVersion",
        ],
    ),
    (
        "Runtime 10 Dynamic Session Event Split",
        [
            "session/events.rs",
            "runtime_10_dynamic_session_event_split_keeps_abi_owner_and_event_router",
            "expected_source_file_count = 21",
            "active compile lanes deferred",
        ],
    ),
    (
        "Runtime 10 Dynamic Session Test Owner Split",
        [
            "session/tests/{mod,helpers,vampire_gameplay,vampire_menu,vampire_hud,frame_diagnostics,runtime_errors}.rs",
            "runtime_10_dynamic_session_test_owner_split_keeps_focused_modules",
            "session/tests/frame_diagnostics.rs",
            "standalone `dynamic_api_session.rs` 5/5",
        ],
    ),
    (
        "Runtime 11 JobSystem 镜像文档守卫",
        [
            "runtime_11_job_system_mirror_docs_match_structure_audit_counts",
            "job_system_boundary",
            "standalone rustc 1/1",
            "tasks/ecs_schedule/worker_pool/rayon Cargo gates pending",
        ],
    ),
    (
        "Runtime 11 JobSystem 行为测试锚审计同步",
        [
            "behavior_test_anchor_count = 10",
            "missing_behavior_test_anchors = []",
            "scheduler_wait_all_waits_for_all_handles_and_records_sync_time",
            "tasks/ecs_schedule/worker_pool/rayon Cargo gates pending",
        ],
    ),
    (
        "Runtime 11 scheduler wait_all 同步点",
        [
            "JobScheduler::wait_all",
            "scheduler_wait_all_waits_for_all_handles_and_records_sync_time",
            "behavior_test_anchor_count = 10",
            "tasks/ecs_schedule/worker_pool/rayon Cargo gates pending",
        ],
    ),
    (
        "Runtime 11 graphics frustum rayon cutover",
        [
            "runtime_11_m2_1_graphics_frustum_rayon_cutover_static_passed_cargo_pending",
            "parallel_frustum.rs",
            "direct_rayon_paths = 2",
            "tasks/ecs_schedule/worker_pool/rayon Cargo gates pending",
        ],
    ),
    (
        "Runtime 12 Input stack 镜像文档守卫",
        [
            "runtime_12_input_stack_mirror_docs_match_structure_audit_counts",
            "input_stack_boundary",
            "standalone rustc 4/4",
            "Cargo input/action_map/gamepad/app gates pending",
        ],
    ),
    (
        "Runtime 12 gamepad event-owner 漂移同步",
        [
            "session/events.rs",
            "InputEvent::Gamepad*",
            "missing_gamepad_abi_anchors = []",
            "standalone `input_stack.rs` rustc 4/4 passed",
        ],
    ),
    (
        "Runtime 12 Input stack 行为测试锚审计同步",
        [
            "behavior_test_anchor_count = 12",
            "missing_behavior_test_anchors = []",
            "standalone input_stack 4/4",
            "input/action_map/gamepad/app Cargo gates pending",
        ],
    ),
    (
        "Runtime 12 action context routing",
        [
            "InputActionContext",
            "evaluate_with_active_contexts",
            "action_contexts_filter_gameplay_and_menu_maps_without_rebinding",
            "behavior_test_anchor_count = 7",
        ],
    ),
    (
        "Runtime 12 gamepad bridge source guard event-owner sync",
        [
            "gamepad_host_bridge_uses_runtime_gamepad_abi_constructors",
            "session/events.rs",
            "public_surface_anchor_count = 11",
            "605s timeout no result",
        ],
    ),
    (
        "Runtime 12 action axis value bindings",
        [
            "gamepad_axis_binding_reports_continuous_action_value",
            "InputActionState::value",
            "public_surface_anchors = 13/13",
            "behavior_test_anchor_count = 8",
        ],
    ),
    (
        "Runtime 12 action map config source",
        [
            "input_config_builds_action_evaluator_from_serialized_action_map",
            "InputConfig::action_evaluator",
            "public_surface_anchors = 14/14",
            "behavior_test_anchor_count = 9",
        ],
    ),
    (
        "Runtime 12 action manager registration path",
        [
            "input_action_manager_resolves_from_runtime_module_descriptor",
            "resolve_input_action_manager",
            "public_surface_anchors = 17/17",
            "behavior_test_anchor_count = 10",
        ],
    ),
    (
        "Runtime 12 gamepad axis transition edges",
        [
            "gamepad_axis_action_reports_deadzone_transition_edges",
            "GamepadAxisTransition",
            "public_surface_anchors = 19/19",
            "behavior_test_anchor_count = 12",
        ],
    ),
    (
        "Runtime 12 consumed gamepad axis arbitration",
        [
            "consumed_gamepad_axis_does_not_activate_gameplay_action",
            "GamepadAxisInput",
            "evaluate_with_consumed_input",
            "public_surface_anchors = 19/19",
        ],
    ),
    (
        "Runtime 12 input recording/replay",
        [
            "InputRecording",
            "InputReplayCursor",
            "input_recording_captures_drainable_event_records_by_frame",
            "180s timeout no result",
        ],
    ),
    (
        "Runtime 13 Script binding 镜像文档守卫",
        [
            "runtime_13_script_binding_mirror_docs_match_structure_audit_counts",
            "expected_source_file_count = 18",
            "standalone rustc 2/2",
            "script Cargo filters pending",
        ],
    ),
    (
        "Runtime 13 Gameplay Host Owner Split",
        [
            "gameplay_host/{combat,components,input,lifecycle,navigation,script_bindings,transform,values}.rs",
            "runtime_13_gameplay_host_owner_split_keeps_domain_files",
            "script::vm -- --nocapture` 48/48 passed",
            "script Cargo filters pending",
        ],
    ),
    (
        "Runtime 13 Gameplay host predicate functions for real ZR VM",
        [
            "gameplay.entity_exists",
            "gameplay.script_number_at_most",
            "gameplay_host_script_property_match_and_heal_update_bindings",
            "host_function_registry_matches_documented_ledger",
        ],
    ),
];
