use crate::core::framework::input::InputManager;

use crate::input::{
    DefaultInputManager, GamepadAxis, GamepadButton, GamepadConnectionInfo, GamepadId, InputButton,
    InputEvent,
};

#[test]
fn gamepad_disconnect_clears_held_state_without_panic() {
    let input = DefaultInputManager::default();
    let gamepad = GamepadId(42);
    let button = GamepadButton::South;
    let input_button = InputButton::Gamepad { gamepad, button };

    input.submit_event(InputEvent::GamepadConnection(GamepadConnectionInfo {
        gamepad,
        connected: true,
        name: Some("Runtime Pad".to_string()),
        vendor_id: Some(1),
        product_id: Some(2),
    }));
    input.submit_event(InputEvent::GamepadButton {
        gamepad,
        button,
        value: 1.0,
        pressed: true,
    });
    input.submit_event(InputEvent::GamepadAxis {
        gamepad,
        axis: GamepadAxis::LeftStickX,
        value: 0.75,
    });

    let connected = input.frame_snapshot();

    assert_eq!(connected.connected_gamepads, vec![gamepad]);
    assert!(connected.buttons.pressed(&input_button));
    assert_eq!(connected.gamepad_axes.len(), 1);
    assert_eq!(connected.gamepad_button_values.len(), 1);

    input.submit_event(InputEvent::GamepadConnection(GamepadConnectionInfo {
        gamepad,
        connected: false,
        name: None,
        vendor_id: None,
        product_id: None,
    }));

    let disconnected = input.frame_snapshot();

    assert!(disconnected.connected_gamepads.is_empty());
    assert!(!disconnected.buttons.pressed(&input_button));
    assert!(disconnected.buttons.just_released(&input_button));
    assert!(disconnected.gamepad_axes.is_empty());
    assert!(disconnected.gamepad_button_values.is_empty());

    input.begin_frame();
    let next_frame = input.frame_snapshot();

    assert!(!next_frame.buttons.just_released(&input_button));
    assert!(next_frame.connected_gamepads.is_empty());
}

#[test]
fn gamepad_host_bridge_uses_runtime_gamepad_abi_constructors() {
    let app_events =
        include_str!("../../../../zircon_app/src/entry/runtime_entry_app/gamepad/events.rs");
    let app_polling =
        include_str!("../../../../zircon_app/src/entry/runtime_entry_app/gamepad/polling.rs");
    let session = include_str!("../../dynamic_api/session.rs");
    let session_events = include_str!("../../dynamic_api/session/events.rs");

    assert!(app_events.contains("ZrRuntimeEventV1::gamepad_connection_with_ids"));
    assert!(app_events.contains("ZrRuntimeEventV1::gamepad_button"));
    assert!(app_events.contains("ZrRuntimeEventV1::gamepad_axis"));
    assert!(app_polling.contains("EventType::Connected"));
    assert!(app_polling.contains("EventType::Disconnected"));
    assert!(app_polling.contains("clear_gamepad_rumble_effects_for_gamepad"));
    assert!(
        session.contains("handle_gamepad_connection")
            || session_events.contains("handle_gamepad_connection")
    );
    assert!(
        session.contains("handle_gamepad_button")
            || session_events.contains("handle_gamepad_button")
    );
    assert!(
        session.contains("handle_gamepad_axis") || session_events.contains("handle_gamepad_axis")
    );
    assert!(
        session.contains("InputEvent::GamepadConnection")
            || session_events.contains("InputEvent::GamepadConnection")
    );
    assert!(
        session.contains("InputEvent::GamepadButton")
            || session_events.contains("InputEvent::GamepadButton")
    );
    assert!(
        session.contains("InputEvent::GamepadAxis")
            || session_events.contains("InputEvent::GamepadAxis")
    );
}
