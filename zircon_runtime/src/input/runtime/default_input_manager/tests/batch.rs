use std::mem::size_of;
use std::num::NonZeroU64;

use crate::core::framework::input::{
    GamepadConnectionInfo, GamepadId, GamepadRumbleRequest, InputBatchLimits, InputBatchRejection,
    InputButton, InputEvent, InputEventRecordingConfig, InputManager,
};
use crate::input::DefaultInputManager;

fn limits() -> InputBatchLimits {
    InputBatchLimits::new(32, 64 * 1024)
}

fn connection(generation: u64) -> InputEvent {
    InputEvent::GamepadConnection(GamepadConnectionInfo {
        gamepad: GamepadId(7),
        generation: NonZeroU64::new(generation).unwrap(),
        connected: true,
        name: None,
        vendor_id: None,
        product_id: None,
    })
}

#[test]
fn runtime220_atomic_batch_rejects_invalid_tail_before_physical_or_recording_changes() {
    let manager = DefaultInputManager::default();
    manager.set_event_recording_config(InputEventRecordingConfig::enabled(32));
    let batch = vec![
        InputEvent::ButtonPressed(InputButton::MouseLeft),
        InputEvent::CursorMoved {
            x: f32::NAN,
            y: 1.0,
        },
    ];
    assert_eq!(
        manager.submit_batch_with_admission(batch, limits()),
        Err(InputBatchRejection::NonFinite { event_index: 1 })
    );
    assert!(!manager.button_pressed(&InputButton::MouseLeft));
    assert_eq!(manager.snapshot().cursor_position, [0.0, 0.0]);
    assert!(manager.drain_events().is_empty());
    assert!(manager.drain_event_records().is_empty());
}

#[test]
fn runtime220_atomic_batch_counts_allocated_string_and_vec_capacity() {
    let manager = DefaultInputManager::default();
    let mut text = String::with_capacity(4096);
    text.push('x');
    let text_capacity = text.capacity();
    let mut events = Vec::with_capacity(8);
    events.push(InputEvent::KeyboardInput {
        key_code: 17,
        logical_key: None,
        text: Some(text),
        pressed: true,
        repeat: false,
    });
    let expected = events.capacity() * size_of::<InputEvent>() + text_capacity;
    assert_eq!(
        manager.submit_batch_with_admission(events, InputBatchLimits::new(1, expected - 1),),
        Err(InputBatchRejection::OwnedByteCapacity {
            requested: expected,
            remaining: expected - 1
        })
    );
    assert!(!manager.button_pressed(&InputButton::KeyCode(17)));
    assert!(manager.drain_events().is_empty());

    let mut slots = Vec::with_capacity(8);
    slots.push(InputEvent::CursorEntered);
    assert!(matches!(
        manager
            .submit_batch_with_admission(slots, InputBatchLimits::new(1, size_of::<InputEvent>()),),
        Err(InputBatchRejection::OwnedByteCapacity { .. })
    ));
    assert!(manager.drain_events().is_empty());
}

#[test]
fn runtime220_atomic_batch_qualifies_lifecycle_and_rumble_in_order_without_partial_connect() {
    let manager = DefaultInputManager::default();
    let stop = InputEvent::GamepadRumbleRequest(GamepadRumbleRequest::stop(GamepadId(7)));
    assert!(matches!(
        manager.submit_batch_with_admission(vec![stop.clone(), connection(1)], limits(),),
        Err(InputBatchRejection::Gamepad { event_index: 0, .. })
    ));
    assert!(manager.frame_snapshot().connected_gamepads.is_empty());
    assert!(manager.drain_events().is_empty());

    manager
        .submit_batch_with_admission(vec![connection(1), stop], limits())
        .unwrap();
    let requests = manager.drain_gamepad_rumble_requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].generation(), NonZeroU64::new(1).unwrap());
    manager.drain_events();
    assert!(matches!(
        manager.submit_batch_with_admission(vec![connection(3), connection(2)], limits(),),
        Err(InputBatchRejection::Gamepad { event_index: 1, .. })
    ));
    assert!(manager.drain_events().is_empty());
    manager
        .submit_batch_with_admission(vec![connection(2)], limits())
        .unwrap();
}

#[test]
fn runtime220_atomic_batch_rejects_finite_accumulator_overflow_and_keeps_focus_reset_real() {
    let manager = DefaultInputManager::default();
    assert_eq!(
        manager.submit_batch_with_admission(
            vec![
                InputEvent::MouseMotion {
                    delta_x: f32::MAX,
                    delta_y: 0.0
                },
                InputEvent::MouseMotion {
                    delta_x: f32::MAX,
                    delta_y: 0.0
                },
            ],
            limits()
        ),
        Err(InputBatchRejection::NonFinite { event_index: 1 })
    );
    assert_eq!(
        manager.frame_snapshot().mouse_motion_accumulator,
        [0.0, 0.0]
    );
    assert!(manager.drain_events().is_empty());

    manager.submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));
    manager.submit_event(InputEvent::Touch {
        id: 1,
        phase: crate::input::TouchPhase::Started,
        x: 1.0,
        y: 2.0,
    });
    manager
        .submit_batch_with_admission(vec![InputEvent::FocusLost], limits())
        .unwrap();
    assert!(!manager.button_pressed(&InputButton::MouseLeft));
    assert!(manager.frame_snapshot().active_touches.is_empty());
}

#[test]
fn runtime220_atomic_touch_subbatch_is_rejected_as_a_whole() {
    let manager = DefaultInputManager::default();
    assert!(matches!(
        manager.submit_batch_with_admission(
            vec![
                InputEvent::CursorMoved { x: 3.0, y: 4.0 },
                InputEvent::Touch {
                    id: 4,
                    phase: crate::input::TouchPhase::Started,
                    x: 3.0,
                    y: 4.0
                },
            ],
            InputBatchLimits::new(1, 64 * 1024)
        ),
        Err(InputBatchRejection::EventCapacity {
            requested: 2,
            remaining: 1
        })
    ));
    assert_eq!(manager.snapshot().cursor_position, [0.0, 0.0]);
    assert!(manager.frame_snapshot().active_touches.is_empty());
    assert!(manager.drain_events().is_empty());
}
