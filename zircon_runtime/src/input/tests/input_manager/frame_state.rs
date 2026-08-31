use super::*;

#[test]
fn input_manager_tracks_state_and_drains_events() {
    let input = DefaultInputManager::default();
    input.submit_event(InputEvent::CursorMoved { x: 42.0, y: 12.0 });
    input.submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));
    input.submit_event(InputEvent::WheelScrolled { delta: 1.5 });

    let snapshot = input.snapshot();
    assert_eq!(snapshot.cursor_position, [42.0, 12.0]);
    assert_eq!(snapshot.pressed_buttons, vec![InputButton::MouseLeft]);
    assert_eq!(snapshot.wheel_accumulator, 1.5);

    let drained = input.drain_events();
    assert_eq!(drained.len(), 3);
    assert!(input.drain_events().is_empty());
}

#[test]
fn input_manager_records_sequences_and_timestamps_for_ui_bridge_consumers() {
    let input = DefaultInputManager::default();
    input.set_event_recording_config(crate::input::InputEventRecordingConfig::enabled(16));
    input.submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));
    input.submit_event(InputEvent::ButtonReleased(InputButton::MouseLeft));

    let records = input.drain_event_records();

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].sequence, 1);
    assert_eq!(records[1].sequence, 2);
    assert!(records[0].timestamp_millis > 0);
    assert!(records[1].timestamp_millis >= records[0].timestamp_millis);
    assert!(input.drain_event_records().is_empty());
}

#[test]
fn button_input_state_tracks_bevy_style_frame_transitions() {
    let mut buttons = ButtonInputState::default();
    let left = InputButton::MouseLeft;

    assert!(buttons.press(left.clone()));
    assert!(!buttons.press(left.clone()));
    assert!(buttons.pressed(&left));
    assert!(buttons.just_pressed(&left));
    assert_eq!(buttons.just_pressed_inputs(), vec![left.clone()]);

    buttons.clear_transitions();

    assert!(buttons.pressed(&left));
    assert!(!buttons.just_pressed(&left));
    assert!(buttons.release(&left));
    assert!(!buttons.release(&left));
    assert!(!buttons.pressed(&left));
    assert!(buttons.just_released(&left));
}

#[test]
fn button_release_moves_the_stored_owned_key_into_transition_state() {
    let source = include_str!("../../../core/framework/input/button_input_state.rs");
    assert!(source.contains("self.pressed.take(input)"));
    assert!(
        !source.contains("self.pressed.remove(input)"),
        "release must not discard the stored key and clone the lookup key"
    );
}

#[test]
fn input_snapshot_just_pressed_is_true_for_exactly_one_frame() {
    let input = DefaultInputManager::default();
    let key = InputButton::Key("Jump".to_string());

    input.begin_frame();
    input.submit_event(InputEvent::ButtonPressed(key.clone()));
    let pressed_frame = input.frame_snapshot();

    assert!(pressed_frame.buttons.pressed(&key));
    assert!(pressed_frame.buttons.just_pressed(&key));
    assert!(!pressed_frame.buttons.just_released(&key));

    input.begin_frame();
    let held_frame = input.frame_snapshot();

    assert!(held_frame.buttons.pressed(&key));
    assert!(!held_frame.buttons.just_pressed(&key));
    assert!(!held_frame.buttons.just_released(&key));

    input.submit_event(InputEvent::ButtonReleased(key.clone()));
    let released_frame = input.frame_snapshot();

    assert!(!released_frame.buttons.pressed(&key));
    assert!(!released_frame.buttons.just_pressed(&key));
    assert!(released_frame.buttons.just_released(&key));

    input.begin_frame();
    let cleared_frame = input.frame_snapshot();

    assert!(!cleared_frame.buttons.pressed(&key));
    assert!(!cleared_frame.buttons.just_pressed(&key));
    assert!(!cleared_frame.buttons.just_released(&key));
}

#[test]
fn frame_input_clears_after_level_tick_not_before() {
    let session_source = include_str!("../../../dynamic_api/session/state.rs");
    let tick_start = session_source
        .find("fn tick_frame(&mut self)")
        .expect("dynamic session should keep a tick_frame owner");
    let drain_start = session_source[tick_start..]
        .find("fn drain_host_requests")
        .map(|offset| tick_start + offset)
        .expect("tick_frame should stay before host-request draining");
    let tick_body = &session_source[tick_start..drain_start];
    let level_tick = tick_body
        .find(".tick(&self.runtime.handle(), advance)")
        .expect("tick_frame should advance the loaded level");
    let clear_input = tick_body
        .find(".begin_frame();")
        .expect("tick_frame should clear frame-local input through begin_frame");

    assert!(
        level_tick < clear_input,
        "runtime tick_frame must keep current-frame input visible to level systems before clearing transitions"
    );

    let input = DefaultInputManager::default();
    let button = InputButton::MouseLeft;
    input.begin_frame();
    input.submit_event(InputEvent::ButtonPressed(button.clone()));

    let visible_to_level_tick = input.frame_snapshot();
    assert!(visible_to_level_tick.buttons.just_pressed(&button));

    input.begin_frame();
    let next_frame = input.frame_snapshot();

    assert!(next_frame.buttons.pressed(&button));
    assert!(!next_frame.buttons.just_pressed(&button));
}

#[test]
fn input_manager_frame_snapshot_tracks_transitions_and_motion() {
    let input = DefaultInputManager::default();
    input.begin_frame();
    input.submit_event(InputEvent::CursorEntered);
    input.submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));
    input.submit_event(InputEvent::MouseMotion {
        delta_x: 3.0,
        delta_y: -2.0,
    });
    input.submit_event(InputEvent::WheelScrolled { delta: 2.0 });
    input.submit_event(InputEvent::MouseWheel(MouseWheelEvent::pixels(4.0, 20.0)));
    input.submit_event(InputEvent::WindowStatus(WindowStatusEvent::Moved {
        x: 10,
        y: 20,
    }));
    input.submit_event(InputEvent::WindowStatus(WindowStatusEvent::ThemeChanged(
        WindowTheme::Dark,
    )));
    input.submit_event(InputEvent::WindowStatus(
        WindowStatusEvent::BackendScaleFactorChanged { scale_factor: 2.0 },
    ));
    input.submit_event(InputEvent::WindowStatus(
        WindowStatusEvent::ScaleFactorChanged { scale_factor: 1.5 },
    ));
    input.submit_event(InputEvent::FileDragDrop(FileDragDropEvent::Hovered {
        path: "C:/tmp/asset.png".to_string(),
    }));
    input.submit_event(InputEvent::FileDragDrop(FileDragDropEvent::Dropped {
        path: "C:/tmp/asset.png".to_string(),
    }));
    input.submit_event(InputEvent::FileDragDrop(FileDragDropEvent::Cancelled));

    let frame = input.frame_snapshot();

    assert!(frame.buttons.pressed(&InputButton::MouseLeft));
    assert!(frame.buttons.just_pressed(&InputButton::MouseLeft));
    assert!(frame.cursor_inside_window);
    assert_eq!(frame.mouse_motion_accumulator, [3.0, -2.0]);
    assert_eq!(
        MouseWheelEvent::pixels(4.0, 20.0).vertical_line_delta(),
        20.0 * PIXEL_SCROLL_LINE_DELTA_SCALE
    );
    assert_eq!(
        frame.wheel_accumulator,
        2.0 + 20.0 * PIXEL_SCROLL_LINE_DELTA_SCALE
    );
    assert_eq!(frame.mouse_wheel_accumulator, [4.0, 22.0]);
    assert_eq!(frame.mouse_wheel_unit, MouseScrollUnit::Pixel);
    assert_eq!(
        frame.mouse_wheel_events,
        vec![
            MouseWheelEvent::lines(0.0, 2.0),
            MouseWheelEvent::pixels(4.0, 20.0)
        ]
    );
    assert_eq!(
        frame.window_status_events,
        vec![
            WindowStatusEvent::Moved { x: 10, y: 20 },
            WindowStatusEvent::ThemeChanged(WindowTheme::Dark),
            WindowStatusEvent::BackendScaleFactorChanged { scale_factor: 2.0 },
            WindowStatusEvent::ScaleFactorChanged { scale_factor: 1.5 }
        ]
    );
    assert_eq!(
        frame.file_drag_drop_events,
        vec![
            FileDragDropEvent::Hovered {
                path: "C:/tmp/asset.png".to_string()
            },
            FileDragDropEvent::Dropped {
                path: "C:/tmp/asset.png".to_string()
            },
            FileDragDropEvent::Cancelled
        ]
    );

    input.submit_event(InputEvent::CursorLeft);
    let cursor_left_frame = input.frame_snapshot();

    assert!(!cursor_left_frame.cursor_inside_window);

    input.begin_frame();
    let next_frame = input.frame_snapshot();

    assert!(next_frame.buttons.pressed(&InputButton::MouseLeft));
    assert!(!next_frame.buttons.just_pressed(&InputButton::MouseLeft));
    assert!(!next_frame.cursor_inside_window);
    assert_eq!(next_frame.mouse_motion_accumulator, [0.0, 0.0]);
    assert_eq!(next_frame.wheel_accumulator, 0.0);
    assert_eq!(next_frame.mouse_wheel_accumulator, [0.0, 0.0]);
    assert_eq!(next_frame.mouse_wheel_unit, MouseScrollUnit::Line);
    assert!(next_frame.mouse_wheel_events.is_empty());
    assert!(next_frame.window_status_events.is_empty());
    assert!(next_frame.file_drag_drop_events.is_empty());
}

#[test]
fn focus_loss_releases_active_input_without_disconnect() {
    let input = DefaultInputManager::default();
    let gamepad = GamepadId(7);
    input.submit_event(InputEvent::KeyboardInput {
        key_code: 16,
        logical_key: Some("Shift".to_string()),
        text: None,
        pressed: true,
        repeat: false,
    });
    input.submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));
    input.submit_event(InputEvent::GamepadConnection(GamepadConnectionInfo {
        gamepad,
        connected: true,
        name: None,
        vendor_id: None,
        product_id: None,
    }));
    input.submit_event(InputEvent::GamepadButton {
        gamepad,
        button: GamepadButton::South,
        value: 1.0,
        pressed: true,
    });
    input.submit_event(InputEvent::GamepadAxis {
        gamepad,
        axis: GamepadAxis::LeftStickX,
        value: 1.0,
    });
    input.submit_event(InputEvent::Touch {
        id: 42,
        phase: TouchPhase::Started,
        x: 12.0,
        y: 24.0,
    });
    input.submit_event(InputEvent::Ime(ImeEvent::Enabled));
    input.submit_event(InputEvent::Ime(ImeEvent::Preedit(ImePreedit::new(
        "composing",
        Some(ImeCursorRange::new(0, 9)),
    ))));

    input.begin_frame();
    input.submit_event(InputEvent::MouseMotion {
        delta_x: 4.0,
        delta_y: -2.0,
    });
    input.submit_event(InputEvent::WheelScrolled { delta: 1.5 });
    input.submit_event(InputEvent::FocusLost);
    let frame = input.frame_snapshot();

    assert!(!frame.buttons.pressed(&InputButton::KeyCode(16)));
    assert!(!frame
        .buttons
        .pressed(&InputButton::Key("Shift".to_string())));
    assert!(frame.buttons.just_released(&InputButton::KeyCode(16)));
    assert!(frame
        .buttons
        .just_released(&InputButton::Key("Shift".to_string())));
    assert!(!frame.buttons.pressed(&InputButton::MouseLeft));
    assert!(frame.buttons.just_released(&InputButton::MouseLeft));
    assert!(!frame.buttons.pressed(&InputButton::Gamepad {
        gamepad,
        button: GamepadButton::South,
    }));
    assert!(frame.buttons.just_released(&InputButton::Gamepad {
        gamepad,
        button: GamepadButton::South,
    }));
    assert!(frame.active_touches.is_empty());
    assert_eq!(frame.connected_gamepads, vec![gamepad]);
    assert!(frame.gamepad_axes.is_empty());
    assert!(frame.gamepad_button_values.is_empty());
    assert!(frame.gamepad_axis_transitions.iter().any(|transition| {
        transition.gamepad == gamepad
            && transition.axis == GamepadAxis::LeftStickX
            && transition.previous_value == 1.0
            && transition.value == 0.0
    }));
    assert_eq!(frame.mouse_motion_accumulator, [0.0, 0.0]);
    assert_eq!(frame.wheel_accumulator, 0.0);
    assert!(frame.mouse_wheel_events.is_empty());
    assert!(!frame.ime_enabled);
    assert_eq!(frame.ime_preedit, None);
}

#[test]
fn focus_loss_deactivates_held_button_and_axis_actions() {
    use crate::input::{
        InputAction, InputActionEvaluator, InputActionMap, InputAxisBinding, InputBinding,
    };

    let gamepad = GamepadId(7);
    let mut actions = InputActionMap::new();
    actions.add_action(InputAction::new("gameplay.jump"));
    actions.add_action(InputAction::new("gameplay.move_x"));
    actions.bind(InputBinding::button(
        "gameplay.jump",
        InputButton::KeyCode(32),
    ));
    actions.bind(InputBinding::axis(
        "gameplay.move_x",
        InputAxisBinding::new(gamepad, GamepadAxis::LeftStickX),
    ));
    let evaluator = InputActionEvaluator::new(actions);
    let input = DefaultInputManager::default();

    input.submit_event(InputEvent::KeyboardInput {
        key_code: 32,
        logical_key: Some("Space".to_string()),
        text: None,
        pressed: true,
        repeat: false,
    });
    input.submit_event(InputEvent::GamepadAxis {
        gamepad,
        axis: GamepadAxis::LeftStickX,
        value: 1.0,
    });
    input.begin_frame();

    let held = evaluator.evaluate(&input.frame_snapshot());
    assert!(held.pressed("gameplay.jump"));
    assert!(held.pressed("gameplay.move_x"));

    input.submit_event(InputEvent::FocusLost);

    let released = evaluator.evaluate(&input.frame_snapshot());
    assert!(!released.pressed("gameplay.jump"));
    assert!(released.just_deactivated("gameplay.jump"));
    assert!(!released.pressed("gameplay.move_x"));
    assert!(released.just_deactivated("gameplay.move_x"));
}

#[test]
fn focus_loss_after_same_frame_press_does_not_activate_an_action() {
    use crate::input::{InputAction, InputActionEvaluator, InputActionMap, InputBinding};

    let mut actions = InputActionMap::new();
    actions.add_action(InputAction::new("gameplay.interact"));
    actions.bind(InputBinding::button(
        "gameplay.interact",
        InputButton::KeyCode(69),
    ));
    let evaluator = InputActionEvaluator::new(actions);
    let input = DefaultInputManager::default();

    input.submit_event(InputEvent::KeyboardInput {
        key_code: 69,
        logical_key: Some("E".to_string()),
        text: None,
        pressed: true,
        repeat: false,
    });
    input.submit_event(InputEvent::FocusLost);

    let frame = input.frame_snapshot();
    assert!(frame.buttons.just_pressed(&InputButton::KeyCode(69)));
    assert!(frame.buttons.just_released(&InputButton::KeyCode(69)));

    let actions = evaluator.evaluate(&frame);
    assert!(!actions.pressed("gameplay.interact"));
    assert!(!actions.just_activated("gameplay.interact"));
    assert!(actions.just_deactivated("gameplay.interact"));
}

#[test]
fn focus_loss_cancels_pending_ime_and_cursor_capture_requests() {
    let input = DefaultInputManager::default();
    input.submit_event(InputEvent::ImeHostRequest(ImeHostRequest::Enable));
    input.submit_event(InputEvent::CursorHostRequest(
        CursorHostRequest::set_grab_mode(CursorGrabMode::Locked),
    ));

    input.submit_event(InputEvent::FocusLost);
    input.submit_event(InputEvent::FocusLost);

    let frame = input.frame_snapshot();
    assert_eq!(frame.ime_host_requests, vec![ImeHostRequest::Disable]);
    assert_eq!(
        frame.cursor_host_requests,
        vec![CursorHostRequest::set_grab_mode(CursorGrabMode::None)]
    );
    assert_eq!(
        input.drain_ime_host_requests(),
        vec![ImeHostRequest::Disable]
    );
    assert_eq!(
        input.drain_cursor_host_requests(),
        vec![CursorHostRequest::set_grab_mode(CursorGrabMode::None)]
    );
}

#[test]
fn input_manager_tracks_ime_preedit_and_frame_commits() {
    let input = DefaultInputManager::default();

    input.submit_event(InputEvent::Ime(ImeEvent::Enabled));
    input.submit_event(InputEvent::Ime(ImeEvent::Preedit(ImePreedit::new(
        "ni",
        Some(ImeCursorRange::new(0, 2)),
    ))));
    input.submit_event(InputEvent::Ime(ImeEvent::Commit("你".to_string())));
    input.submit_event(InputEvent::Ime(ImeEvent::DeleteSurrounding(
        ImeDeleteSurrounding::new(1, 2),
    )));
    input.submit_event(InputEvent::ImeHostRequest(ImeHostRequest::Enable));
    input.submit_event(InputEvent::ImeHostRequest(ImeHostRequest::SetCursorArea(
        ImeCursorArea::new(16.0, 24.0, 1.0, 18.0),
    )));
    input.submit_event(InputEvent::ImeHostRequest(
        ImeHostRequest::SetSurroundingText(ImeSurroundingText::new("hello", 5, 0)),
    ));

    let frame = input.frame_snapshot();

    assert!(frame.ime_enabled);
    assert_eq!(frame.ime_preedit, None);
    assert_eq!(frame.ime_commits, vec!["你".to_string()]);
    assert_eq!(
        frame.ime_delete_surrounding,
        vec![ImeDeleteSurrounding::new(1, 2)]
    );
    assert_eq!(
        frame.ime_host_requests,
        vec![
            ImeHostRequest::Enable,
            ImeHostRequest::SetCursorArea(ImeCursorArea::new(16.0, 24.0, 1.0, 18.0)),
            ImeHostRequest::SetSurroundingText(ImeSurroundingText::new("hello", 5, 0))
        ]
    );

    let drained_host_requests = input.drain_ime_host_requests();
    assert_eq!(
        drained_host_requests,
        vec![
            ImeHostRequest::Enable,
            ImeHostRequest::SetCursorArea(ImeCursorArea::new(16.0, 24.0, 1.0, 18.0)),
            ImeHostRequest::SetSurroundingText(ImeSurroundingText::new("hello", 5, 0))
        ]
    );
    assert!(input.frame_snapshot().ime_host_requests.is_empty());

    input.begin_frame();
    let next_frame = input.frame_snapshot();

    assert!(next_frame.ime_enabled);
    assert!(next_frame.ime_commits.is_empty());
    assert!(next_frame.ime_delete_surrounding.is_empty());
    assert!(next_frame.ime_host_requests.is_empty());

    input.submit_event(InputEvent::ImeHostRequest(ImeHostRequest::Enable));
    input.begin_frame();
    assert!(input.frame_snapshot().ime_host_requests.is_empty());
    assert_eq!(
        input.drain_ime_host_requests(),
        vec![ImeHostRequest::Enable]
    );
    assert!(input.drain_ime_host_requests().is_empty());

    input.submit_event(InputEvent::Ime(ImeEvent::Preedit(ImePreedit::new(
        "hao", None,
    ))));
    let preedit_frame = input.frame_snapshot();

    assert_eq!(
        preedit_frame.ime_preedit,
        Some(ImePreedit::new("hao", None))
    );

    input.submit_event(InputEvent::Ime(ImeEvent::Disabled));
    let disabled_frame = input.frame_snapshot();

    assert!(!disabled_frame.ime_enabled);
    assert_eq!(disabled_frame.ime_preedit, None);
}
