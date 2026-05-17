use crate::core::framework::input::InputManager;

use crate::input::{
    ButtonInputState, DefaultInputManager, FileDragDropEvent, GamepadAxis, GamepadButton,
    GamepadConnectionInfo, GamepadId, ImeCursorArea, ImeCursorRange, ImeDeleteSurrounding,
    ImeEvent, ImeHostRequest, ImePreedit, ImeSurroundingText, InputButton, InputEvent,
    MouseScrollUnit, MouseWheelEvent, TouchPhase, WindowStatusEvent, WindowTheme,
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
    assert_eq!(frame.gamepad_axes[0].value, 0.5);

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
    assert!(!cleared.buttons.pressed(&InputButton::Gamepad {
        gamepad,
        button: GamepadButton::South
    }));
}
