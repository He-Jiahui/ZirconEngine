use crate::core::framework::input::InputManager;
use crate::input::{
    CursorGrabMode, CursorHostRequest, DefaultInputManager, GamepadAxis, GamepadId, ImeHostRequest,
    InputButton, InputEvent, InputEventRecordingConfig, InputRecording, InputRecordingFrame,
};

#[test]
fn input_recording_captures_drainable_event_records_by_frame() {
    let input = DefaultInputManager::default();
    input.set_event_recording_config(InputEventRecordingConfig::enabled(16));
    input.submit_event(InputEvent::CursorMoved { x: 4.0, y: 9.0 });
    input.submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));

    let frame = InputRecordingFrame::capture_from_manager(17, &input);
    assert_eq!(frame.frame_index(), 17);
    assert_eq!(frame.event_count(), 2);
    assert!(frame.is_complete());
    assert!(input.drain_event_records().is_empty());

    let mut recording = InputRecording::new();
    recording.push_frame(frame.clone());

    assert_eq!(recording.frame_count(), 1);
    assert_eq!(recording.event_count(), 2);
    assert!(recording.is_complete());
    assert_eq!(recording.frames(), &[frame]);
}

#[test]
fn input_recording_marks_a_bounded_capture_incomplete_after_discard() {
    let input = DefaultInputManager::default();
    input.set_event_recording_config(InputEventRecordingConfig::enabled(2));
    input.submit_event(InputEvent::CursorMoved { x: 1.0, y: 0.0 });
    input.submit_event(InputEvent::CursorMoved { x: 2.0, y: 0.0 });
    input.submit_event(InputEvent::CursorMoved { x: 3.0, y: 0.0 });

    let frame = InputRecordingFrame::capture_from_manager(5, &input);
    assert_eq!(frame.event_count(), 2);
    assert_eq!(frame.discarded_record_count(), 1);
    assert!(!frame.is_complete());

    let recording = InputRecording::from_frames(vec![frame]);
    assert_eq!(recording.discarded_record_count(), 1);
    assert!(!recording.is_complete());
}

#[test]
fn input_recording_marks_capture_incomplete_when_recording_is_disabled() {
    let input = DefaultInputManager::default();
    input.submit_event(InputEvent::CursorMoved { x: 1.0, y: 2.0 });

    let frame = InputRecordingFrame::capture_from_manager(6, &input);

    assert!(!frame.recording_enabled());
    assert!(frame.is_empty());
    assert!(!frame.is_complete());
    assert!(!InputRecording::from_frames(vec![frame]).is_complete());
}

#[test]
fn input_replay_restores_frame_snapshots_in_recorded_order() {
    let key = InputButton::KeyCode(42);
    let recording = InputRecording::from_frames(vec![
        InputRecordingFrame::from_events(
            3,
            [
                InputEvent::CursorMoved { x: 8.0, y: 13.0 },
                InputEvent::ButtonPressed(key.clone()),
            ],
        ),
        InputRecordingFrame::from_events(4, [InputEvent::ButtonReleased(key.clone())]),
    ]);
    let input = DefaultInputManager::default();
    let mut cursor = recording.replay_cursor();

    assert_eq!(cursor.next_recording_frame_index(), Some(3));
    let pressed = cursor.replay_next_frame(&input).unwrap();
    assert_eq!(pressed.frame_index, 3);
    assert_eq!(pressed.event_count, 2);
    assert_eq!(pressed.snapshot.cursor_position, [8.0, 13.0]);
    assert!(pressed.snapshot.buttons.pressed(&key));
    assert!(pressed.snapshot.buttons.just_pressed(&key));

    assert_eq!(cursor.next_recording_frame_index(), Some(4));
    let released = cursor.replay_next_frame(&input).unwrap();
    assert_eq!(released.frame_index, 4);
    assert_eq!(released.event_count, 1);
    assert!(!released.snapshot.buttons.pressed(&key));
    assert!(released.snapshot.buttons.just_released(&key));
    assert!(cursor.is_finished());
    assert!(cursor.replay_next_frame(&input).is_none());
}

#[test]
fn input_replay_applies_focus_loss_as_a_release_transaction() {
    let key = InputButton::KeyCode(42);
    let gamepad = GamepadId(7);
    let axis = GamepadAxis::LeftStickX;
    let recording = InputRecording::from_frames(vec![
        InputRecordingFrame::from_events(
            3,
            [
                InputEvent::ButtonPressed(key.clone()),
                InputEvent::GamepadAxis {
                    gamepad,
                    axis,
                    value: 1.0,
                },
                InputEvent::ImeHostRequest(ImeHostRequest::Enable),
                InputEvent::CursorHostRequest(CursorHostRequest::set_grab_mode(
                    CursorGrabMode::Locked,
                )),
            ],
        ),
        InputRecordingFrame::from_events(4, [InputEvent::FocusLost]),
    ]);
    let input = DefaultInputManager::default();
    let mut cursor = recording.replay_cursor();

    let held = cursor.replay_next_frame(&input).unwrap();
    assert!(held.snapshot.buttons.pressed(&key));
    assert_eq!(held.snapshot.gamepad_axes.len(), 1);

    let released = cursor.replay_next_frame(&input).unwrap();
    assert!(!released.snapshot.buttons.pressed(&key));
    assert!(released.snapshot.buttons.just_released(&key));
    assert!(released.snapshot.gamepad_axes.is_empty());
    assert!(released
        .snapshot
        .gamepad_axis_transitions
        .iter()
        .any(|transition| {
            transition.gamepad == gamepad
                && transition.axis == axis
                && transition.previous_value == 1.0
                && transition.value == 0.0
        }));
    assert_eq!(
        released.snapshot.ime_host_requests,
        vec![ImeHostRequest::Disable]
    );
    assert_eq!(
        released.snapshot.cursor_host_requests,
        vec![CursorHostRequest::set_grab_mode(CursorGrabMode::None)]
    );
}
