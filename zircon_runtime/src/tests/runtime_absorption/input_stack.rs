use std::fs;
use std::path::Path;

const INPUT_PRODUCTION_MODULE_MAX_LINES: usize = 500;
const INPUT_TEST_MODULE_MAX_LINES: usize = 800;
const EXPECTED_INPUT_RUNTIME_MODULES: &[&str] = &[
    "mod.rs",
    "module/config.rs",
    "module/descriptor.rs",
    "module/mod.rs",
    "module/module_type.rs",
    "runtime/action_evaluator.rs",
    "runtime/default_input_action_manager.rs",
    "runtime/default_input_manager.rs",
    "runtime/input_driver.rs",
    "runtime/recording.rs",
    "runtime/input_state.rs",
    "runtime/mod.rs",
];
const EXPECTED_FRAMEWORK_INPUT_MODULES: &[&str] = &[
    "button_input_state.rs",
    "cursor.rs",
    "file_drag_drop.rs",
    "gamepad.rs",
    "ime.rs",
    "input_action.rs",
    "input_action_context.rs",
    "input_action_manager.rs",
    "input_action_map.rs",
    "input_action_state.rs",
    "input_binding.rs",
    "input_button.rs",
    "input_event.rs",
    "input_event_record.rs",
    "input_frame_snapshot.rs",
    "input_snapshot.rs",
    "mod.rs",
    "mouse_wheel.rs",
    "touch.rs",
    "window_status.rs",
];
const EXPECTED_INPUT_TEST_MODULES: &[&str] = &[
    "action_axis_transitions.rs",
    "action_mapping.rs",
    "boundary.rs",
    "gamepad_bridge.rs",
    "input_manager.rs",
    "mod.rs",
    "recording_replay.rs",
];
const EXPECTED_RUNTIME_12_BEHAVIOR_TEST_ANCHORS: &[&str] = &[
    "input_snapshot_just_pressed_is_true_for_exactly_one_frame",
    "frame_input_clears_after_level_tick_not_before",
    "action_map_resolves_chords_and_reports_just_activated",
    "rebinding_action_does_not_require_recompilation",
    "action_contexts_filter_gameplay_and_menu_maps_without_rebinding",
    "gamepad_axis_binding_reports_continuous_action_value",
    "consumed_gamepad_axis_does_not_activate_gameplay_action",
    "gamepad_axis_action_reports_deadzone_transition_edges",
    "input_config_builds_action_evaluator_from_serialized_action_map",
    "input_action_manager_resolves_from_runtime_module_descriptor",
    "gamepad_disconnect_clears_held_state_without_panic",
    "gamepad_host_bridge_uses_runtime_gamepad_abi_constructors",
    "input_recording_captures_drainable_event_records_by_frame",
    "input_replay_restores_frame_snapshots_in_recorded_order",
    "cursor_host_requests_are_frame_local_and_drainable",
];

#[test]
fn runtime_12_input_stack_mirror_docs_match_structure_audit_counts() {
    assert_eq!(EXPECTED_INPUT_RUNTIME_MODULES.len(), 12);
    assert_eq!(EXPECTED_FRAMEWORK_INPUT_MODULES.len(), 20);
    assert_eq!(EXPECTED_INPUT_TEST_MODULES.len(), 7);
    assert_eq!(EXPECTED_RUNTIME_12_BEHAVIOR_TEST_ANCHORS.len(), 15);

    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_owner_files(
        &runtime_root.join("src").join("input"),
        EXPECTED_INPUT_RUNTIME_MODULES,
        INPUT_PRODUCTION_MODULE_MAX_LINES,
        "Runtime 12 input runtime owner",
    );
    assert_owner_files(
        &runtime_root
            .join("src")
            .join("core")
            .join("framework")
            .join("input"),
        EXPECTED_FRAMEWORK_INPUT_MODULES,
        INPUT_PRODUCTION_MODULE_MAX_LINES,
        "Runtime 12 framework input contract",
    );
    assert_owner_files(
        &runtime_root.join("src").join("input").join("tests"),
        EXPECTED_INPUT_TEST_MODULES,
        INPUT_TEST_MODULE_MAX_LINES,
        "Runtime 12 input test owner",
    );

    let input_mod = include_str!("../../input/mod.rs");
    let framework_input_mod = include_str!("../../core/framework/input/mod.rs");
    let prelude = include_str!("../../prelude.rs");
    for public_anchor in [
        "CursorGrabMode",
        "CursorHostRequest",
        "CursorPosition",
        "DefaultInputManager",
        "DefaultInputActionManager",
        "InputActionEvaluator",
        "InputActionManager",
        "InputAction",
        "InputActionContext",
        "InputBinding",
        "InputAxisBinding",
        "InputAxisDirection",
        "InputActionMap",
        "InputActionState",
        "InputConfig",
        "InputFrameSnapshot",
        "InputSnapshot",
        "GamepadConnectionInfo",
        "GamepadAxisInput",
        "GamepadAxisTransition",
        "GamepadRumbleRequest",
        "InputRecording",
        "InputRecordingFrame",
        "InputReplayCursor",
        "InputReplayFrameReport",
        "INPUT_ACTION_MANAGER_NAME",
    ] {
        assert!(
            input_mod.contains(public_anchor)
                || framework_input_mod.contains(public_anchor)
                || prelude.contains(public_anchor),
            "Runtime 12 public input surface should retain `{public_anchor}`"
        );
    }

    let input_stack_guard = include_str!("input_stack.rs");
    let cargo_gate_guard = include_str!("plan_status/cargo_gates/late.rs");
    for guard_anchor in [
        "runtime_12_input_stack_contracts_stay_documented_and_exported",
        "runtime_12_action_mapping_keeps_ui_filtered_evaluation_path",
        "runtime_12_gamepad_bridge_keeps_runtime_abi_path",
        "runtime_12_input_stack_mirror_docs_match_structure_audit_counts",
        "runtime_12_input_stack_cargo_pending_gate_stays_explicit_until_input_validation",
    ] {
        assert!(
            input_stack_guard.contains(guard_anchor) || cargo_gate_guard.contains(guard_anchor),
            "Runtime 12 guard anchor `{guard_anchor}` should stay visible to input_stack_boundary"
        );
    }

    let behavior_test_sources = [
        include_str!("../../input/tests/input_manager.rs"),
        include_str!("../../input/tests/action_mapping.rs"),
        include_str!("../../input/tests/action_axis_transitions.rs"),
        include_str!("../../input/tests/gamepad_bridge.rs"),
        include_str!("../../input/tests/recording_replay.rs"),
    ];
    for behavior_anchor in EXPECTED_RUNTIME_12_BEHAVIOR_TEST_ANCHORS {
        assert!(
            behavior_test_sources
                .iter()
                .any(|source| source.contains(behavior_anchor)),
            "Runtime 12 behavior test anchor `{behavior_anchor}` should stay visible to input_stack_boundary"
        );
    }

    let cursor_host_request_sources = [
        include_str!("../../core/framework/input/mod.rs"),
        include_str!("../../core/framework/input/input_event.rs"),
        include_str!("../../input/runtime/default_input_manager.rs"),
        include_str!("../../dynamic_api/session.rs"),
        include_str!("../../dynamic_api/session/host_requests.rs"),
        include_str!("../../../../zircon_runtime_interface/src/runtime_api/host_requests.rs"),
        include_str!("../../../../zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs"),
        include_str!(
            "../../../../zircon_app/src/entry/runtime_entry_app/host_requests/cursor/request.rs"
        ),
        include_str!("../../platform/tests/backend_tokens.rs"),
        include_str!("../../platform/tests/diagnostics.rs"),
    ];
    for required_cursor_anchor in [
        "CursorHostRequest",
        "InputEvent::CursorHostRequest",
        "drain_cursor_host_requests",
        "runtime_cursor_host_request",
        "ZrRuntimeCursorHostRequestV1",
        "ZrRuntimeHostRequestV1::Cursor",
        "apply_runtime_cursor_host_request",
        "set_cursor_visible",
        "set_cursor_grab",
        "set_cursor_hittest",
        "set_cursor_position",
        "platform.cursor_options=supported:winit_window_options",
    ] {
        assert!(
            cursor_host_request_sources
                .iter()
                .any(|source| source.contains(required_cursor_anchor)),
            "Runtime 12 cursor host-request source path should retain `{required_cursor_anchor}`"
        );
    }

    let mirror_docs = [
        (
            "Runtime input module doc",
            include_str!("../../../../docs/zircon_runtime/input/input_state.md"),
        ),
        (
            "Runtime 12 plan",
            include_str!(
                "../../../../docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md"
            ),
        ),
        (
            "runtime index",
            include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "M0 review",
            include_str!("../../../../docs/engine-architecture/runtime-architecture-review-m0.md"),
        ),
        (
            "interface convergence",
            include_str!("../../../../docs/engine-architecture/runtime-interface-convergence.md"),
        ),
    ];

    for (doc_name, doc_source) in mirror_docs {
        for required_anchor in [
            "input_stack_boundary",
            "expected_runtime_module_count = 12",
            "expected_framework_module_count = 20",
            "expected_test_module_count = 7",
            "public_surface_anchors = 26/26",
            "runtime_12_guard_anchors = 5/5",
            "missing_gamepad_abi_anchors = []",
            "missing_cursor_host_request_anchors = []",
            "missing_doc_anchors = []",
            "missing_test_anchors = []",
            "behavior_test_anchor_count = 15",
            "missing_behavior_test_anchors = []",
            "missing_cargo_gate_anchors = []",
            "oversized_modules = []",
            "mirror_docs_guard_present = true",
            "risks = []",
            "runtime_12_input_stack_mirror_docs_match_structure_audit_counts",
        ] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should mirror Runtime 12 input-stack audit anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_12_input_stack_contracts_stay_documented_and_exported() {
    let runtime_12_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let input_doc = include_str!("../../../../docs/zircon_runtime/input/input_state.md");
    let input_mod = include_str!("../../input/mod.rs");
    let framework_input_mod = include_str!("../../core/framework/input/mod.rs");
    let prelude = include_str!("../../prelude.rs");
    let input_tests = include_str!("../../input/tests/input_manager.rs");

    for required_anchor in [
        "Frame Input Contract",
        "DefaultInputManager::begin_frame()",
        "RuntimeDynamicSession::tick_frame() runs the loaded level before it calls `input_manager.begin_frame()`",
        "input_snapshot_just_pressed_is_true_for_exactly_one_frame",
        "frame_input_clears_after_level_tick_not_before",
        "input_frame_contract_static_passed_cargo_pending",
    ] {
        assert!(
            input_doc.contains(required_anchor)
                || runtime_12_plan.contains(required_anchor)
                || runtime_index.contains(required_anchor)
                || input_tests.contains(required_anchor),
            "Runtime 12 frame-input contract should retain `{required_anchor}`"
        );
    }

    for required_export in [
        "CursorGrabMode",
        "CursorHostRequest",
        "CursorPosition",
        "InputAction",
        "InputActionContext",
        "InputBinding",
        "InputAxisBinding",
        "InputAxisDirection",
        "InputActionMap",
        "InputActionState",
        "InputConfig",
        "InputFrameSnapshot",
        "InputSnapshot",
        "InputActionEvaluator",
        "InputActionManager",
        "GamepadAxisInput",
        "DefaultInputActionManager",
        "DefaultInputManager",
        "InputRecording",
        "InputRecordingFrame",
        "InputReplayCursor",
        "InputReplayFrameReport",
        "INPUT_ACTION_MANAGER_NAME",
    ] {
        assert!(
            input_mod.contains(required_export)
                || framework_input_mod.contains(required_export)
                || prelude.contains(required_export),
            "Runtime input public surface should keep `{required_export}` exported"
        );
    }
}

fn assert_owner_files(
    owner_root: &Path,
    expected_modules: &[&str],
    max_lines: usize,
    owner_label: &str,
) {
    for module in expected_modules {
        let path = owner_root.join(module);
        assert!(
            path.exists(),
            "{owner_label} module `{module}` is missing; update input_stack_boundary before changing the input owner set"
        );
        let line_count = line_count(&path);
        assert!(
            line_count <= max_lines,
            "{owner_label} module `{module}` has {line_count} lines, exceeding the {max_lines}-line owner budget"
        );
    }
}

fn line_count(path: &Path) -> usize {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
        .lines()
        .count()
}

#[test]
fn runtime_12_action_mapping_keeps_ui_filtered_evaluation_path() {
    let runtime_12_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let input_doc = include_str!("../../../../docs/zircon_runtime/input/input_state.md");
    let action = include_str!("../../core/framework/input/input_action.rs");
    let action_context = include_str!("../../core/framework/input/input_action_context.rs");
    let binding = include_str!("../../core/framework/input/input_binding.rs");
    let action_map = include_str!("../../core/framework/input/input_action_map.rs");
    let action_manager = include_str!("../../core/framework/input/input_action_manager.rs");
    let action_state = include_str!("../../core/framework/input/input_action_state.rs");
    let config = include_str!("../../input/module/config.rs");
    let descriptor = include_str!("../../input/module/descriptor.rs");
    let evaluator = include_str!("../../input/runtime/action_evaluator.rs");
    let default_action_manager =
        include_str!("../../input/runtime/default_input_action_manager.rs");
    let core_manager = include_str!("../../core/manager/mod.rs");
    let core_manager_resolver = include_str!("../../core/manager/resolver.rs");
    let core_manager_service_names = include_str!("../../core/manager/service_names.rs");
    let action_tests = include_str!("../../input/tests/action_mapping.rs");
    let action_axis_transition_tests = include_str!("../../input/tests/action_axis_transitions.rs");

    for required_contract_anchor in [
        "pub struct InputAction",
        "pub struct InputActionContext",
        "pub trait InputActionManager",
        "pub struct InputBinding",
        "pub struct InputAxisBinding",
        "pub enum InputAxisDirection",
        "pub struct InputActionMap",
        "pub struct InputActionState",
        "pub struct InputConfig",
        "Serialize, Deserialize",
    ] {
        assert!(
            action.contains(required_contract_anchor)
                || action_context.contains(required_contract_anchor)
                || action_manager.contains(required_contract_anchor)
                || binding.contains(required_contract_anchor)
                || action_map.contains(required_contract_anchor)
                || action_state.contains(required_contract_anchor)
                || config.contains(required_contract_anchor),
            "Runtime 12 action contract should retain `{required_contract_anchor}`"
        );
    }

    for required_evaluator_anchor in [
        "pub struct InputActionEvaluator",
        "DefaultInputActionManager",
        "set_action_map",
        "evaluate_with_consumed_buttons",
        "evaluate_with_consumed_input",
        "evaluate_with_active_contexts",
        "evaluate_with_active_contexts_and_consumed_buttons",
        "evaluate_with_active_contexts_and_consumed_input",
        "consumed_buttons.contains(button)",
        "consumed_axes.contains",
        "action_context_is_active",
        "InputActionState::from_sets_and_values",
        "binding_axis_consumed",
        "binding_axis_value",
        "binding_axis_transition",
        "dominant_action_value",
    ] {
        assert!(
            evaluator.contains(required_evaluator_anchor)
                || default_action_manager.contains(required_evaluator_anchor),
            "Runtime 12 action evaluator should retain `{required_evaluator_anchor}`"
        );
    }

    for required_registration_anchor in [
        "module_descriptor_with_config",
        "INPUT_ACTION_MANAGER_NAME",
        "InputActionManagerHandle",
        "resolve_input_action_manager",
        "InputModule.Manager.InputActionManager",
    ] {
        assert!(
            descriptor.contains(required_registration_anchor)
                || core_manager.contains(required_registration_anchor)
                || core_manager_resolver.contains(required_registration_anchor)
                || core_manager_service_names.contains(required_registration_anchor),
            "Runtime 12 action manager registration should retain `{required_registration_anchor}`"
        );
    }

    for required_test_anchor in [
        "action_map_resolves_chords_and_reports_just_activated",
        "rebinding_action_does_not_require_recompilation",
        "action_contexts_filter_gameplay_and_menu_maps_without_rebinding",
        "gamepad_axis_binding_reports_continuous_action_value",
        "consumed_gamepad_axis_does_not_activate_gameplay_action",
        "gamepad_axis_action_reports_deadzone_transition_edges",
        "input_config_builds_action_evaluator_from_serialized_action_map",
        "input_action_manager_resolves_from_runtime_module_descriptor",
        "evaluate_with_consumed_buttons",
        "evaluate_with_consumed_input",
        "evaluate_with_active_contexts",
        "resolve_input_action_manager",
        "value(\"gameplay.move_x\")",
        "action_evaluator()",
        "action_manager",
        "clear_bindings",
    ] {
        assert!(
            action_tests.contains(required_test_anchor)
                || action_axis_transition_tests.contains(required_test_anchor)
                || config.contains(required_test_anchor),
            "Runtime 12 action mapping tests/config should retain `{required_test_anchor}`"
        );
    }

    for required_plan_anchor in [
        "UI surface/pointer capture/popup/focus 优先",
        "玩法/action mapping 只消费 UI 未处理",
        "action_contract_static_passed_cargo_pending",
        "action_evaluator_static_passed_cargo_pending",
        "action_context_static_passed_cargo_pending",
        "action_axis_value_static_passed_cargo_deferred",
        "action_config_static_passed_cargo_deferred",
        "action_manager_registration_static_passed_cargo_deferred",
    ] {
        assert!(
            runtime_12_plan.contains(required_plan_anchor)
                || runtime_index.contains(required_plan_anchor)
                || input_doc.contains(required_plan_anchor),
            "Runtime 12 docs/index should retain action arbitration anchor `{required_plan_anchor}`"
        );
    }
}

#[test]
fn runtime_12_gamepad_bridge_keeps_runtime_abi_path() {
    let runtime_12_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let input_doc = include_str!("../../../../docs/zircon_runtime/input/input_state.md");
    let gamepad_contract = include_str!("../../core/framework/input/gamepad.rs");
    let input_event = include_str!("../../core/framework/input/input_event.rs");
    let gamepad_tests = include_str!("../../input/tests/gamepad_bridge.rs");
    let app_events =
        include_str!("../../../../zircon_app/src/entry/runtime_entry_app/gamepad/events.rs");
    let app_polling =
        include_str!("../../../../zircon_app/src/entry/runtime_entry_app/gamepad/polling.rs");
    let session = include_str!("../../dynamic_api/session.rs");
    let session_events = include_str!("../../dynamic_api/session/events.rs");

    for required_contract_anchor in [
        "GamepadConnectionInfo",
        "GamepadButton",
        "GamepadAxis",
        "GAMEPAD_BUTTON_PRESS_THRESHOLD",
        "GAMEPAD_AXIS_DEADZONE_LOWER",
        "GamepadRumbleRequest",
    ] {
        assert!(
            gamepad_contract.contains(required_contract_anchor),
            "Runtime gamepad contract should retain `{required_contract_anchor}`"
        );
    }

    for required_event_anchor in [
        "GamepadConnection(GamepadConnectionInfo)",
        "GamepadButton",
        "GamepadAxis",
        "GamepadRumbleRequest",
    ] {
        assert!(
            input_event.contains(required_event_anchor),
            "Runtime input event contract should retain `{required_event_anchor}`"
        );
    }

    for required_bridge_anchor in [
        "ZrRuntimeEventV1::gamepad_connection_with_ids",
        "ZrRuntimeEventV1::gamepad_button",
        "ZrRuntimeEventV1::gamepad_axis",
        "EventType::Connected",
        "EventType::Disconnected",
        "handle_gamepad_connection",
        "handle_gamepad_button",
        "handle_gamepad_axis",
        "InputEvent::GamepadConnection",
        "InputEvent::GamepadButton",
        "InputEvent::GamepadAxis",
    ] {
        assert!(
            app_events.contains(required_bridge_anchor)
                || app_polling.contains(required_bridge_anchor)
                || session.contains(required_bridge_anchor)
                || session_events.contains(required_bridge_anchor),
            "Runtime 12 gamepad bridge should retain `{required_bridge_anchor}`"
        );
    }

    for required_test_anchor in [
        "gamepad_disconnect_clears_held_state_without_panic",
        "gamepad_host_bridge_uses_runtime_gamepad_abi_constructors",
    ] {
        assert!(
            gamepad_tests.contains(required_test_anchor),
            "Runtime 12 gamepad tests should retain `{required_test_anchor}`"
        );
    }

    for required_doc_anchor in [
        "gamepad_bridge_static_passed_cargo_pending",
        "app 侧 gilrs",
        "InputEvent::GamepadConnection",
        "InputEvent::GamepadButton",
        "InputEvent::GamepadAxis",
    ] {
        assert!(
            runtime_12_plan.contains(required_doc_anchor)
                || runtime_index.contains(required_doc_anchor)
                || input_doc.contains(required_doc_anchor),
            "Runtime 12 docs/index should retain gamepad bridge anchor `{required_doc_anchor}`"
        );
    }
}
