//! 输入动作、手柄、宿主请求与公共契约保持由运行时输入栈拥有。以结果断言检查当前接口或源码快照对应的边界。
#[test]
fn runtime_12_input_stack_public_surface_exports_remain_visible() {
    let input_mod = include_str!("../../../../input/mod.rs");
    let framework_input_mod = include_str!("../../../../core/framework/input/mod.rs");
    let prelude = include_str!("../../../../prelude.rs");
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
}
