use super::*;

#[test]
fn pointer_event_streams_are_frame_bounded_at_common_polling_rates() {
    for events_per_second in [125, 500, 1_000] {
        let input = DefaultInputManager::default();
        for _second in 0..600 {
            input.begin_frame();
            for sample in 0..events_per_second {
                input.submit_event(InputEvent::CursorMoved {
                    x: sample as f32,
                    y: -(sample as f32),
                });
            }
            assert_eq!(input.event_queue_status().retained_events, 1);
            assert_eq!(
                input.event_queue_status().coalesced_events,
                (events_per_second - 1) as u64
            );
            assert_eq!(
                input.drain_events(),
                vec![InputEvent::CursorMoved {
                    x: (events_per_second - 1) as f32,
                    y: -((events_per_second - 1) as f32),
                }]
            );

            input.begin_frame();
            for _ in 0..events_per_second {
                input.submit_event(InputEvent::MouseMotion {
                    delta_x: 0.25,
                    delta_y: -0.5,
                });
            }
            assert_eq!(input.event_queue_status().retained_events, 1);
            assert_eq!(
                input.event_queue_status().coalesced_events,
                (events_per_second - 1) as u64
            );
            assert_eq!(
                input.drain_events(),
                vec![InputEvent::MouseMotion {
                    delta_x: events_per_second as f32 * 0.25,
                    delta_y: events_per_second as f32 * -0.5,
                }]
            );
        }
    }
}

#[test]
fn pointer_coalescing_preserves_button_and_touch_edges_in_order() {
    let input = DefaultInputManager::default();
    input.begin_frame();
    input.submit_event(InputEvent::CursorMoved { x: 1.0, y: 2.0 });
    input.submit_event(InputEvent::CursorMoved { x: 3.0, y: 4.0 });
    input.submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));
    input.submit_event(InputEvent::CursorMoved { x: 5.0, y: 6.0 });
    input.submit_event(InputEvent::CursorMoved { x: 7.0, y: 8.0 });
    input.submit_event(InputEvent::ButtonReleased(InputButton::MouseLeft));
    input.submit_event(InputEvent::Touch {
        id: 9,
        phase: TouchPhase::Started,
        x: 10.0,
        y: 11.0,
    });
    input.submit_event(InputEvent::Touch {
        id: 9,
        phase: TouchPhase::Moved,
        x: 12.0,
        y: 13.0,
    });
    input.submit_event(InputEvent::Touch {
        id: 9,
        phase: TouchPhase::Ended,
        x: 14.0,
        y: 15.0,
    });

    assert_eq!(
        input.drain_events(),
        vec![
            InputEvent::CursorMoved { x: 3.0, y: 4.0 },
            InputEvent::ButtonPressed(InputButton::MouseLeft),
            InputEvent::CursorMoved { x: 7.0, y: 8.0 },
            InputEvent::ButtonReleased(InputButton::MouseLeft),
            InputEvent::Touch {
                id: 9,
                phase: TouchPhase::Started,
                x: 10.0,
                y: 11.0,
            },
            InputEvent::Touch {
                id: 9,
                phase: TouchPhase::Moved,
                x: 12.0,
                y: 13.0,
            },
            InputEvent::Touch {
                id: 9,
                phase: TouchPhase::Ended,
                x: 14.0,
                y: 15.0,
            },
        ]
    );
}

#[test]
fn focus_loss_is_a_barrier_to_pointer_event_coalescing() {
    let input = DefaultInputManager::default();
    input.begin_frame();
    input.submit_event(InputEvent::CursorMoved { x: 1.0, y: 2.0 });
    input.submit_event(InputEvent::CursorMoved { x: 3.0, y: 4.0 });
    input.submit_event(InputEvent::FocusLost);
    input.submit_event(InputEvent::CursorMoved { x: 5.0, y: 6.0 });
    input.submit_event(InputEvent::CursorMoved { x: 7.0, y: 8.0 });

    assert_eq!(
        input.drain_events(),
        vec![
            InputEvent::CursorMoved { x: 3.0, y: 4.0 },
            InputEvent::FocusLost,
            InputEvent::CursorMoved { x: 7.0, y: 8.0 },
        ]
    );
}

#[test]
fn begin_frame_discards_undrained_transient_events() {
    let input = DefaultInputManager::default();
    input.submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));

    input.begin_frame();

    assert!(input.drain_events().is_empty());
    assert!(input
        .frame_snapshot()
        .buttons
        .pressed(&InputButton::MouseLeft));
}

#[test]
fn recording_is_opt_in_bounded_and_reports_discarded_raw_records() {
    let input = DefaultInputManager::default();
    for sample in 0..1_000 {
        input.submit_event(InputEvent::CursorMoved {
            x: sample as f32,
            y: 0.0,
        });
    }

    assert!(input.drain_event_records().is_empty());
    assert_eq!(input.event_recording_status().retained_records, 0);
    assert!(!input.event_recording_status().enabled);

    input.set_event_recording_config(crate::input::InputEventRecordingConfig::enabled(3));
    for sample in 0..5 {
        input.submit_event(InputEvent::CursorMoved {
            x: sample as f32,
            y: 1.0,
        });
    }

    let status = input.event_recording_status();
    assert!(status.enabled);
    assert_eq!(status.capacity, 3);
    assert_eq!(status.retained_records, 3);
    assert_eq!(status.discarded_records, 2);
    assert_eq!(
        input.drain_events(),
        vec![InputEvent::CursorMoved { x: 4.0, y: 1.0 }]
    );

    let records = input.drain_event_records();
    assert_eq!(
        records
            .iter()
            .map(|record| record.sequence)
            .collect::<Vec<_>>(),
        vec![3, 4, 5]
    );
    assert_eq!(
        records
            .iter()
            .map(|record| record.event.clone())
            .collect::<Vec<_>>(),
        vec![
            InputEvent::CursorMoved { x: 2.0, y: 1.0 },
            InputEvent::CursorMoved { x: 3.0, y: 1.0 },
            InputEvent::CursorMoved { x: 4.0, y: 1.0 },
        ]
    );
    let drained_status = input.event_recording_status();
    assert_eq!(drained_status.retained_records, 0);
    assert_eq!(drained_status.discarded_records, 2);

    input.set_event_recording_config(crate::input::InputEventRecordingConfig::disabled());
    input.submit_event(InputEvent::ButtonPressed(InputButton::MouseRight));
    let disabled_status = input.event_recording_status();
    assert!(!disabled_status.enabled);
    assert_eq!(disabled_status.retained_records, 0);
    assert_eq!(disabled_status.discarded_records, 0);
    assert!(input.drain_event_records().is_empty());
}
