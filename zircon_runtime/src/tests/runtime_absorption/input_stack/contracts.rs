#[test]
fn runtime_12_input_stack_contracts_stay_documented_and_exported() {
    let runtime_12_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let input_doc = include_str!("../../../../../docs/zircon_runtime/input/input_state.md");
    let input_mod = include_str!("../../../input/mod.rs");
    let framework_input_mod = include_str!("../../../core/framework/input/mod.rs");
    let prelude = include_str!("../../../prelude.rs");
    let input_tests = [
        include_str!("../../../input/tests/input_manager.rs"),
        include_str!("../../../input/tests/input_manager/frame_state.rs"),
    ]
    .join("\n");

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
