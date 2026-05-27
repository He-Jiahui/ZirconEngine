use crate::core::framework::input::InputManager;

use crate::input::{
    ButtonInputState, DefaultInputManager, FileDragDropEvent, GamepadAxis, GamepadAxisSettings,
    GamepadButton, GamepadButtonAxisSettings, GamepadConnectionInfo, GamepadId,
    GamepadRumbleIntensity, GamepadRumbleRequest, ImeCursorArea, ImeCursorRange,
    ImeDeleteSurrounding, ImeEvent, ImeHostRequest, ImePreedit, ImeSurroundingText, InputButton,
    InputEvent, InputFrameSnapshot, MouseScrollUnit, MouseWheelEvent, TouchPhase,
    WindowStatusEvent, WindowTheme,
};

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
    assert_eq!(frame.wheel_accumulator, 4.0);
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
fn keyboard_focus_lost_releases_keyboard_buttons_only() {
    let input = DefaultInputManager::default();
    input.submit_event(InputEvent::KeyboardInput {
        key_code: 16,
        logical_key: Some("Shift".to_string()),
        text: None,
        pressed: true,
        repeat: false,
    });
    input.submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));

    input.begin_frame();
    input.submit_event(InputEvent::KeyboardFocusLost);
    let frame = input.frame_snapshot();

    assert!(!frame.buttons.pressed(&InputButton::KeyCode(16)));
    assert!(!frame
        .buttons
        .pressed(&InputButton::Key("Shift".to_string())));
    assert!(frame.buttons.just_released(&InputButton::KeyCode(16)));
    assert!(frame
        .buttons
        .just_released(&InputButton::Key("Shift".to_string())));
    assert!(frame.buttons.pressed(&InputButton::MouseLeft));
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

#[test]
fn input_manager_tracks_touch_and_gamepad_state() {
    let input = DefaultInputManager::default();
    let gamepad = GamepadId(7);

    input.submit_event(InputEvent::Touch {
        id: 42,
        phase: TouchPhase::Started,
        x: 11.0,
        y: 22.0,
    });
    input.submit_event(InputEvent::GamepadConnection(GamepadConnectionInfo {
        gamepad,
        connected: true,
        name: Some("Pad".to_string()),
        vendor_id: Some(1),
        product_id: Some(2),
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
        value: 0.5,
    });

    let frame = input.frame_snapshot();

    assert_eq!(frame.active_touches.len(), 1);
    assert_eq!(frame.active_touches[0].position, [11.0, 22.0]);
    assert_eq!(frame.connected_gamepads, vec![gamepad]);
    assert!(frame.buttons.pressed(&InputButton::Gamepad {
        gamepad,
        button: GamepadButton::South
    }));
    assert_eq!(frame.gamepad_axes.len(), 1);
    assert_eq!(
        frame.gamepad_axes[0].value,
        GamepadAxisSettings::default().scaled_value(0.5)
    );
    assert_eq!(frame.gamepad_button_values.len(), 1);
    assert_eq!(
        frame.gamepad_button_values[0].value,
        GamepadButtonAxisSettings::default().scaled_value(1.0)
    );

    input.submit_event(InputEvent::Touch {
        id: 42,
        phase: TouchPhase::Ended,
        x: 11.0,
        y: 22.0,
    });
    input.submit_event(InputEvent::GamepadConnection(GamepadConnectionInfo {
        gamepad,
        connected: false,
        name: None,
        vendor_id: None,
        product_id: None,
    }));

    let cleared = input.frame_snapshot();

    assert!(cleared.active_touches.is_empty());
    assert!(cleared.connected_gamepads.is_empty());
    assert!(cleared.gamepad_axes.is_empty());
    assert!(cleared.gamepad_button_values.is_empty());
    assert!(!cleared.buttons.pressed(&InputButton::Gamepad {
        gamepad,
        button: GamepadButton::South
    }));
}

#[test]
fn input_manager_event_log_harness_covers_window_keyboard_mouse_touch_and_gamepad() {
    let input = DefaultInputManager::default();
    let gamepad = GamepadId(9);

    input.begin_frame();
    input.submit_event(InputEvent::WindowStatus(WindowStatusEvent::Moved {
        x: 640,
        y: 480,
    }));
    input.submit_event(InputEvent::WindowStatus(WindowStatusEvent::CloseRequested));
    input.submit_event(InputEvent::KeyboardInput {
        key_code: 65,
        logical_key: Some("KeyA".to_string()),
        text: Some("a".to_string()),
        pressed: true,
        repeat: false,
    });
    input.submit_event(InputEvent::CursorEntered);
    input.submit_event(InputEvent::CursorMoved { x: 320.0, y: 180.0 });
    input.submit_event(InputEvent::MouseMotion {
        delta_x: 2.5,
        delta_y: -1.0,
    });
    input.submit_event(InputEvent::MouseWheel(MouseWheelEvent::lines(0.0, -1.0)));
    input.submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));
    input.submit_event(InputEvent::Touch {
        id: 12,
        phase: TouchPhase::Started,
        x: 10.0,
        y: 20.0,
    });
    input.submit_event(InputEvent::GamepadConnection(GamepadConnectionInfo {
        gamepad,
        connected: true,
        name: Some("Harness Pad".to_string()),
        vendor_id: Some(100),
        product_id: Some(200),
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
        value: 0.5,
    });

    let frame = input.frame_snapshot();

    assert_eq!(
        runtime_input_event_log_harness(&frame),
        vec![
            "window:moved(640,480)",
            "window:close_requested",
            "keyboard:key_code:65:pressed",
            "keyboard:key:KeyA:pressed",
            "mouse:cursor(320.0,180.0):inside",
            "mouse:motion(2.5,-1.0)",
            "mouse:wheel:line(0.0,-1.0)",
            "mouse:button:left:pressed",
            "touch:12:Started(10.0,20.0)",
            "gamepad:9:connected",
            "gamepad:9:South:pressed",
            "gamepad:9:LeftStickX=0.47",
        ]
    );
    assert_eq!(
        input
            .drain_event_records()
            .iter()
            .map(|record| record.sequence)
            .collect::<Vec<_>>(),
        (1..=12).collect::<Vec<_>>()
    );
}

fn runtime_input_event_log_harness(frame: &InputFrameSnapshot) -> Vec<String> {
    let mut log = Vec::new();

    for event in &frame.window_status_events {
        match event {
            WindowStatusEvent::Moved { x, y } => log.push(format!("window:moved({x},{y})")),
            WindowStatusEvent::CloseRequested => log.push("window:close_requested".to_string()),
            other => log.push(format!("window:{other:?}")),
        }
    }

    for button in frame.buttons.just_pressed_inputs() {
        match button {
            InputButton::KeyCode(code) => log.push(format!("keyboard:key_code:{code}:pressed")),
            InputButton::Key(key) => log.push(format!("keyboard:key:{key}:pressed")),
            _ => {}
        }
    }

    if frame.cursor_inside_window || frame.cursor_position != [0.0, 0.0] {
        let boundary = if frame.cursor_inside_window {
            "inside"
        } else {
            "outside"
        };
        log.push(format!(
            "mouse:cursor({:.1},{:.1}):{boundary}",
            frame.cursor_position[0], frame.cursor_position[1]
        ));
    }
    if frame.mouse_motion_accumulator != [0.0, 0.0] {
        log.push(format!(
            "mouse:motion({:.1},{:.1})",
            frame.mouse_motion_accumulator[0], frame.mouse_motion_accumulator[1]
        ));
    }
    for wheel in &frame.mouse_wheel_events {
        let unit = match wheel.unit {
            MouseScrollUnit::Line => "line",
            MouseScrollUnit::Pixel => "pixel",
        };
        log.push(format!("mouse:wheel:{unit}({:.1},{:.1})", wheel.x, wheel.y));
    }
    for button in frame.buttons.just_pressed_inputs() {
        if let Some(label) = mouse_button_label(&button) {
            log.push(format!("mouse:button:{label}:pressed"));
        }
    }

    for touch in &frame.active_touches {
        log.push(format!(
            "touch:{}:{:?}({:.1},{:.1})",
            touch.id, touch.phase, touch.position[0], touch.position[1]
        ));
    }

    for gamepad in &frame.connected_gamepads {
        log.push(format!("gamepad:{}:connected", gamepad.0));
    }
    for button in frame.buttons.just_pressed_inputs() {
        if let InputButton::Gamepad { gamepad, button } = button {
            log.push(format!("gamepad:{}:{button:?}:pressed", gamepad.0));
        }
    }
    for axis in &frame.gamepad_axes {
        log.push(format!(
            "gamepad:{}:{:?}={:.2}",
            axis.gamepad.0, axis.axis, axis.value
        ));
    }

    log
}

fn mouse_button_label(button: &InputButton) -> Option<String> {
    match button {
        InputButton::MouseLeft => Some("left".to_string()),
        InputButton::MouseRight => Some("right".to_string()),
        InputButton::MouseMiddle => Some("middle".to_string()),
        InputButton::MouseBack => Some("back".to_string()),
        InputButton::MouseForward => Some("forward".to_string()),
        InputButton::MouseOther(code) => Some(format!("other:{code}")),
        _ => None,
    }
}

#[test]
fn gamepad_button_values_use_runtime_thresholds_and_hysteresis() {
    let input = DefaultInputManager::default();
    let gamepad = GamepadId(3);
    let button = GamepadButton::South;
    let input_button = InputButton::Gamepad { gamepad, button };

    input.submit_event(InputEvent::GamepadButton {
        gamepad,
        button,
        value: 0.70,
        pressed: true,
    });
    let below_press = input.frame_snapshot();
    assert!(!below_press.buttons.pressed(&input_button));
    assert_eq!(
        below_press.gamepad_button_values[0].value,
        GamepadButtonAxisSettings::default().scaled_value(0.70)
    );

    input.submit_event(InputEvent::GamepadButton {
        gamepad,
        button,
        value: 0.80,
        pressed: true,
    });
    let pressed = input.frame_snapshot();
    assert!(pressed.buttons.pressed(&input_button));
    assert!(pressed.buttons.just_pressed(&input_button));

    input.begin_frame();
    input.submit_event(InputEvent::GamepadButton {
        gamepad,
        button,
        value: 0.70,
        pressed: true,
    });
    let held_by_hysteresis = input.frame_snapshot();
    assert!(held_by_hysteresis.buttons.pressed(&input_button));
    assert!(!held_by_hysteresis.buttons.just_pressed(&input_button));

    input.submit_event(InputEvent::GamepadButton {
        gamepad,
        button,
        value: 0.60,
        pressed: true,
    });
    let released = input.frame_snapshot();
    assert!(!released.buttons.pressed(&input_button));
    assert!(released.buttons.just_released(&input_button));
}

#[test]
fn gamepad_axis_values_use_deadzone_livezone_and_change_threshold() {
    let input = DefaultInputManager::default();
    let gamepad = GamepadId(5);
    let axis = GamepadAxis::LeftStickX;

    input.submit_event(InputEvent::GamepadAxis {
        gamepad,
        axis,
        value: 0.03,
    });
    let deadzone = input.frame_snapshot();
    assert_eq!(deadzone.gamepad_axes.len(), 1);
    assert_eq!(deadzone.gamepad_axes[0].value, 0.0);

    input.submit_event(InputEvent::GamepadAxis {
        gamepad,
        axis,
        value: 0.04,
    });
    let unchanged = input.frame_snapshot();
    assert_eq!(unchanged.gamepad_axes.len(), 1);
    assert_eq!(unchanged.gamepad_axes[0].value, 0.0);

    input.submit_event(InputEvent::GamepadAxis {
        gamepad,
        axis,
        value: 0.50,
    });
    let moved = input.frame_snapshot();
    assert_eq!(
        moved.gamepad_axes[0].value,
        GamepadAxisSettings::default().scaled_value(0.50)
    );

    input.submit_event(InputEvent::GamepadAxis {
        gamepad,
        axis,
        value: 0.505,
    });
    let filtered = input.frame_snapshot();
    assert_eq!(filtered.gamepad_axes[0].value, moved.gamepad_axes[0].value);

    input.submit_event(InputEvent::GamepadAxis {
        gamepad,
        axis,
        value: -1.25,
    });
    let clamped = input.frame_snapshot();
    assert_eq!(clamped.gamepad_axes[0].value, -1.0);
}

#[test]
fn gamepad_rumble_requests_are_frame_local_and_drainable() {
    let input = DefaultInputManager::default();
    let gamepad = GamepadId(11);
    let add = GamepadRumbleRequest::add(gamepad, GamepadRumbleIntensity::new(1.2, 0.4), 125);
    let stop = GamepadRumbleRequest::stop(gamepad);

    assert_eq!(
        GamepadRumbleIntensity::new(f32::NAN, 1.5).clamped(),
        GamepadRumbleIntensity::new(0.0, 1.0)
    );

    input.submit_event(InputEvent::GamepadRumbleRequest(add));
    input.submit_event(InputEvent::GamepadRumbleRequest(stop));

    let frame = input.frame_snapshot();
    assert_eq!(frame.gamepad_rumble_requests, vec![add, stop]);

    assert_eq!(input.drain_gamepad_rumble_requests(), vec![add, stop]);
    assert!(input.frame_snapshot().gamepad_rumble_requests.is_empty());

    input.submit_event(InputEvent::GamepadRumbleRequest(add));
    input.begin_frame();

    assert!(input.frame_snapshot().gamepad_rumble_requests.is_empty());
    assert!(input.drain_gamepad_rumble_requests().is_empty());
}
