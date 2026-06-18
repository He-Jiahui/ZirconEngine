use crate::core::framework::input::InputManager;
use crate::core::manager::{resolve_input_action_manager, resolve_input_manager};
use crate::core::CoreRuntime;

use crate::input::{
    module_descriptor_with_config, DefaultInputManager, GamepadAxis, GamepadAxisInput,
    GamepadAxisSettings, GamepadId, InputAction, InputActionContext, InputActionEvaluator,
    InputActionMap, InputAxisBinding, InputAxisDirection, InputBinding, InputButton, InputConfig,
    InputEvent, INPUT_MODULE_NAME,
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

#[test]
fn action_contexts_filter_gameplay_and_menu_maps_without_rebinding() {
    let activate = InputButton::Key("Enter".to_string());
    let mut map = InputActionMap::new()
        .with_context(InputActionContext::new("gameplay").with_priority(10))
        .with_context(InputActionContext::new("menu"));
    map.add_action(InputAction::new("gameplay.interact").with_context("gameplay"));
    map.add_action(InputAction::new("menu.confirm").with_context("menu"));
    map.add_action(InputAction::new("global.pause"));
    map.bind(InputBinding::button("gameplay.interact", activate.clone()));
    map.bind(InputBinding::button("menu.confirm", activate.clone()));
    map.bind(InputBinding::button("global.pause", activate.clone()));
    let evaluator = InputActionEvaluator::new(map);
    let input = DefaultInputManager::default();

    input.begin_frame();
    input.submit_event(InputEvent::ButtonPressed(activate.clone()));
    let frame = input.frame_snapshot();

    let gameplay = evaluator.evaluate_with_active_contexts(&frame, &["gameplay"]);
    assert!(gameplay.just_activated("gameplay.interact"));
    assert!(!gameplay.just_activated("menu.confirm"));
    assert!(gameplay.just_activated("global.pause"));

    let menu = evaluator.evaluate_with_active_contexts(&frame, &["menu"]);
    assert!(!menu.just_activated("gameplay.interact"));
    assert!(menu.just_activated("menu.confirm"));
    assert!(menu.just_activated("global.pause"));

    let ui_consumed = evaluator.evaluate_with_active_contexts_and_consumed_buttons(
        &frame,
        &["gameplay"],
        &[activate],
    );
    assert!(!ui_consumed.just_activated("gameplay.interact"));
    assert!(!ui_consumed.just_activated("global.pause"));
}

#[test]
fn gamepad_axis_binding_reports_continuous_action_value() {
    let pad = GamepadId(7);
    let axis = GamepadAxis::LeftStickX;
    let modifier = InputButton::Key("Shift".to_string());
    let mut map = InputActionMap::new()
        .with_context(InputActionContext::new("gameplay").with_priority(10))
        .with_context(InputActionContext::new("menu"));
    map.add_action(InputAction::new("gameplay.move_x").with_context("gameplay"));
    map.add_action(InputAction::new("gameplay.move_right").with_context("gameplay"));
    map.add_action(InputAction::new("gameplay.move_left").with_context("gameplay"));
    map.add_action(InputAction::new("gameplay.aim_x").with_context("gameplay"));
    map.add_action(InputAction::new("menu.move_x").with_context("menu"));
    map.bind(InputBinding::axis(
        "gameplay.move_x",
        InputAxisBinding::new(pad, axis),
    ));
    map.bind(InputBinding::axis(
        "gameplay.move_right",
        InputAxisBinding::positive(pad, axis),
    ));
    map.bind(InputBinding::axis(
        "gameplay.move_left",
        InputAxisBinding {
            gamepad: pad,
            axis,
            direction: InputAxisDirection::Negative,
        },
    ));
    map.bind(InputBinding::buttons_and_axes(
        "gameplay.aim_x",
        [modifier.clone()],
        [InputAxisBinding::new(pad, axis)],
    ));
    map.bind(InputBinding::axis(
        "menu.move_x",
        InputAxisBinding::new(pad, axis),
    ));
    let evaluator = InputActionEvaluator::new(map);
    let input = DefaultInputManager::default();

    input.begin_frame();
    input.submit_event(InputEvent::ButtonPressed(modifier.clone()));
    input.submit_event(InputEvent::GamepadAxis {
        gamepad: pad,
        axis,
        value: 0.75,
    });
    let positive = evaluator.evaluate_with_active_contexts(&input.frame_snapshot(), &["gameplay"]);
    let expected_positive = GamepadAxisSettings::default().scaled_value(0.75);

    assert!(positive.pressed("gameplay.move_x"));
    assert!(positive.pressed("gameplay.move_right"));
    assert!(!positive.pressed("gameplay.move_left"));
    assert!(positive.pressed("gameplay.aim_x"));
    assert!(!positive.pressed("menu.move_x"));
    assert_close(positive.value("gameplay.move_x"), expected_positive);
    assert_close(positive.value("gameplay.move_right"), expected_positive);
    assert_close(positive.value("gameplay.move_left"), 0.0);
    assert_close(positive.value("gameplay.aim_x"), expected_positive);
    assert_close(positive.value("menu.move_x"), 0.0);
    assert!(positive.just_activated("gameplay.move_x"));
    assert!(positive.just_activated("gameplay.aim_x"));

    let consumed_gate = evaluator.evaluate_with_active_contexts_and_consumed_buttons(
        &input.frame_snapshot(),
        &["gameplay"],
        &[modifier],
    );
    assert!(!consumed_gate.pressed("gameplay.aim_x"));
    assert_close(consumed_gate.value("gameplay.aim_x"), 0.0);

    let consumed_axis = evaluator.evaluate_with_active_contexts_and_consumed_input(
        &input.frame_snapshot(),
        &["gameplay"],
        &[],
        &[GamepadAxisInput::new(pad, axis)],
    );
    assert!(!consumed_axis.pressed("gameplay.move_x"));
    assert!(!consumed_axis.just_activated("gameplay.move_x"));
    assert_close(consumed_axis.value("gameplay.move_x"), 0.0);

    let menu = evaluator.evaluate_with_active_contexts(&input.frame_snapshot(), &["menu"]);
    assert!(menu.pressed("menu.move_x"));
    assert_close(menu.value("menu.move_x"), expected_positive);
    assert_close(menu.value("gameplay.move_x"), 0.0);

    input.begin_frame();
    input.submit_event(InputEvent::GamepadAxis {
        gamepad: pad,
        axis,
        value: -0.75,
    });
    let negative = evaluator.evaluate_with_active_contexts(&input.frame_snapshot(), &["gameplay"]);
    let expected_negative = GamepadAxisSettings::default().scaled_value(-0.75);

    assert!(negative.pressed("gameplay.move_x"));
    assert!(!negative.pressed("gameplay.move_right"));
    assert!(negative.pressed("gameplay.move_left"));
    assert_close(negative.value("gameplay.move_x"), expected_negative);
    assert_close(negative.value("gameplay.move_right"), 0.0);
    assert_close(
        negative.value("gameplay.move_left"),
        expected_negative.abs(),
    );
}

#[test]
fn consumed_gamepad_axis_does_not_activate_gameplay_action() {
    let pad = GamepadId(3);
    let axis = GamepadAxis::LeftStickY;
    let mut map = InputActionMap::new()
        .with_context(InputActionContext::new("gameplay").with_priority(10))
        .with_context(InputActionContext::new("menu").with_priority(20));
    map.add_action(InputAction::new("gameplay.move_y").with_context("gameplay"));
    map.add_action(InputAction::new("menu.navigate_y").with_context("menu"));
    map.bind(InputBinding::axis(
        "gameplay.move_y",
        InputAxisBinding::new(pad, axis),
    ));
    map.bind(InputBinding::axis(
        "menu.navigate_y",
        InputAxisBinding::new(pad, axis),
    ));
    let evaluator = InputActionEvaluator::new(map);
    let input = DefaultInputManager::default();

    input.begin_frame();
    input.submit_event(InputEvent::GamepadAxis {
        gamepad: pad,
        axis,
        value: 0.85,
    });
    let frame = input.frame_snapshot();
    let expected_value = GamepadAxisSettings::default().scaled_value(0.85);

    let gameplay = evaluator.evaluate_with_active_contexts_and_consumed_input(
        &frame,
        &["gameplay"],
        &[],
        &[GamepadAxisInput::new(pad, axis)],
    );
    assert!(!gameplay.pressed("gameplay.move_y"));
    assert!(!gameplay.just_activated("gameplay.move_y"));
    assert!(!gameplay.just_deactivated("gameplay.move_y"));
    assert_close(gameplay.value("gameplay.move_y"), 0.0);

    let consumed_without_context_filter =
        evaluator.evaluate_with_consumed_input(&frame, &[], &[GamepadAxisInput::new(pad, axis)]);
    assert!(!consumed_without_context_filter.pressed("gameplay.move_y"));
    assert!(!consumed_without_context_filter.pressed("menu.navigate_y"));
    assert_close(
        consumed_without_context_filter.value("gameplay.move_y"),
        0.0,
    );
    assert_close(
        consumed_without_context_filter.value("menu.navigate_y"),
        0.0,
    );

    let menu = evaluator.evaluate_with_active_contexts(&frame, &["menu"]);
    assert!(menu.pressed("menu.navigate_y"));
    assert!(menu.just_activated("menu.navigate_y"));
    assert_close(menu.value("menu.navigate_y"), expected_value);
}

#[test]
fn input_config_builds_action_evaluator_from_serialized_action_map() {
    let activate = InputButton::Key("Enter".to_string());
    let alternate = InputButton::Key("Space".to_string());
    let mut map =
        InputActionMap::new().with_context(InputActionContext::new("gameplay").with_priority(10));
    map.add_action(InputAction::new("gameplay.confirm").with_context("gameplay"));
    map.bind(InputBinding::button("gameplay.confirm", activate.clone()));
    let config = InputConfig::default()
        .with_enabled(true)
        .with_action_map(map);
    let serialized = serde_json::to_string(&config).expect("InputConfig should serialize");
    let restored: InputConfig =
        serde_json::from_str(&serialized).expect("InputConfig should deserialize");
    assert_eq!(restored, config);

    let evaluator = restored.action_evaluator();
    let input = DefaultInputManager::default();
    input.begin_frame();
    input.submit_event(InputEvent::ButtonPressed(activate));
    let active = evaluator.evaluate_with_active_contexts(&input.frame_snapshot(), &["gameplay"]);

    assert!(active.just_activated("gameplay.confirm"));

    let mut rebound = restored.action_map.clone();
    rebound.clear_bindings("gameplay.confirm");
    rebound.bind(InputBinding::button("gameplay.confirm", alternate.clone()));
    let rebound_config = restored.with_action_map(rebound);
    let rebound_evaluator = rebound_config.action_evaluator();

    input.begin_frame();
    input.submit_event(InputEvent::ButtonPressed(alternate));
    let rebound_state =
        rebound_evaluator.evaluate_with_active_contexts(&input.frame_snapshot(), &["gameplay"]);

    assert!(rebound_state.just_activated("gameplay.confirm"));

    let disabled_config = rebound_config.with_enabled(false);
    let disabled_evaluator = disabled_config.action_evaluator();
    input.begin_frame();
    input.submit_event(InputEvent::ButtonPressed(InputButton::Key(
        "Space".to_string(),
    )));
    let disabled_state =
        disabled_evaluator.evaluate_with_active_contexts(&input.frame_snapshot(), &["gameplay"]);

    assert!(!disabled_state.pressed("gameplay.confirm"));
    assert!(!disabled_state.just_activated("gameplay.confirm"));
}

#[test]
fn input_action_manager_resolves_from_runtime_module_descriptor() {
    let activate = InputButton::Key("Enter".to_string());
    let alternate = InputButton::Key("Space".to_string());
    let mut map =
        InputActionMap::new().with_context(InputActionContext::new("gameplay").with_priority(10));
    map.add_action(InputAction::new("gameplay.confirm").with_context("gameplay"));
    map.bind(InputBinding::button("gameplay.confirm", activate.clone()));

    let runtime = CoreRuntime::new();
    runtime
        .register_module(module_descriptor_with_config(
            InputConfig::default()
                .with_enabled(true)
                .with_action_map(map),
        ))
        .expect("register input module");
    runtime
        .activate_module(INPUT_MODULE_NAME)
        .expect("activate input module");
    let input = resolve_input_manager(&runtime.handle()).expect("resolve input manager");
    let actions =
        resolve_input_action_manager(&runtime.handle()).expect("resolve input action manager");

    input.begin_frame();
    input.submit_event(InputEvent::ButtonPressed(activate));
    let active =
        actions.evaluate_actions_with_active_contexts(&input.frame_snapshot(), &["gameplay"]);

    assert!(active.just_activated("gameplay.confirm"));

    let mut rebound = actions.action_map();
    rebound.clear_bindings("gameplay.confirm");
    rebound.bind(InputBinding::button("gameplay.confirm", alternate.clone()));
    actions.set_action_map(rebound);

    input.begin_frame();
    input.submit_event(InputEvent::ButtonPressed(alternate));
    let rebound_state =
        actions.evaluate_actions_with_active_contexts(&input.frame_snapshot(), &["gameplay"]);

    assert!(rebound_state.just_activated("gameplay.confirm"));
}

fn assert_close(left: f32, right: f32) {
    assert!(
        (left - right).abs() <= 0.0001,
        "expected {left} to be within 0.0001 of {right}"
    );
}
