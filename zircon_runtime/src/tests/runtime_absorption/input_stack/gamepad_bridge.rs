#[test]
fn runtime_12_gamepad_bridge_keeps_runtime_abi_path() {
    let runtime_12_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let input_doc = include_str!("../../../../../docs/zircon_runtime/input/input_state.md");
    let gamepad_contract = include_str!("../../../core/framework/input/gamepad.rs");
    let input_event = include_str!("../../../core/framework/input/input_event.rs");
    let gamepad_tests = include_str!("../../../input/tests/gamepad_bridge.rs");
    let app_events =
        include_str!("../../../../../zircon_app/src/entry/runtime_entry_app/gamepad/events.rs");
    let app_polling =
        include_str!("../../../../../zircon_app/src/entry/runtime_entry_app/gamepad/polling.rs");
    let session = include_str!("../../../dynamic_api/session.rs");
    let session_events = include_str!("../../../dynamic_api/session/events.rs");

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
