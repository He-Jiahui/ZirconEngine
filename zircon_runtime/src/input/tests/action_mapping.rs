use crate::core::framework::input::InputManager;

use crate::input::{
    DefaultInputManager, InputAction, InputActionEvaluator, InputActionMap, InputBinding,
    InputButton, InputEvent,
};

#[test]
fn action_map_resolves_chords_and_reports_just_activated() {
    let shift = InputButton::Key("Shift".to_string());
    let forward = InputButton::Key("KeyW".to_string());
    let mut map = InputActionMap::new();
    map.add_action(InputAction::new("gameplay.dash").with_context("gameplay"));
    map.bind(InputBinding::chord(
        "gameplay.dash",
        [shift.clone(), forward.clone()],
    ));
    let evaluator = InputActionEvaluator::new(map);
    let input = DefaultInputManager::default();

    input.begin_frame();
    input.submit_event(InputEvent::ButtonPressed(shift.clone()));
    input.submit_event(InputEvent::ButtonPressed(forward.clone()));
    let pressed = evaluator.evaluate(&input.frame_snapshot());

    assert!(pressed.pressed("gameplay.dash"));
    assert!(pressed.just_activated("gameplay.dash"));
    assert!(!pressed.just_deactivated("gameplay.dash"));
    assert_eq!(
        pressed.just_activated_actions(),
        vec!["gameplay.dash".to_string()]
    );

    let ui_consumed =
        evaluator.evaluate_with_consumed_buttons(&input.frame_snapshot(), &[shift.clone()]);

    assert!(!ui_consumed.pressed("gameplay.dash"));
    assert!(!ui_consumed.just_activated("gameplay.dash"));

    input.begin_frame();
    let held = evaluator.evaluate(&input.frame_snapshot());

    assert!(held.pressed("gameplay.dash"));
    assert!(!held.just_activated("gameplay.dash"));
    assert!(!held.just_deactivated("gameplay.dash"));

    input.submit_event(InputEvent::ButtonReleased(forward));
    let released = evaluator.evaluate(&input.frame_snapshot());

    assert!(!released.pressed("gameplay.dash"));
    assert!(!released.just_activated("gameplay.dash"));
    assert!(released.just_deactivated("gameplay.dash"));
}

#[test]
fn rebinding_action_does_not_require_recompilation() {
    let mut map = InputActionMap::new();
    map.add_action(InputAction::new("gameplay.jump").with_context("gameplay"));
    map.bind(InputBinding::button(
        "gameplay.jump",
        InputButton::MouseLeft,
    ));
    let mut evaluator = InputActionEvaluator::new(map);
    let input = DefaultInputManager::default();

    input.begin_frame();
    input.submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));
    let mouse_bound = evaluator.evaluate(&input.frame_snapshot());

    assert!(mouse_bound.just_activated("gameplay.jump"));

    let mut rebound = evaluator.action_map().clone();
    rebound.clear_bindings("gameplay.jump");
    rebound.bind(InputBinding::button(
        "gameplay.jump",
        InputButton::Key("Space".to_string()),
    ));
    evaluator.set_action_map(rebound);

    input.begin_frame();
    let old_binding_held = evaluator.evaluate(&input.frame_snapshot());

    assert!(!old_binding_held.pressed("gameplay.jump"));
    assert!(!old_binding_held.just_activated("gameplay.jump"));

    input.submit_event(InputEvent::ButtonPressed(InputButton::Key(
        "Space".to_string(),
    )));
    let rebound_frame = evaluator.evaluate(&input.frame_snapshot());

    assert!(rebound_frame.pressed("gameplay.jump"));
    assert!(rebound_frame.just_activated("gameplay.jump"));
}
