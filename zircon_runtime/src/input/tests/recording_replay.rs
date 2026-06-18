use crate::core::framework::input::InputManager;
use crate::input::{
    DefaultInputManager, InputButton, InputEvent, InputRecording, InputRecordingFrame,
};

#[test]
fn input_recording_captures_drainable_event_records_by_frame() {
    let input = DefaultInputManager::default();
    input.submit_event(InputEvent::CursorMoved { x: 4.0, y: 9.0 });
    input.submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));

    let frame = InputRecordingFrame::capture_from_manager(17, &input);
    assert_eq!(frame.frame_index(), 17);
    assert_eq!(frame.event_count(), 2);
    assert!(input.drain_event_records().is_empty());

    let mut recording = InputRecording::new();
    recording.push_frame(frame.clone());

    assert_eq!(recording.frame_count(), 1);
    assert_eq!(recording.event_count(), 2);
    assert_eq!(recording.frames(), &[frame]);
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
