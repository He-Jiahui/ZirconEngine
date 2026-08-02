use crate::{
    ui::{
        accessibility::{
            UiAccessibilityAction, UiAccessibilityActionRequest, UiAccessibilityActionSource,
        },
        component::UiDragPayloadKind,
        dispatch::{
            UiAnalogInputEvent, UiDragDropInputEvent, UiDragDropInputEventKind,
            UiImeDeleteSurrounding, UiImeInputEvent, UiImeInputEventKind, UiInputEvent,
            UiInputSequence, UiInputTimestamp, UiKeyboardInputEvent, UiKeyboardInputState,
            UiPointerEvent, UiPointerInputEvent, UiPointerSource, UiPreciseScrollDelta,
            UiScrollDeltaUnit, UiTextInputEvent, UiWindowId,
        },
        event_ui::UiNodeId,
        layout::{UiPoint, UiSize},
        surface::{UiPointerButton, UiPointerEventKind},
        window::{
            runtime_event_to_window_input_pump_event, runtime_events_to_window_input_pump_batch,
            UiRuntimeEventAdapterContext, UiRuntimeEventAdapterError, UiWindowEventKind,
            UiWindowInputPumpEvent, UiWindowMetrics, UiWindowPixelPosition, UiWindowPixelSize,
        },
    },
    ZrByteSlice, ZrRuntimeEventV1, ZrRuntimeViewportHandle, ZrRuntimeViewportMetricsV1,
    ZrRuntimeViewportSizeV1, ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
    ZR_RUNTIME_BUTTON_STATE_RELEASED_V1, ZR_RUNTIME_EVENT_KIND_GAMEPAD_CONNECTION_V1,
    ZR_RUNTIME_FILE_DRAG_DROPPED_V1, ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_X_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_EAST_V1, ZR_RUNTIME_GAMEPAD_BUTTON_SOUTH_V1,
    ZR_RUNTIME_IME_STATE_COMMIT_V1, ZR_RUNTIME_IME_STATE_DELETE_SURROUNDING_V1,
    ZR_RUNTIME_IME_STATE_PREEDIT_V1, ZR_RUNTIME_KEY_ACTION_PRESSED_V1,
    ZR_RUNTIME_KEY_ACTION_TEXT_V1, ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1,
    ZR_RUNTIME_LIFECYCLE_STATE_LOW_MEMORY_V1, ZR_RUNTIME_LIFECYCLE_STATE_RESUMED_V1,
    ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1, ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1,
    ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1, ZR_RUNTIME_TOUCH_PHASE_MOVED_V1,
    ZR_RUNTIME_WINDOW_BOOL_TRUE_V1, ZR_RUNTIME_WINDOW_STATUS_OCCLUDED_V1,
    ZR_RUNTIME_WINDOW_STATUS_SCALE_FACTOR_CHANGED_V1, ZR_RUNTIME_WINDOW_STATUS_THEME_CHANGED_V1,
};

fn adapter_context() -> UiRuntimeEventAdapterContext {
    UiRuntimeEventAdapterContext::for_window("runtime.main")
        .with_timestamp(UiInputTimestamp::from_micros(1234))
        .with_sequence(UiInputSequence::new(42))
}

fn viewport() -> ZrRuntimeViewportHandle {
    ZrRuntimeViewportHandle::new(1)
}

fn bytes(value: &str) -> ZrByteSlice {
    ZrByteSlice {
        data: value.as_bytes().as_ptr(),
        len: value.len(),
    }
}

fn adapt(event: ZrRuntimeEventV1) -> UiWindowInputPumpEvent {
    runtime_event_to_window_input_pump_event(&adapter_context(), event).unwrap()
}

#[test]
fn runtime_event_adapter_maps_viewport_and_window_events_to_window_pump_facts() {
    let metrics = ZrRuntimeViewportMetricsV1::new(
        ZrRuntimeViewportSizeV1::new(640, 360),
        2.0,
        ZrRuntimeViewportSizeV1::new(1280, 720),
    );
    let resized = adapt(ZrRuntimeEventV1::viewport_metrics(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
        metrics,
    ));
    let moved = adapt(ZrRuntimeEventV1::window_moved(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
        10,
        20,
    ));
    let occluded = adapt(ZrRuntimeEventV1::window_occluded(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
        true,
    ));
    let scaled = adapt(ZrRuntimeEventV1::window_scale_factor_changed(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
        1.5,
    ));
    let close = adapt(ZrRuntimeEventV1::window_close_requested(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
    ));
    let destroyed = adapt(ZrRuntimeEventV1::window_destroyed(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
    ));

    assert!(matches!(
        resized,
        UiWindowInputPumpEvent::Window(window)
            if window.metadata.window_id == UiWindowId::new("runtime.main")
                && window.metadata.timestamp == UiInputTimestamp::from_micros(1234)
                && window.metadata.sequence == UiInputSequence::new(42)
                && matches!(
                    window.kind,
                    UiWindowEventKind::Resized {
                        metrics: UiWindowMetrics {
                            logical_size: UiSize { width: 640.0, height: 360.0 },
                            physical_size: UiWindowPixelSize { width: 1280, height: 720 },
                            scale_factor: 2.0,
                        }
                    }
                )
    ));
    assert!(matches!(
        moved,
        UiWindowInputPumpEvent::Window(window)
            if matches!(
                window.kind,
                UiWindowEventKind::Moved {
                    position: UiWindowPixelPosition { x: 10, y: 20 }
                }
            )
    ));
    assert!(matches!(
        occluded,
        UiWindowInputPumpEvent::Window(window)
            if matches!(window.kind, UiWindowEventKind::Occluded { occluded: true })
    ));
    assert!(matches!(
        scaled,
        UiWindowInputPumpEvent::Window(window)
            if matches!(
                window.kind,
                UiWindowEventKind::ScaleFactorChanged { scale_factor }
                    if scale_factor == 1.5
            )
    ));
    assert!(matches!(
        close,
        UiWindowInputPumpEvent::Window(window)
            if matches!(window.kind, UiWindowEventKind::CloseRequested)
    ));
    assert!(matches!(
        destroyed,
        UiWindowInputPumpEvent::Window(window)
            if matches!(window.kind, UiWindowEventKind::Destroyed)
    ));
}

#[test]
fn runtime_event_adapter_maps_lifecycle_and_cursor_to_window_events() {
    let active = adapt(ZrRuntimeEventV1::lifecycle(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
        ZR_RUNTIME_LIFECYCLE_STATE_RESUMED_V1,
    ));
    let inactive = adapt(ZrRuntimeEventV1::lifecycle(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
        ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1,
    ));
    let focus_lost = adapt(ZrRuntimeEventV1::lifecycle(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
        ZR_RUNTIME_LIFECYCLE_STATE_LOW_MEMORY_V1,
    ));
    let cursor_moved = adapt(ZrRuntimeEventV1::pointer_moved(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
        24.0,
        36.0,
    ));
    let cursor_entered = adapt(ZrRuntimeEventV1::cursor_entered(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
    ));
    let cursor_left = adapt(ZrRuntimeEventV1::cursor_left(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
    ));

    assert!(matches!(
        active,
        UiWindowInputPumpEvent::Window(window)
            if matches!(
                window.kind,
                UiWindowEventKind::ApplicationActivation { is_active: true }
            )
    ));
    assert!(matches!(
        inactive,
        UiWindowInputPumpEvent::Window(window)
            if matches!(
                window.kind,
                UiWindowEventKind::ApplicationActivation { is_active: false }
            )
    ));
    assert!(matches!(
        focus_lost,
        UiWindowInputPumpEvent::Window(window)
            if matches!(window.kind, UiWindowEventKind::Focused { focused: false })
    ));
    assert!(matches!(
        cursor_moved,
        UiWindowInputPumpEvent::Window(window)
            if matches!(
                window.kind,
                UiWindowEventKind::CursorMoved {
                    position: UiPoint { x: 24.0, y: 36.0 },
                    delta: None,
                }
            )
    ));
    assert!(matches!(
        cursor_entered,
        UiWindowInputPumpEvent::Window(window)
            if matches!(window.kind, UiWindowEventKind::CursorEntered)
    ));
    assert!(matches!(
        cursor_left,
        UiWindowInputPumpEvent::Window(window)
            if matches!(window.kind, UiWindowEventKind::CursorLeft)
    ));
}

#[test]
fn runtime_event_adapter_maps_pointer_touch_and_wheel_to_shared_input_events() {
    let mouse_down = adapt(ZrRuntimeEventV1::mouse_button(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
        ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
        ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
        8.0,
        9.0,
    ));
    let mouse_up = adapt(ZrRuntimeEventV1::mouse_button(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
        ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1,
        ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
        10.0,
        11.0,
    ));
    let wheel = adapt(ZrRuntimeEventV1::mouse_wheel_delta(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
        ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1,
        2.0,
        -3.0,
    ));
    let wheel_at = adapt(ZrRuntimeEventV1::mouse_wheel_delta_at(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
        ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1,
        24.0,
        36.0,
        2.0,
        -3.0,
    ));
    let touch = adapt(ZrRuntimeEventV1::touch(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
        77,
        ZR_RUNTIME_TOUCH_PHASE_MOVED_V1,
        12.0,
        13.0,
    ));

    assert_pointer(
        mouse_down,
        UiPointerEventKind::Down,
        Some(UiPointerButton::Primary),
        UiPoint::new(8.0, 9.0),
        None,
    );
    assert_pointer(
        mouse_up,
        UiPointerEventKind::Up,
        Some(UiPointerButton::Secondary),
        UiPoint::new(10.0, 11.0),
        None,
    );
    assert_pointer(
        wheel,
        UiPointerEventKind::Scroll,
        None,
        UiPoint::default(),
        Some(UiPreciseScrollDelta::pixels(2.0, -3.0)),
    );
    assert_pointer(
        wheel_at,
        UiPointerEventKind::Scroll,
        None,
        UiPoint::new(24.0, 36.0),
        Some(UiPreciseScrollDelta::pixels(2.0, -3.0)),
    );
    let UiWindowInputPumpEvent::Input(UiInputEvent::Pointer(pointer)) = touch else {
        panic!("touch should normalize to pointer input");
    };
    assert_eq!(pointer.metadata.pointer_source, UiPointerSource::Touch);
    assert_eq!(pointer.metadata.pointer_id.unwrap().0, 77);
    assert_eq!(pointer.event.kind, UiPointerEventKind::Move);
    assert_eq!(pointer.event.point, UiPoint::new(12.0, 13.0));
}

#[test]
fn runtime_event_adapter_maps_keyboard_ime_drag_gamepad_and_accessibility_inputs() {
    let keyboard = adapt(ZrRuntimeEventV1::keyboard(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
        ZR_RUNTIME_KEY_ACTION_PRESSED_V1,
        u32::from(b'W'),
        17,
        bytes("w"),
    ));
    let text = adapt(ZrRuntimeEventV1::keyboard(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
        ZR_RUNTIME_KEY_ACTION_TEXT_V1,
        0,
        0,
        bytes("typed"),
    ));
    let ime = adapt(ZrRuntimeEventV1::ime_preedit(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
        bytes("draft"),
        1,
        4,
    ));
    let ime_cancel = adapt(ZrRuntimeEventV1::ime_disabled(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
    ));
    let file = adapt(ZrRuntimeEventV1::file_dropped(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
        bytes("C:/tmp/asset.png"),
    ));
    let gamepad_button = adapt(ZrRuntimeEventV1::gamepad_button(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
        2,
        ZR_RUNTIME_GAMEPAD_BUTTON_SOUTH_V1,
        ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
        1.0,
    ));
    let gamepad_back = adapt(ZrRuntimeEventV1::gamepad_button(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
        2,
        ZR_RUNTIME_GAMEPAD_BUTTON_EAST_V1,
        ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
        0.0,
    ));
    let gamepad_axis = adapt(ZrRuntimeEventV1::gamepad_axis(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
        2,
        ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_X_V1,
        -0.75,
    ));
    let request = UiAccessibilityActionRequest {
        target: UiNodeId::new(9),
        action: UiAccessibilityAction::Activate,
        source: UiAccessibilityActionSource::Pointer,
        value: None,
        numeric_value: None,
        text_selection: None,
        scroll_offset: None,
    };
    let accessibility_bytes = serde_json::to_vec(&request).unwrap();
    let accessibility = adapt(ZrRuntimeEventV1::accessibility_action(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
        ZrByteSlice {
            data: accessibility_bytes.as_ptr(),
            len: accessibility_bytes.len(),
        },
    ));

    assert!(matches!(
        keyboard,
        UiWindowInputPumpEvent::Input(UiInputEvent::Keyboard(UiKeyboardInputEvent {
            state: UiKeyboardInputState::Pressed,
            key_code,
            scan_code: Some(17),
            ref physical_key,
            ref logical_key,
            text: Some(ref typed),
            ..
        })) if key_code == u32::from(b'W')
            && physical_key == "W"
            && logical_key == "W"
            && typed == "w"
    ));
    assert!(matches!(
        text,
        UiWindowInputPumpEvent::Input(UiInputEvent::Text(UiTextInputEvent { ref text, .. }))
            if text == "typed"
    ));
    assert!(matches!(
        ime,
        UiWindowInputPumpEvent::Input(UiInputEvent::Ime(UiImeInputEvent {
            kind: UiImeInputEventKind::Preedit,
            ref text,
            cursor_range: Some(range),
            delete_surrounding: None,
            ..
        })) if text == "draft" && range.start_byte == 1 && range.end_byte == 4
    ));
    assert!(matches!(
        ime_cancel,
        UiWindowInputPumpEvent::Input(UiInputEvent::Ime(UiImeInputEvent {
            kind: UiImeInputEventKind::Cancel,
            ref text,
            cursor_range: None,
            delete_surrounding: None,
            ..
        })) if text.is_empty()
    ));
    assert!(matches!(
        file,
        UiWindowInputPumpEvent::Input(UiInputEvent::DragDrop(UiDragDropInputEvent {
            kind: UiDragDropInputEventKind::Drop,
            payload: Some(ref payload),
            ..
        })) if payload.kind == UiDragPayloadKind::Asset
            && payload.reference == "C:/tmp/asset.png"
    ));
    assert!(matches!(
        gamepad_button,
        UiWindowInputPumpEvent::Input(UiInputEvent::Keyboard(UiKeyboardInputEvent {
            state: UiKeyboardInputState::Pressed,
            ref logical_key,
            ..
        })) if logical_key == "Virtual_Accept"
    ));
    assert!(matches!(
        gamepad_back,
        UiWindowInputPumpEvent::Input(UiInputEvent::Keyboard(UiKeyboardInputEvent {
            state: UiKeyboardInputState::Released,
            ref logical_key,
            ..
        })) if logical_key == "Virtual_Back"
    ));
    assert!(matches!(
        gamepad_axis,
        UiWindowInputPumpEvent::Input(UiInputEvent::Analog(UiAnalogInputEvent {
            ref control,
            value,
            ..
        })) if control == "Gamepad_LeftX" && value == -0.75
    ));
    assert!(matches!(
        accessibility,
        UiWindowInputPumpEvent::Input(UiInputEvent::Accessibility(event))
            if event.request.target == UiNodeId::new(9)
    ));
}

#[test]
fn runtime_event_adapter_preserves_batch_order_and_stops_on_invalid_event() {
    let batch = runtime_events_to_window_input_pump_batch(
        &adapter_context(),
        [
            ZrRuntimeEventV1::pointer_moved(ZIRCON_RUNTIME_ABI_VERSION_V1, viewport(), 1.0, 2.0),
            ZrRuntimeEventV1::keyboard(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                viewport(),
                ZR_RUNTIME_KEY_ACTION_TEXT_V1,
                0,
                0,
                bytes("a"),
            ),
            ZrRuntimeEventV1::window_close_requested(ZIRCON_RUNTIME_ABI_VERSION_V1, viewport()),
        ],
    )
    .unwrap();

    assert_eq!(batch.events.len(), 3);
    assert!(matches!(batch.events[0], UiWindowInputPumpEvent::Window(_)));
    assert!(matches!(
        batch.events[1],
        UiWindowInputPumpEvent::Input(UiInputEvent::Text(_))
    ));
    assert!(matches!(batch.events[2], UiWindowInputPumpEvent::Window(_)));

    let mut bad =
        ZrRuntimeEventV1::window_occluded(ZIRCON_RUNTIME_ABI_VERSION_V1, viewport(), true);
    bad.button = 99;
    let error = runtime_events_to_window_input_pump_batch(&adapter_context(), [bad]).unwrap_err();
    assert_eq!(error, UiRuntimeEventAdapterError::UnknownWindowBool(99));
}

#[test]
fn runtime_event_adapter_rejects_unsupported_or_malformed_events() {
    let mut wrong_abi = ZrRuntimeEventV1::pointer_moved(999, viewport(), 1.0, 2.0);
    wrong_abi.abi_version = 999;
    assert_eq!(
        runtime_event_to_window_input_pump_event(&adapter_context(), wrong_abi).unwrap_err(),
        UiRuntimeEventAdapterError::UnsupportedAbi {
            actual: 999,
            expected: ZIRCON_RUNTIME_ABI_VERSION_V1,
        }
    );

    let connection = ZrRuntimeEventV1::gamepad_connection(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
        1,
        ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
        bytes("Pad"),
    );
    assert_eq!(
        runtime_event_to_window_input_pump_event(&adapter_context(), connection).unwrap_err(),
        UiRuntimeEventAdapterError::NoPumpEquivalent(ZR_RUNTIME_EVENT_KIND_GAMEPAD_CONNECTION_V1)
    );

    let bad_payload = [0xff];
    let bad_keyboard = ZrRuntimeEventV1::keyboard(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport(),
        ZR_RUNTIME_KEY_ACTION_TEXT_V1,
        0,
        0,
        ZrByteSlice {
            data: bad_payload.as_ptr(),
            len: bad_payload.len(),
        },
    );
    assert_eq!(
        runtime_event_to_window_input_pump_event(&adapter_context(), bad_keyboard).unwrap_err(),
        UiRuntimeEventAdapterError::InvalidTextPayload
    );

    let mut theme = ZrRuntimeEventV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        crate::ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1,
        viewport(),
    );
    theme.state = ZR_RUNTIME_WINDOW_STATUS_THEME_CHANGED_V1;
    assert_eq!(
        runtime_event_to_window_input_pump_event(&adapter_context(), theme).unwrap_err(),
        UiRuntimeEventAdapterError::NoPumpEquivalent(crate::ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1)
    );

    let mut bad_window = ZrRuntimeEventV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        crate::ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1,
        viewport(),
    );
    bad_window.state = 99;
    assert_eq!(
        runtime_event_to_window_input_pump_event(&adapter_context(), bad_window).unwrap_err(),
        UiRuntimeEventAdapterError::UnknownWindowStatus(99)
    );
}

#[test]
fn runtime_event_adapter_maps_manual_window_event_shapes() {
    let mut occluded = ZrRuntimeEventV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        crate::ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1,
        viewport(),
    );
    occluded.state = ZR_RUNTIME_WINDOW_STATUS_OCCLUDED_V1;
    occluded.button = ZR_RUNTIME_WINDOW_BOOL_TRUE_V1;
    assert!(matches!(
        adapt(occluded),
        UiWindowInputPumpEvent::Window(window)
            if matches!(window.kind, UiWindowEventKind::Occluded { occluded: true })
    ));

    let mut scaled = ZrRuntimeEventV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        crate::ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1,
        viewport(),
    );
    scaled.state = ZR_RUNTIME_WINDOW_STATUS_SCALE_FACTOR_CHANGED_V1;
    scaled.delta = 2.25;
    assert!(matches!(
        adapt(scaled),
        UiWindowInputPumpEvent::Window(window)
            if matches!(
                window.kind,
                UiWindowEventKind::ScaleFactorChanged { scale_factor }
                    if scale_factor == 2.25
            )
    ));

    let mut file_dropped = ZrRuntimeEventV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        crate::ZR_RUNTIME_EVENT_KIND_FILE_DRAG_DROP_V1,
        viewport(),
    );
    file_dropped.state = ZR_RUNTIME_FILE_DRAG_DROPPED_V1;
    file_dropped.payload = bytes("res://asset.ui");
    assert!(matches!(
        adapt(file_dropped),
        UiWindowInputPumpEvent::Input(UiInputEvent::DragDrop(UiDragDropInputEvent {
            kind: UiDragDropInputEventKind::Drop,
            payload: Some(ref payload),
            ..
        })) if payload.reference == "res://asset.ui"
    ));

    let commit = ZrRuntimeEventV1 {
        state: ZR_RUNTIME_IME_STATE_COMMIT_V1,
        payload: bytes("done"),
        ..ZrRuntimeEventV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            crate::ZR_RUNTIME_EVENT_KIND_IME_V1,
            viewport(),
        )
    };
    assert!(matches!(
        adapt(commit),
        UiWindowInputPumpEvent::Input(UiInputEvent::Ime(UiImeInputEvent {
            kind: UiImeInputEventKind::Commit,
            ref text,
            delete_surrounding: None,
            ..
        })) if text == "done"
    ));

    let preedit = ZrRuntimeEventV1 {
        state: ZR_RUNTIME_IME_STATE_PREEDIT_V1,
        payload: bytes("preedit"),
        key_code: 2,
        scan_code: 5,
        ..ZrRuntimeEventV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            crate::ZR_RUNTIME_EVENT_KIND_IME_V1,
            viewport(),
        )
    };
    assert!(matches!(
        adapt(preedit),
        UiWindowInputPumpEvent::Input(UiInputEvent::Ime(UiImeInputEvent {
            kind: UiImeInputEventKind::Preedit,
            cursor_range: Some(range),
            delete_surrounding: None,
            ..
        })) if range.start_byte == 2 && range.end_byte == 5
    ));

    let delete_surrounding = ZrRuntimeEventV1 {
        state: ZR_RUNTIME_IME_STATE_DELETE_SURROUNDING_V1,
        key_code: 3,
        scan_code: 1,
        ..ZrRuntimeEventV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            crate::ZR_RUNTIME_EVENT_KIND_IME_V1,
            viewport(),
        )
    };
    assert!(matches!(
        adapt(delete_surrounding),
        UiWindowInputPumpEvent::Input(UiInputEvent::Ime(UiImeInputEvent {
            kind: UiImeInputEventKind::DeleteSurrounding,
            ref text,
            cursor_range: None,
            delete_surrounding: Some(delete),
            ..
        })) if text.is_empty() && delete == UiImeDeleteSurrounding::new(3, 1)
    ));
}

fn assert_pointer(
    event: UiWindowInputPumpEvent,
    kind: UiPointerEventKind,
    button: Option<UiPointerButton>,
    point: UiPoint,
    precise_scroll: Option<UiPreciseScrollDelta>,
) {
    let UiWindowInputPumpEvent::Input(UiInputEvent::Pointer(UiPointerInputEvent {
        metadata,
        event:
            UiPointerEvent {
                kind: actual_kind,
                button: actual_button,
                point: actual_point,
                ..
            },
        precise_scroll: actual_precise_scroll,
    })) = event
    else {
        panic!("expected pointer input");
    };

    assert_eq!(metadata.window_id, Some(UiWindowId::new("runtime.main")));
    assert_eq!(actual_kind, kind);
    assert_eq!(actual_button, button);
    assert_eq!(actual_point, point);
    assert_eq!(actual_precise_scroll, precise_scroll);
    if let Some(delta) = actual_precise_scroll {
        assert_eq!(delta.unit, UiScrollDeltaUnit::Pixels);
    }
}
