use crate::ui::{
    dispatch::{
        UiImeInputEvent, UiImeInputEventKind, UiInputEvent, UiInputEventMetadata, UiInputSequence,
        UiInputTimestamp, UiTextByteRange, UiWindowId,
    },
    layout::{UiPoint, UiSize},
    window::{
        UiWindowEvent, UiWindowEventKind, UiWindowEventMetadata, UiWindowInputPumpBatch,
        UiWindowInputPumpEvent, UiWindowMetrics, UiWindowPixelPosition, UiWindowPixelSize,
        UiWindowRedrawReason,
    },
};

fn sample_window_metadata() -> UiWindowEventMetadata {
    UiWindowEventMetadata::for_window(
        UiWindowId::new("editor.main"),
        UiInputTimestamp::from_micros(123),
        UiInputSequence::new(7),
    )
}

fn sample_input_metadata() -> UiInputEventMetadata {
    let mut metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(124), UiInputSequence::new(8));
    metadata.window_id = Some(UiWindowId::new("editor.main"));
    metadata
}

fn round_trip<T>(value: &T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    serde_json::from_str(&serde_json::to_string(value).unwrap()).unwrap()
}

#[test]
fn ui_window_events_carry_cursor_focus_scale_redraw_and_close_contracts() {
    let metadata = sample_window_metadata();
    let cursor_moved = UiWindowEvent::new(
        metadata.clone(),
        UiWindowEventKind::CursorMoved {
            position: UiPoint::new(32.0, 48.0),
            delta: Some(UiPoint::new(4.0, -2.0)),
        },
    );
    let cursor_left = UiWindowEvent::new(metadata.clone(), UiWindowEventKind::CursorLeft);
    let scale_factor = UiWindowEvent::new(
        metadata.clone(),
        UiWindowEventKind::ScaleFactorChanged { scale_factor: 2.0 },
    );
    let resized = UiWindowEvent::new(
        metadata.clone(),
        UiWindowEventKind::Resized {
            metrics: UiWindowMetrics::new(
                UiSize::new(640.0, 360.0),
                UiWindowPixelSize::new(1280, 720),
                2.0,
            ),
        },
    );
    let moved = UiWindowEvent::new(
        metadata.clone(),
        UiWindowEventKind::Moved {
            position: UiWindowPixelPosition::new(12, 24),
        },
    );
    let focused = UiWindowEvent::new(
        metadata.clone(),
        UiWindowEventKind::Focused { focused: true },
    );
    let redraw = UiWindowEvent::new(
        metadata.clone(),
        UiWindowEventKind::RequestRedraw {
            reason: UiWindowRedrawReason::Input,
        },
    );
    let close = UiWindowEvent::new(metadata, UiWindowEventKind::CloseRequested);

    assert_eq!(cursor_moved.window_id().unwrap().0, "editor.main");
    assert_eq!(cursor_moved.impact().input_state_dirty, true);
    assert_eq!(cursor_left.impact().clears_hover, true);
    assert_eq!(cursor_left.impact().requests_redraw, true);
    assert_eq!(scale_factor.impact().layout_metrics_dirty, true);
    assert_eq!(scale_factor.impact().input_state_dirty, false);
    assert_eq!(scale_factor.impact().requests_redraw, false);
    assert_eq!(scale_factor.impact().clears_hover, false);
    assert_eq!(resized.impact().layout_metrics_dirty, true);
    assert_eq!(resized.impact().requests_redraw, true);
    assert_eq!(moved.impact().input_state_dirty, false);
    assert_eq!(focused.impact().input_state_dirty, true);
    assert_eq!(redraw.impact().requests_redraw, true);
    assert_eq!(close.impact().close_requested, true);
    assert_eq!(round_trip(&cursor_moved), cursor_moved);
}

#[test]
fn ui_window_input_pump_wraps_window_and_shared_input_events_with_redraw_coalescing() {
    let metadata = sample_window_metadata();
    let redraw = UiWindowInputPumpEvent::Window(UiWindowEvent::new(
        metadata.clone(),
        UiWindowEventKind::RequestRedraw {
            reason: UiWindowRedrawReason::Input,
        },
    ));
    let second_redraw = UiWindowInputPumpEvent::Window(UiWindowEvent::new(
        metadata.clone(),
        UiWindowEventKind::RequestRedraw {
            reason: UiWindowRedrawReason::Animation,
        },
    ));
    let cursor_left =
        UiWindowInputPumpEvent::Window(UiWindowEvent::new(metadata, UiWindowEventKind::CursorLeft));
    let ime = UiWindowInputPumpEvent::Input(UiInputEvent::Ime(UiImeInputEvent {
        metadata: sample_input_metadata(),
        kind: UiImeInputEventKind::Preedit,
        text: "draft".to_string(),
        cursor_range: Some(UiTextByteRange::new(0, 5)),
    }));

    let mut batch = UiWindowInputPumpBatch::default();
    batch.push_coalesced(redraw.clone());
    batch.push_coalesced(second_redraw);
    batch.push_coalesced(cursor_left.clone());
    batch.push_coalesced(ime.clone());

    assert_eq!(batch.events.len(), 3);
    assert_eq!(batch.events[0], redraw);
    assert_eq!(batch.events[1], cursor_left);
    assert_eq!(batch.events[2], ime);
    assert!(matches!(batch.events[2], UiWindowInputPumpEvent::Input(_)));
    assert_eq!(round_trip(&batch), batch);
}
