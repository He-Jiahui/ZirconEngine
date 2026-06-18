use crate::core::framework::input::InputManager;

use crate::input::{
    DefaultInputManager, GamepadAxis, GamepadAxisSettings, GamepadConnectionInfo, GamepadId,
    InputAction, InputActionEvaluator, InputActionMap, InputAxisBinding, InputBinding, InputEvent,
};

#[test]
fn gamepad_axis_action_reports_deadzone_transition_edges() {
    let gamepad = GamepadId(12);
    let axis = GamepadAxis::LeftStickX;
    let mut map = InputActionMap::new();
    map.add_action(InputAction::new("gameplay.move_x"));
    map.bind(InputBinding::axis(
        "gameplay.move_x",
        InputAxisBinding::new(gamepad, axis),
    ));
    let evaluator = InputActionEvaluator::new(map);
    let input = DefaultInputManager::default();

    input.begin_frame();
    input.submit_event(InputEvent::GamepadAxis {
        gamepad,
        axis,
        value: 0.75,
    });
    let activated = evaluator.evaluate(&input.frame_snapshot());
    let expected_value = GamepadAxisSettings::default().scaled_value(0.75);

    assert!(activated.pressed("gameplay.move_x"));
    assert!(activated.just_activated("gameplay.move_x"));
    assert!(!activated.just_deactivated("gameplay.move_x"));
    assert_eq!(
        activated.just_activated_actions(),
        vec!["gameplay.move_x".to_string()]
    );
    assert_close(activated.value("gameplay.move_x"), expected_value);

    input.begin_frame();
    let held = evaluator.evaluate(&input.frame_snapshot());

    assert!(held.pressed("gameplay.move_x"));
    assert!(!held.just_activated("gameplay.move_x"));
    assert!(!held.just_deactivated("gameplay.move_x"));

    input.begin_frame();
    input.submit_event(InputEvent::GamepadAxis {
        gamepad,
        axis,
        value: 0.0,
    });
    let deactivated = evaluator.evaluate(&input.frame_snapshot());

    assert!(!deactivated.pressed("gameplay.move_x"));
    assert!(!deactivated.just_activated("gameplay.move_x"));
    assert!(deactivated.just_deactivated("gameplay.move_x"));
    assert_eq!(
        deactivated.just_deactivated_actions(),
        vec!["gameplay.move_x".to_string()]
    );
    assert_close(deactivated.value("gameplay.move_x"), 0.0);
}

#[test]
fn gamepad_disconnect_deactivates_axis_action() {
    let gamepad = GamepadId(14);
    let axis = GamepadAxis::LeftStickX;
    let mut map = InputActionMap::new();
    map.add_action(InputAction::new("gameplay.move_x"));
    map.bind(InputBinding::axis(
        "gameplay.move_x",
        InputAxisBinding::new(gamepad, axis),
    ));
    let evaluator = InputActionEvaluator::new(map);
    let input = DefaultInputManager::default();

    input.begin_frame();
    input.submit_event(InputEvent::GamepadAxis {
        gamepad,
        axis,
        value: 0.75,
    });
    let activated = evaluator.evaluate(&input.frame_snapshot());

    assert!(activated.pressed("gameplay.move_x"));
    assert!(activated.just_activated("gameplay.move_x"));

    input.begin_frame();
    input.submit_event(InputEvent::GamepadConnection(GamepadConnectionInfo {
        gamepad,
        connected: false,
        name: None,
        vendor_id: None,
        product_id: None,
    }));
    let deactivated = evaluator.evaluate(&input.frame_snapshot());

    assert!(!deactivated.pressed("gameplay.move_x"));
    assert!(deactivated.just_deactivated("gameplay.move_x"));
}

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.0001,
        "expected {actual} to be close to {expected}"
    );
}
