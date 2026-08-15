use crate::core::framework::input::InputManager;
use crate::core::manager::ManagerResolver;
use crate::core::CoreRuntime;

use crate::input::{
    module_descriptor_with_config, DefaultInputManager, GamepadAxis, GamepadAxisInput,
    GamepadAxisSettings, GamepadAxisState, GamepadAxisTransition, GamepadId, InputAction,
    InputActionContext, InputActionEvaluator, InputActionMap, InputAxisBinding, InputAxisDirection,
    InputBinding, InputButton, InputConfig, InputEvent, InputFrameSnapshot, INPUT_MODULE_NAME,
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
fn replacing_action_map_rebuilds_bindings_automatically() {
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
    let resolver = ManagerResolver::new(runtime.handle());
    let input = resolver
        .resolve(resolver.input_handle().expect("input manager handle"))
        .expect("resolve input manager");
    let actions = resolver
        .resolve(
            resolver
                .input_actions_handle()
                .expect("input action manager handle"),
        )
        .expect("resolve input action manager");

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

#[test]
fn action_evaluator_indexes_10_100_1000_and_10000_bindings_once() {
    for binding_count in [10, 100, 1_000, 10_000] {
        let evaluator = InputActionEvaluator::new(action_map_with_unique_bindings(binding_count));

        assert_eq!(
            evaluator.indexed_binding_candidate_count(),
            binding_count,
            "a stable action index should inspect each configured binding once, not {binding_count} squared candidates"
        );
        assert!(evaluator
            .evaluate(&Default::default())
            .pressed_actions()
            .is_empty());
        assert_eq!(evaluator.evaluation_binding_visit_count(), binding_count);
        assert_eq!(evaluator.evaluation_output_action_count(), 0);
    }
}

#[test]
fn action_evaluator_indexes_axis_frame_sources_once_for_10_100_1000_and_10000_bindings() {
    for binding_count in [10, 100, 1_000, 10_000] {
        let (action_map, frame) = action_map_with_unique_axis_bindings(binding_count);
        let evaluator = InputActionEvaluator::new(action_map);

        let state = evaluator.evaluate(&frame);

        assert_eq!(state.pressed_actions().len(), binding_count);
        assert_eq!(evaluator.evaluation_binding_visit_count(), binding_count);
        assert_eq!(evaluator.evaluation_output_action_count(), binding_count);
        assert_eq!(
            evaluator.evaluation_axis_source_visit_count(),
            binding_count * 2,
            "frame axis state and transition sources should each be indexed once"
        );
    }
}

#[test]
fn action_evaluator_records_generation_builds_and_distinct_projected_actions() {
    let (initial_map, active_frame) = action_map_with_unique_axis_bindings(2);
    let mut evaluator = InputActionEvaluator::new(initial_map);

    assert_eq!(evaluator.evaluation_generation_build_count(), 1);
    let initial_state = evaluator.evaluate(&active_frame);
    assert_eq!(initial_state.pressed_actions().len(), 2);
    assert_eq!(evaluator.evaluation_output_action_count(), 2);

    evaluator.set_action_map(InputActionMap::default());

    assert_eq!(evaluator.evaluation_generation_build_count(), 2);
    let replaced_state = evaluator.evaluate(&active_frame);
    assert!(replaced_state.pressed_actions().is_empty());
    assert_eq!(evaluator.evaluation_output_action_count(), 0);
}

#[test]
fn action_evaluator_reuses_workspace_after_axis_warmup() {
    let (action_map, frame) = action_map_with_unique_axis_bindings(1_000);
    let evaluator = InputActionEvaluator::new(action_map);

    let consumed_button = InputButton::KeyCode(1);
    let consumed_axis = GamepadAxisInput::new(GamepadId(1), GamepadAxis::LeftStickX);
    let first = evaluator.evaluate_with_consumed_input(
        &frame,
        &[consumed_button.clone()],
        &[consumed_axis],
    );
    let growth_after_warmup = evaluator.workspace_storage_growth_count();
    let second =
        evaluator.evaluate_with_consumed_input(&frame, &[consumed_button], &[consumed_axis]);

    assert_eq!(second, first);
    assert_eq!(
        evaluator.workspace_storage_growth_count(),
        growth_after_warmup,
        "steady action evaluation must reuse its axis and action workspace after warm-up, including filtered-input calls"
    );
    assert_eq!(evaluator.evaluation_binding_visit_count(), 1_000);
    assert_eq!(evaluator.evaluation_axis_source_visit_count(), 2_000);
}

#[test]
fn action_evaluator_reuses_consumed_button_index_at_10000_bindings() {
    const BINDING_COUNT: usize = 10_000;

    let evaluator = InputActionEvaluator::new(action_map_with_unique_bindings(BINDING_COUNT));
    let mut frame = InputFrameSnapshot::default();
    frame
        .buttons
        .press(InputButton::KeyCode((BINDING_COUNT - 1) as u32));
    let consumed_buttons = (0..BINDING_COUNT)
        .map(|index| InputButton::KeyCode(index as u32))
        .collect::<Vec<_>>();

    let first = evaluator.evaluate_with_consumed_buttons(&frame, &consumed_buttons);
    let growth_after_warmup = evaluator.workspace_storage_growth_count();
    let second = evaluator.evaluate_with_consumed_buttons(&frame, &consumed_buttons);

    assert!(first.pressed_actions().is_empty());
    assert_eq!(second, first);
    assert_eq!(
        evaluator.evaluation_consumed_input_source_visit_count(),
        BINDING_COUNT
    );
    assert_eq!(
        evaluator.workspace_storage_growth_count(),
        growth_after_warmup,
        "a repeated large UI-consumed button set must reuse the evaluator workspace"
    );
}

#[test]
fn action_evaluator_reuses_consumed_axis_index_at_10000_bindings() {
    const BINDING_COUNT: usize = 10_000;

    let (action_map, frame) = action_map_with_unique_axis_bindings(BINDING_COUNT);
    let evaluator = InputActionEvaluator::new(action_map);
    let consumed_axes = (0..BINDING_COUNT)
        .map(|index| GamepadAxisInput::new(GamepadId(index as u64), GamepadAxis::LeftStickX))
        .collect::<Vec<_>>();

    let first = evaluator.evaluate_with_consumed_input(&frame, &[], &consumed_axes);
    let growth_after_warmup = evaluator.workspace_storage_growth_count();
    let second = evaluator.evaluate_with_consumed_input(&frame, &[], &consumed_axes);

    assert!(first.pressed_actions().is_empty());
    assert_eq!(second, first);
    assert_eq!(
        evaluator.evaluation_consumed_input_source_visit_count(),
        BINDING_COUNT
    );
    assert_eq!(
        evaluator.workspace_storage_growth_count(),
        growth_after_warmup,
        "a repeated large UI-consumed axis set must reuse the evaluator workspace"
    );
}

#[test]
fn input_action_evaluator_preserves_send_and_sync_public_boundary() {
    fn assert_send_and_sync<T: Send + Sync>() {}

    assert_send_and_sync::<InputActionEvaluator>();
}

#[test]
fn replacing_an_action_map_rebuilds_the_compiled_generation() {
    let mut evaluator = InputActionEvaluator::new(action_map_with_unique_bindings(10));
    assert_eq!(evaluator.indexed_binding_candidate_count(), 10);

    evaluator.set_action_map(action_map_with_unique_bindings(3));

    assert_eq!(evaluator.indexed_binding_candidate_count(), 3);
}

#[test]
fn duplicate_gamepad_axis_samples_keep_the_last_source_sample() {
    let gamepad = GamepadId(91);
    let axis = GamepadAxis::LeftStickX;
    let mut map = InputActionMap::new();
    map.add_action(InputAction::new("gameplay.move_x"));
    map.bind(InputBinding::axis(
        "gameplay.move_x",
        InputAxisBinding::new(gamepad, axis),
    ));
    let evaluator = InputActionEvaluator::new(map);
    let mut frame = InputFrameSnapshot::default();
    frame.gamepad_axes.extend([
        GamepadAxisState {
            gamepad,
            axis,
            value: -0.25,
        },
        GamepadAxisState {
            gamepad,
            axis,
            value: 0.75,
        },
    ]);
    frame.gamepad_axis_transitions.extend([
        GamepadAxisTransition {
            gamepad,
            axis,
            previous_value: -0.5,
            value: -0.25,
        },
        GamepadAxisTransition {
            gamepad,
            axis,
            previous_value: -0.25,
            value: 0.75,
        },
    ]);

    let state = evaluator.evaluate(&frame);

    assert!(state.pressed("gameplay.move_x"));
    assert_close(state.value("gameplay.move_x"), 0.75);
    assert!(!state.just_activated("gameplay.move_x"));
    assert!(!state.just_deactivated("gameplay.move_x"));
}

#[test]
fn action_evaluator_hot_path_uses_compiled_contexts_and_one_axis_pass() {
    let evaluator_source = include_str!("../runtime/action_evaluator.rs");
    assert!(
        !evaluator_source.contains("self.action_map.context_enabled(context)"),
        "context enabled-state must come from the compiled evaluator index"
    );
    assert!(
        evaluator_source.contains("evaluate_binding_axes("),
        "axis value and transition state must be evaluated in one pass"
    );
    assert!(
        !evaluator_source.contains("binding_axis_value(&frame_axes"),
        "axis value must not trigger a second binding-axis traversal"
    );
    assert!(
        evaluator_source.contains("workspace.consumed_inputs()"),
        "consumed inputs must use the reusable workspace index"
    );
    assert!(
        !evaluator_source.contains("consumed_buttons.contains(button)"),
        "large consumed button sets must not be linearly scanned for every binding"
    );
    assert!(
        !evaluator_source.contains("consumed_axes.contains(&axis_input)"),
        "large consumed axis sets must not be linearly scanned for every binding axis"
    );

    let consumed_index_source = include_str!("../runtime/action_evaluator/consumed_input_index.rs");
    assert!(
        consumed_index_source.contains("sort_unstable_by"),
        "consumed input membership must use an in-place sorted index"
    );
    assert!(
        consumed_index_source.contains("binary_search_by"),
        "consumed input membership must remain logarithmic after index preparation"
    );

    let axis_index_source = include_str!("../runtime/action_evaluator/frame_axis_index.rs");
    assert!(
        axis_index_source
            .contains("sort_unstable_by_key(|value| (value.input, value.source_index))"),
        "axis lookup sorting must remain in-place while retaining source order as a tie-breaker"
    );
    assert!(
        !axis_index_source.contains(".sort_by_key("),
        "stable axis sorting may allocate outside the reusable evaluator workspace"
    );

    let action_manager_source = include_str!("../runtime/default_input_action_manager.rs");
    assert_eq!(
        action_manager_source
            .matches("evaluate_while_manager_locked")
            .count(),
        6,
        "the default manager must reuse its evaluator lock for every action-evaluation entry point"
    );

    let descriptor_source = include_str!("../module/descriptor.rs");
    assert!(
        !descriptor_source.contains("let action_config = config.clone();"),
        "input module descriptor must move its owned config into the manager factory"
    );
}

fn action_map_with_unique_bindings(binding_count: usize) -> InputActionMap {
    let mut actions = Vec::with_capacity(binding_count);
    let mut bindings = Vec::with_capacity(binding_count);
    for index in 0..binding_count {
        let action = format!("gameplay.action_{index}");
        actions.push(InputAction::new(action.clone()));
        bindings.push(InputBinding::button(
            action,
            InputButton::KeyCode(index as u32),
        ));
    }
    InputActionMap {
        contexts: Vec::new(),
        actions,
        bindings,
    }
}

fn action_map_with_unique_axis_bindings(
    binding_count: usize,
) -> (InputActionMap, InputFrameSnapshot) {
    let mut actions = Vec::with_capacity(binding_count);
    let mut bindings = Vec::with_capacity(binding_count);
    let mut frame = InputFrameSnapshot::default();
    for index in 0..binding_count {
        let action = format!("gameplay.axis_action_{index}");
        let gamepad = GamepadId(index as u64);
        actions.push(InputAction::new(action.clone()));
        bindings.push(InputBinding::axis(
            action,
            InputAxisBinding::new(gamepad, GamepadAxis::LeftStickX),
        ));
        frame.gamepad_axes.push(GamepadAxisState {
            gamepad,
            axis: GamepadAxis::LeftStickX,
            value: 0.5,
        });
        frame.gamepad_axis_transitions.push(GamepadAxisTransition {
            gamepad,
            axis: GamepadAxis::LeftStickX,
            previous_value: 0.0,
            value: 0.5,
        });
    }
    (
        InputActionMap {
            contexts: Vec::new(),
            actions,
            bindings,
        },
        frame,
    )
}

fn assert_close(left: f32, right: f32) {
    assert!(
        (left - right).abs() <= 0.0001,
        "expected {left} to be within 0.0001 of {right}"
    );
}
