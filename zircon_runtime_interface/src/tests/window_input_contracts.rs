use crate::ui::{
    accessibility::{UiAccessibilityAction, UiAccessibilityActionRequest},
    component::{UiDragPayload, UiDragPayloadKind},
    dispatch::{
        UiDeviceId, UiDragDropInputEventKind, UiDragSessionId, UiImeInputEvent,
        UiImeInputEventKind, UiInputEvent, UiInputEventMetadata, UiInputModifiers, UiInputSequence,
        UiInputTimestamp, UiKeyboardInputState, UiPointerEvent, UiPointerId, UiPointerSource,
        UiPopupInputEventKind, UiPreciseScrollDelta, UiScrollDeltaUnit, UiTextByteRange,
        UiTooltipTimerInputEventKind, UiUserId, UiWindowId,
    },
    event_ui::UiNodeId,
    layout::{UiPoint, UiSize},
    surface::{UiNavigationEventKind, UiPointerButton, UiPointerEventKind},
    window::{
        UiWindowAction, UiWindowActivation, UiWindowEvent, UiWindowEventImpact, UiWindowEventKind,
        UiWindowEventMetadata, UiWindowInputContext, UiWindowInputPumpBatch,
        UiWindowInputPumpEvent, UiWindowMetrics, UiWindowPixelPosition, UiWindowPixelSize,
        UiWindowPlatformInputEvent, UiWindowRedrawReason, UiWindowTouchPhase,
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
    let size_changed = UiWindowEvent::size_changed(
        metadata.clone(),
        UiWindowMetrics::new(
            UiSize::new(800.0, 450.0),
            UiWindowPixelSize::new(1600, 900),
            2.0,
        ),
    );
    let moved = UiWindowEvent::new(
        metadata.clone(),
        UiWindowEventKind::Moved {
            position: UiWindowPixelPosition::new(12, 24),
        },
    );
    let moved_window =
        UiWindowEvent::moved_window(metadata.clone(), UiWindowPixelPosition::new(18, 36));
    let focused = UiWindowEvent::new(
        metadata.clone(),
        UiWindowEventKind::Focused { focused: true },
    );
    let window_focused = UiWindowEvent::window_focused(metadata.clone(), true);
    let window_unfocused = UiWindowEvent::window_focused(metadata.clone(), false);
    let activated =
        UiWindowEvent::window_activation_changed(metadata.clone(), UiWindowActivation::Activate);
    let activated_by_mouse = UiWindowEvent::window_activation_changed(
        metadata.clone(),
        UiWindowActivation::ActivateByMouse,
    );
    let deactivated =
        UiWindowEvent::window_activation_changed(metadata.clone(), UiWindowActivation::Deactivate);
    let app_active = UiWindowEvent::application_activation_changed(metadata.clone(), true);
    let app_inactive = UiWindowEvent::application_activation_changed(metadata.clone(), false);
    let redraw = UiWindowEvent::new(
        metadata.clone(),
        UiWindowEventKind::RequestRedraw {
            reason: UiWindowRedrawReason::Input,
        },
    );
    let os_paint = UiWindowEvent::os_paint(metadata.clone());
    let resizing_window = UiWindowEvent::resizing_window(metadata.clone());
    let non_client_action =
        UiWindowEvent::window_action(metadata.clone(), UiWindowAction::ClickedNonClientArea);
    let maximize_action = UiWindowEvent::window_action(metadata.clone(), UiWindowAction::Maximize);
    let restore_action = UiWindowEvent::window_action(metadata.clone(), UiWindowAction::Restore);
    let window_menu_action =
        UiWindowEvent::window_action(metadata.clone(), UiWindowAction::WindowMenu);
    let close = UiWindowEvent::new(metadata.clone(), UiWindowEventKind::CloseRequested);
    let window_close = UiWindowEvent::window_close(metadata);

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
    assert!(matches!(
        size_changed.kind,
        UiWindowEventKind::Resized { metrics }
            if metrics.logical_size == UiSize::new(800.0, 450.0)
                && metrics.physical_size == UiWindowPixelSize::new(1600, 900)
                && metrics.scale_factor == 2.0
    ));
    assert_eq!(size_changed.impact().layout_metrics_dirty, true);
    assert_eq!(size_changed.impact().requests_redraw, true);
    assert_eq!(moved.impact().input_state_dirty, false);
    assert!(matches!(
        moved_window.kind,
        UiWindowEventKind::Moved { position }
            if position == UiWindowPixelPosition::new(18, 36)
    ));
    assert_eq!(moved_window.impact(), moved.impact());
    assert_eq!(focused.impact().input_state_dirty, true);
    assert!(matches!(
        window_focused.kind,
        UiWindowEventKind::Focused { focused: true }
    ));
    assert!(matches!(
        window_unfocused.kind,
        UiWindowEventKind::Focused { focused: false }
    ));
    assert_eq!(window_focused.impact(), focused.impact());
    assert_eq!(window_unfocused.impact(), focused.impact());
    assert!(UiWindowActivation::Activate.is_active());
    assert!(UiWindowActivation::ActivateByMouse.is_active());
    assert!(!UiWindowActivation::Deactivate.is_active());
    assert!(matches!(
        activated.kind,
        UiWindowEventKind::Focused { focused: true }
    ));
    assert!(matches!(
        activated_by_mouse.kind,
        UiWindowEventKind::Focused { focused: true }
    ));
    assert!(matches!(
        deactivated.kind,
        UiWindowEventKind::Focused { focused: false }
    ));
    assert!(matches!(
        app_active.kind,
        UiWindowEventKind::Focused { focused: true }
    ));
    assert!(matches!(
        app_inactive.kind,
        UiWindowEventKind::Focused { focused: false }
    ));
    assert_eq!(activated.impact(), focused.impact());
    assert_eq!(activated_by_mouse.impact(), focused.impact());
    assert_eq!(deactivated.impact(), focused.impact());
    assert_eq!(app_active.impact(), focused.impact());
    assert_eq!(app_inactive.impact(), focused.impact());
    assert_eq!(redraw.impact().requests_redraw, true);
    assert!(matches!(
        os_paint.kind,
        UiWindowEventKind::RequestRedraw {
            reason: UiWindowRedrawReason::Paint
        }
    ));
    assert!(matches!(
        resizing_window.kind,
        UiWindowEventKind::RequestRedraw {
            reason: UiWindowRedrawReason::Paint
        }
    ));
    assert_eq!(os_paint.impact().requests_redraw, true);
    assert_eq!(resizing_window.impact(), os_paint.impact());
    assert!(matches!(
        non_client_action.kind,
        UiWindowEventKind::WindowAction {
            action: UiWindowAction::ClickedNonClientArea
        }
    ));
    assert!(matches!(
        maximize_action.kind,
        UiWindowEventKind::WindowAction {
            action: UiWindowAction::Maximize
        }
    ));
    assert!(matches!(
        restore_action.kind,
        UiWindowEventKind::WindowAction {
            action: UiWindowAction::Restore
        }
    ));
    assert!(matches!(
        window_menu_action.kind,
        UiWindowEventKind::WindowAction {
            action: UiWindowAction::WindowMenu
        }
    ));
    assert_eq!(non_client_action.impact(), UiWindowEventImpact::clean());
    assert_eq!(maximize_action.impact(), UiWindowEventImpact::clean());
    assert_eq!(restore_action.impact(), UiWindowEventImpact::clean());
    assert_eq!(window_menu_action.impact(), UiWindowEventImpact::clean());
    assert_eq!(close.impact().close_requested, true);
    assert_eq!(window_close.impact(), close.impact());
    assert!(matches!(
        window_close.kind,
        UiWindowEventKind::CloseRequested
    ));
    assert_eq!(round_trip(&cursor_moved), cursor_moved);
    assert_eq!(round_trip(&size_changed), size_changed);
    assert_eq!(round_trip(&activated), activated);
    assert_eq!(round_trip(&deactivated), deactivated);
    assert_eq!(
        round_trip(&UiWindowActivation::ActivateByMouse),
        UiWindowActivation::ActivateByMouse
    );
    assert_eq!(round_trip(&non_client_action), non_client_action);
    assert_eq!(round_trip(&window_menu_action), window_menu_action);
    assert_eq!(
        round_trip(&UiWindowAction::ClickedNonClientArea),
        UiWindowAction::ClickedNonClientArea
    );
    assert_eq!(round_trip(&os_paint), os_paint);
    assert_eq!(round_trip(&window_close), window_close);
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

#[test]
fn ui_window_input_pump_accepts_platform_input_events_through_normalization() {
    let metadata = sample_window_metadata().synthetic(true);
    let context = UiWindowInputContext::from_window_metadata(&metadata)
        .with_device_id(UiDeviceId::new(9))
        .with_pointer_id(UiPointerId::new(11));

    let mut batch = UiWindowInputPumpBatch::default();
    batch.push_platform_input(UiWindowPlatformInputEvent::ime_with_cursor_range(
        context.clone(),
        UiImeInputEventKind::Preedit,
        "draft",
        Some(UiTextByteRange::new(1, 4)),
    ));
    batch.push_platform_input(UiWindowPlatformInputEvent::touch(
        context,
        UiWindowTouchPhase::Moved,
        UiPointerId::new(77),
        UiPoint::new(14.0, 28.0),
    ));

    assert_eq!(batch.events.len(), 2);
    assert!(matches!(
        &batch.events[0],
        UiWindowInputPumpEvent::Input(UiInputEvent::Ime(ime))
            if ime.kind == UiImeInputEventKind::Preedit
                && ime.text == "draft"
                && ime.cursor_range == Some(UiTextByteRange::new(1, 4))
                && ime.metadata.window_id == Some(metadata.window_id.clone())
                && ime.metadata.device_id == Some(UiDeviceId::new(9))
                && ime.metadata.synthetic
    ));
    assert!(matches!(
        &batch.events[1],
        UiWindowInputPumpEvent::Input(UiInputEvent::Pointer(pointer))
            if pointer.metadata.window_id == Some(metadata.window_id.clone())
                && pointer.metadata.device_id == Some(UiDeviceId::new(9))
                && pointer.metadata.pointer_id == Some(UiPointerId::new(77))
                && pointer.metadata.pointer_source == UiPointerSource::Touch
                && pointer.event.kind == UiPointerEventKind::Move
                && pointer.event.point == UiPoint::new(14.0, 28.0)
                && pointer.metadata.synthetic
    ));
    assert_eq!(round_trip(&batch), batch);
}

#[test]
fn ui_window_platform_input_normalizes_generic_application_style_events() {
    let metadata = sample_window_metadata().synthetic(true);
    let context = UiWindowInputContext::from_window_metadata(&metadata)
        .with_user_id(UiUserId::new(3))
        .with_device_id(UiDeviceId::new(9))
        .with_pointer_id(UiPointerId::new(11))
        .with_pointer_source(UiPointerSource::Pen)
        .with_modifiers(UiInputModifiers {
            shift: true,
            control: false,
            alt: true,
            super_key: false,
            caps_lock: true,
            num_lock: false,
        });

    let pointer = UiWindowPlatformInputEvent::pointer(
        context.clone(),
        UiPointerEvent::new(UiPointerEventKind::Scroll, UiPoint::new(18.0, 24.0))
            .with_button(UiPointerButton::Middle)
            .with_scroll_delta(-42.0),
        Some(UiPreciseScrollDelta {
            x: 0.0,
            y: -42.0,
            unit: UiScrollDeltaUnit::Pixels,
        }),
    )
    .normalize();
    let mouse_move =
        UiWindowPlatformInputEvent::mouse_move(context.clone(), UiPoint::new(8.0, 9.0)).normalize();
    let cursor_entered =
        UiWindowPlatformInputEvent::cursor_entered(context.clone(), UiPoint::new(9.0, 10.0))
            .normalize();
    let cursor_left =
        UiWindowPlatformInputEvent::cursor_left(context.clone(), UiPoint::new(11.0, 13.0))
            .normalize();
    let mouse_capture_lost =
        UiWindowPlatformInputEvent::mouse_capture_lost(context.clone(), UiPoint::new(17.0, 19.0))
            .normalize();
    let mouse_wheel =
        UiWindowPlatformInputEvent::mouse_wheel(context.clone(), UiPoint::new(12.0, 18.0), 3.0)
            .normalize();
    let mouse_pixel_wheel = UiWindowPlatformInputEvent::mouse_wheel_delta(
        context.clone(),
        UiPoint::new(15.0, 21.0),
        UiPreciseScrollDelta::pixels(1.25, -2.5),
    )
    .normalize();
    let raw_mouse_motion =
        UiWindowPlatformInputEvent::raw_mouse_motion(context.clone(), -3.5, 2.25).normalize();
    let mouse_down = UiWindowPlatformInputEvent::mouse_button_down(
        context.clone(),
        UiPointerButton::Primary,
        UiPoint::new(10.0, 12.0),
    )
    .normalize();
    let mouse_up = UiWindowPlatformInputEvent::mouse_button_up(
        context.clone(),
        UiPointerButton::Primary,
        UiPoint::new(10.0, 12.0),
    )
    .normalize();
    let mouse_double_click = UiWindowPlatformInputEvent::mouse_double_click(
        context.clone(),
        UiPointerButton::Primary,
        UiPoint::new(14.0, 16.0),
    )
    .normalize();
    let key_down =
        UiWindowPlatformInputEvent::key_down(context.clone(), 65, Some(30), "KeyA", "A", false)
            .normalize();
    let key_repeat =
        UiWindowPlatformInputEvent::key_down(context.clone(), 65, Some(30), "KeyA", "A", true)
            .normalize();
    let key_up =
        UiWindowPlatformInputEvent::key_up(context.clone(), 65, Some(30), "KeyA", "A").normalize();
    let key_char = UiWindowPlatformInputEvent::key_char(context.clone(), 'A', true).normalize();
    let controller_pressed = UiWindowPlatformInputEvent::controller_button_pressed(
        context.clone(),
        "Virtual_Accept",
        false,
    )
    .normalize();
    let controller_repeated = UiWindowPlatformInputEvent::controller_button_pressed(
        context.clone(),
        "Gamepad_DPad_Right",
        true,
    )
    .normalize();
    let controller_released =
        UiWindowPlatformInputEvent::controller_button_released(context.clone(), "Virtual_Back")
            .normalize();
    let controller_analog =
        UiWindowPlatformInputEvent::controller_analog(context.clone(), "Gamepad_LeftX", -0.625)
            .normalize();
    let keyboard = UiWindowPlatformInputEvent::keyboard(
        context.clone(),
        UiKeyboardInputState::Repeated,
        13,
        Some(28),
        "Enter",
        "Enter",
        None,
    )
    .normalize();
    let text = UiWindowPlatformInputEvent::text(context.clone(), "A").normalize();
    let ime =
        UiWindowPlatformInputEvent::ime(context.clone(), UiImeInputEventKind::Preedit, "draft")
            .normalize();
    let ime_with_cursor = UiWindowPlatformInputEvent::ime_with_cursor_range(
        context.clone(),
        UiImeInputEventKind::Preedit,
        "draft",
        Some(UiTextByteRange::new(2, 5)),
    )
    .normalize();
    let touch = UiWindowPlatformInputEvent::touch(
        context.clone(),
        UiWindowTouchPhase::Started,
        UiPointerId::new(77),
        UiPoint::new(4.0, 8.0),
    )
    .normalize();
    let touch_started = UiWindowPlatformInputEvent::touch_started(
        context.clone(),
        UiPointerId::new(78),
        UiPoint::new(6.0, 10.0),
    )
    .normalize();
    let touch_moved = UiWindowPlatformInputEvent::touch_moved(
        context.clone(),
        UiPointerId::new(79),
        UiPoint::new(8.0, 12.0),
    )
    .normalize();
    let touch_force_changed = UiWindowPlatformInputEvent::touch_force_changed(
        context.clone(),
        UiPointerId::new(82),
        UiPoint::new(14.0, 18.0),
        0.625,
    )
    .normalize();
    let touch_first_move = UiWindowPlatformInputEvent::touch_first_move(
        context.clone(),
        UiPointerId::new(83),
        UiPoint::new(16.0, 20.0),
        0.875,
    )
    .normalize();
    let touch_ended = UiWindowPlatformInputEvent::touch_ended(
        context.clone(),
        UiPointerId::new(80),
        UiPoint::new(10.0, 14.0),
    )
    .normalize();
    let touch_canceled = UiWindowPlatformInputEvent::touch_canceled(
        context.clone(),
        UiPointerId::new(81),
        UiPoint::new(12.0, 16.0),
    )
    .normalize();
    let analog =
        UiWindowPlatformInputEvent::analog(context.clone(), "left_stick_x", 0.75).normalize();
    let navigation =
        UiWindowPlatformInputEvent::navigation(context.clone(), UiNavigationEventKind::Next)
            .normalize();
    let drag_drop = UiWindowPlatformInputEvent::drag_drop(
        context.clone(),
        UiDragDropInputEventKind::Drop,
        UiPoint::new(36.0, 48.0),
        Some(UiDragSessionId::new(12)),
        Some(UiDragPayload::new(
            UiDragPayloadKind::Asset,
            "asset://materials/grid",
        )),
    )
    .normalize();
    let drag_enter = UiWindowPlatformInputEvent::drag_enter(
        context.clone(),
        UiPoint::new(38.0, 50.0),
        Some(UiDragSessionId::new(13)),
        Some(UiDragPayload::new(
            UiDragPayloadKind::SceneInstance,
            "scene://entity/hero",
        )),
    )
    .normalize();
    let drag_over = UiWindowPlatformInputEvent::drag_over(
        context.clone(),
        UiPoint::new(40.0, 52.0),
        Some(UiDragSessionId::new(13)),
    )
    .normalize();
    let drag_leave = UiWindowPlatformInputEvent::drag_leave(
        context.clone(),
        UiPoint::new(42.0, 54.0),
        Some(UiDragSessionId::new(13)),
    )
    .normalize();
    let drag_drop_at = UiWindowPlatformInputEvent::drag_drop_at(
        context.clone(),
        UiPoint::new(44.0, 56.0),
        Some(UiDragSessionId::new(13)),
        Some(UiDragPayload::new(
            UiDragPayloadKind::Object,
            "object://entity/hero",
        )),
    )
    .normalize();
    let drag_end = UiWindowPlatformInputEvent::drag_end(
        context.clone(),
        UiPoint::new(46.0, 58.0),
        Some(UiDragSessionId::new(13)),
    )
    .normalize();
    let popup = UiWindowPlatformInputEvent::popup(
        context.clone(),
        UiPopupInputEventKind::Dismissed,
        "main.file",
        Some(UiNodeId::new(44)),
        Some(UiPoint::new(18.0, 24.0)),
    )
    .normalize();
    let popup_open = UiWindowPlatformInputEvent::popup_open_requested(
        context.clone(),
        "main.edit",
        Some(UiNodeId::new(45)),
        Some(UiPoint::new(20.0, 26.0)),
    )
    .normalize();
    let popup_close = UiWindowPlatformInputEvent::popup_close_requested(
        context.clone(),
        "main.edit",
        Some(UiNodeId::new(45)),
    )
    .normalize();
    let popup_dismissed =
        UiWindowPlatformInputEvent::popup_dismissed(context.clone(), "main.edit").normalize();
    let tooltip = UiWindowPlatformInputEvent::tooltip_timer(
        context.clone(),
        UiTooltipTimerInputEventKind::Elapsed,
        "main.file.tooltip",
        Some(UiNodeId::new(44)),
    )
    .normalize();
    let tooltip_armed = UiWindowPlatformInputEvent::tooltip_armed(
        context.clone(),
        "main.file.tooltip",
        Some(UiNodeId::new(44)),
    )
    .normalize();
    let tooltip_elapsed = UiWindowPlatformInputEvent::tooltip_elapsed(
        context.clone(),
        "main.file.tooltip",
        Some(UiNodeId::new(44)),
    )
    .normalize();
    let tooltip_canceled =
        UiWindowPlatformInputEvent::tooltip_canceled(context.clone(), "main.file.tooltip")
            .normalize();
    let accessibility = UiWindowPlatformInputEvent::accessibility(
        context,
        UiAccessibilityActionRequest {
            target: UiNodeId::new(44),
            action: UiAccessibilityAction::Dismiss,
            ..Default::default()
        },
    )
    .normalize();

    match pointer {
        UiInputEvent::Pointer(pointer) => {
            assert_eq!(pointer.metadata.timestamp, metadata.timestamp);
            assert_eq!(pointer.metadata.sequence, metadata.sequence);
            assert_eq!(pointer.metadata.window_id, Some(metadata.window_id.clone()));
            assert_eq!(pointer.metadata.user_id, Some(UiUserId::new(3)));
            assert_eq!(pointer.metadata.device_id, Some(UiDeviceId::new(9)));
            assert_eq!(pointer.metadata.pointer_id, Some(UiPointerId::new(11)));
            assert_eq!(pointer.metadata.pointer_source, UiPointerSource::Pen);
            assert_eq!(pointer.metadata.modifiers.shift, true);
            assert_eq!(pointer.metadata.modifiers.alt, true);
            assert_eq!(pointer.metadata.modifiers.caps_lock, true);
            assert_eq!(pointer.metadata.synthetic, true);
            assert_eq!(pointer.event.kind, UiPointerEventKind::Scroll);
            assert_eq!(pointer.event.button, Some(UiPointerButton::Middle));
            assert_eq!(pointer.event.scroll_delta, -42.0);
            assert_eq!(
                pointer.precise_scroll,
                Some(UiPreciseScrollDelta {
                    x: 0.0,
                    y: -42.0,
                    unit: UiScrollDeltaUnit::Pixels,
                })
            );
        }
        _ => panic!("expected pointer event"),
    }

    match mouse_move {
        UiInputEvent::Pointer(pointer) => {
            assert_eq!(pointer.metadata.window_id, Some(metadata.window_id.clone()));
            assert_eq!(pointer.metadata.user_id, Some(UiUserId::new(3)));
            assert_eq!(pointer.metadata.device_id, Some(UiDeviceId::new(9)));
            assert_eq!(pointer.metadata.pointer_id, Some(UiPointerId::new(11)));
            assert_eq!(pointer.metadata.pointer_source, UiPointerSource::Pen);
            assert_eq!(pointer.metadata.synthetic, true);
            assert_eq!(pointer.event.kind, UiPointerEventKind::Move);
            assert_eq!(pointer.event.button, None);
            assert_eq!(pointer.event.point, UiPoint::new(8.0, 9.0));
            assert_eq!(pointer.precise_scroll, None);
        }
        _ => panic!("expected mouse move pointer event"),
    }
    assert!(matches!(
        cursor_entered,
        UiInputEvent::Pointer(pointer)
            if pointer.metadata.window_id == Some(metadata.window_id.clone())
                && pointer.metadata.user_id == Some(UiUserId::new(3))
                && pointer.metadata.device_id == Some(UiDeviceId::new(9))
                && pointer.metadata.pointer_id == Some(UiPointerId::new(11))
                && pointer.metadata.pointer_source == UiPointerSource::Pen
                && pointer.event.kind == UiPointerEventKind::Move
                && pointer.event.button == None
                && pointer.event.point == UiPoint::new(9.0, 10.0)
                && pointer.precise_scroll == None
    ));
    assert!(matches!(
        cursor_left,
        UiInputEvent::Pointer(pointer)
            if pointer.metadata.window_id == Some(metadata.window_id.clone())
                && pointer.metadata.user_id == Some(UiUserId::new(3))
                && pointer.metadata.device_id == Some(UiDeviceId::new(9))
                && pointer.metadata.pointer_id == Some(UiPointerId::new(11))
                && pointer.metadata.pointer_source == UiPointerSource::Pen
                && pointer.event.kind == UiPointerEventKind::Cancel
                && pointer.event.button == None
                && pointer.event.point == UiPoint::new(11.0, 13.0)
                && pointer.precise_scroll == None
    ));
    assert!(matches!(
        mouse_capture_lost,
        UiInputEvent::Pointer(pointer)
            if pointer.metadata.window_id == Some(metadata.window_id.clone())
                && pointer.metadata.user_id == Some(UiUserId::new(3))
                && pointer.metadata.device_id == Some(UiDeviceId::new(9))
                && pointer.metadata.pointer_id == Some(UiPointerId::new(11))
                && pointer.metadata.pointer_source == UiPointerSource::Pen
                && pointer.event.kind == UiPointerEventKind::Cancel
                && pointer.event.button == None
                && pointer.event.point == UiPoint::new(17.0, 19.0)
                && pointer.precise_scroll == None
    ));

    match mouse_wheel {
        UiInputEvent::Pointer(pointer) => {
            assert_eq!(pointer.metadata.window_id, Some(metadata.window_id.clone()));
            assert_eq!(pointer.metadata.user_id, Some(UiUserId::new(3)));
            assert_eq!(pointer.metadata.device_id, Some(UiDeviceId::new(9)));
            assert_eq!(pointer.metadata.pointer_id, Some(UiPointerId::new(11)));
            assert_eq!(pointer.metadata.pointer_source, UiPointerSource::Pen);
            assert_eq!(pointer.metadata.synthetic, true);
            assert_eq!(pointer.event.kind, UiPointerEventKind::Scroll);
            assert_eq!(pointer.event.button, None);
            assert_eq!(pointer.event.point, UiPoint::new(12.0, 18.0));
            assert_eq!(pointer.event.scroll_delta, 3.0);
            assert_eq!(
                pointer.precise_scroll,
                Some(UiPreciseScrollDelta::lines(0.0, 3.0))
            );
        }
        _ => panic!("expected mouse wheel pointer event"),
    }

    match mouse_pixel_wheel {
        UiInputEvent::Pointer(pointer) => {
            assert_eq!(pointer.metadata.window_id, Some(metadata.window_id.clone()));
            assert_eq!(pointer.metadata.user_id, Some(UiUserId::new(3)));
            assert_eq!(pointer.metadata.device_id, Some(UiDeviceId::new(9)));
            assert_eq!(pointer.metadata.pointer_id, Some(UiPointerId::new(11)));
            assert_eq!(pointer.metadata.pointer_source, UiPointerSource::Pen);
            assert_eq!(pointer.metadata.synthetic, true);
            assert_eq!(pointer.event.kind, UiPointerEventKind::Scroll);
            assert_eq!(pointer.event.point, UiPoint::new(15.0, 21.0));
            assert_eq!(pointer.event.scroll_delta, -2.5);
            assert_eq!(
                pointer.precise_scroll,
                Some(UiPreciseScrollDelta::pixels(1.25, -2.5))
            );
        }
        _ => panic!("expected precise mouse wheel pointer event"),
    }

    match raw_mouse_motion {
        UiInputEvent::MouseMotion(motion) => {
            assert_eq!(motion.metadata.window_id, Some(metadata.window_id.clone()));
            assert_eq!(motion.metadata.user_id, Some(UiUserId::new(3)));
            assert_eq!(motion.metadata.device_id, Some(UiDeviceId::new(9)));
            assert_eq!(motion.metadata.synthetic, true);
            assert_eq!(motion.delta_x, -3.5);
            assert_eq!(motion.delta_y, 2.25);
        }
        _ => panic!("expected raw mouse motion event"),
    }

    match mouse_down {
        UiInputEvent::Pointer(pointer) => {
            assert_eq!(pointer.metadata.window_id, Some(metadata.window_id.clone()));
            assert_eq!(pointer.metadata.user_id, Some(UiUserId::new(3)));
            assert_eq!(pointer.metadata.device_id, Some(UiDeviceId::new(9)));
            assert_eq!(pointer.metadata.pointer_id, Some(UiPointerId::new(11)));
            assert_eq!(pointer.metadata.pointer_source, UiPointerSource::Pen);
            assert_eq!(pointer.metadata.synthetic, true);
            assert_eq!(pointer.event.kind, UiPointerEventKind::Down);
            assert_eq!(pointer.event.button, Some(UiPointerButton::Primary));
            assert_eq!(pointer.event.point, UiPoint::new(10.0, 12.0));
            assert_eq!(pointer.event.click_count, 1);
            assert_eq!(pointer.precise_scroll, None);
        }
        _ => panic!("expected mouse down pointer event"),
    }

    match mouse_up {
        UiInputEvent::Pointer(pointer) => {
            assert_eq!(pointer.metadata.window_id, Some(metadata.window_id.clone()));
            assert_eq!(pointer.metadata.user_id, Some(UiUserId::new(3)));
            assert_eq!(pointer.metadata.device_id, Some(UiDeviceId::new(9)));
            assert_eq!(pointer.metadata.pointer_id, Some(UiPointerId::new(11)));
            assert_eq!(pointer.metadata.pointer_source, UiPointerSource::Pen);
            assert_eq!(pointer.metadata.synthetic, true);
            assert_eq!(pointer.event.kind, UiPointerEventKind::Up);
            assert_eq!(pointer.event.button, Some(UiPointerButton::Primary));
            assert_eq!(pointer.event.point, UiPoint::new(10.0, 12.0));
            assert_eq!(pointer.event.click_count, 1);
            assert_eq!(pointer.precise_scroll, None);
        }
        _ => panic!("expected mouse up pointer event"),
    }

    match mouse_double_click {
        UiInputEvent::Pointer(pointer) => {
            assert_eq!(pointer.metadata.window_id, Some(metadata.window_id.clone()));
            assert_eq!(pointer.metadata.user_id, Some(UiUserId::new(3)));
            assert_eq!(pointer.metadata.device_id, Some(UiDeviceId::new(9)));
            assert_eq!(pointer.metadata.pointer_id, Some(UiPointerId::new(11)));
            assert_eq!(pointer.metadata.pointer_source, UiPointerSource::Pen);
            assert_eq!(pointer.metadata.synthetic, true);
            assert_eq!(pointer.event.kind, UiPointerEventKind::Up);
            assert_eq!(pointer.event.button, Some(UiPointerButton::Primary));
            assert_eq!(pointer.event.point, UiPoint::new(14.0, 16.0));
            assert_eq!(pointer.event.click_count, 2);
            assert_eq!(pointer.precise_scroll, None);
        }
        _ => panic!("expected mouse double-click pointer event"),
    }

    match key_down {
        UiInputEvent::Keyboard(keyboard) => {
            assert_eq!(
                keyboard.metadata.window_id,
                Some(metadata.window_id.clone())
            );
            assert_eq!(keyboard.metadata.user_id, Some(UiUserId::new(3)));
            assert_eq!(keyboard.metadata.device_id, Some(UiDeviceId::new(9)));
            assert_eq!(keyboard.state, UiKeyboardInputState::Pressed);
            assert_eq!(keyboard.key_code, 65);
            assert_eq!(keyboard.scan_code, Some(30));
            assert_eq!(keyboard.physical_key, "KeyA");
            assert_eq!(keyboard.logical_key, "A");
            assert_eq!(keyboard.text, None);
        }
        _ => panic!("expected key-down keyboard event"),
    }
    assert!(matches!(
        key_repeat,
        UiInputEvent::Keyboard(keyboard)
            if keyboard.state == UiKeyboardInputState::Repeated
                && keyboard.key_code == 65
                && keyboard.scan_code == Some(30)
                && keyboard.physical_key == "KeyA"
                && keyboard.logical_key == "A"
                && keyboard.metadata.window_id == Some(metadata.window_id.clone())
    ));
    assert!(matches!(
        key_up,
        UiInputEvent::Keyboard(keyboard)
            if keyboard.state == UiKeyboardInputState::Released
                && keyboard.key_code == 65
                && keyboard.scan_code == Some(30)
                && keyboard.physical_key == "KeyA"
                && keyboard.logical_key == "A"
                && keyboard.text == None
                && keyboard.metadata.window_id == Some(metadata.window_id.clone())
    ));
    assert!(matches!(
        key_char,
        UiInputEvent::Keyboard(keyboard)
            if keyboard.state == UiKeyboardInputState::Repeated
                && keyboard.key_code == u32::from('A')
                && keyboard.scan_code == None
                && keyboard.physical_key == "Character"
                && keyboard.logical_key == "A"
                && keyboard.text.as_deref() == Some("A")
                && keyboard.metadata.window_id == Some(metadata.window_id.clone())
    ));
    assert!(matches!(
        controller_pressed,
        UiInputEvent::Keyboard(keyboard)
            if keyboard.state == UiKeyboardInputState::Pressed
                && keyboard.key_code == 0
                && keyboard.scan_code == None
                && keyboard.physical_key == "Virtual_Accept"
                && keyboard.logical_key == "Virtual_Accept"
                && keyboard.text == None
                && keyboard.metadata.window_id == Some(metadata.window_id.clone())
    ));
    assert!(matches!(
        controller_repeated,
        UiInputEvent::Keyboard(keyboard)
            if keyboard.state == UiKeyboardInputState::Repeated
                && keyboard.key_code == 0
                && keyboard.scan_code == None
                && keyboard.physical_key == "Gamepad_DPad_Right"
                && keyboard.logical_key == "Gamepad_DPad_Right"
                && keyboard.text == None
                && keyboard.metadata.window_id == Some(metadata.window_id.clone())
    ));
    assert!(matches!(
        controller_released,
        UiInputEvent::Keyboard(keyboard)
            if keyboard.state == UiKeyboardInputState::Released
                && keyboard.key_code == 0
                && keyboard.scan_code == None
                && keyboard.physical_key == "Virtual_Back"
                && keyboard.logical_key == "Virtual_Back"
                && keyboard.text == None
                && keyboard.metadata.window_id == Some(metadata.window_id.clone())
    ));
    assert!(matches!(
        controller_analog,
        UiInputEvent::Analog(analog)
            if analog.control == "Gamepad_LeftX"
                && analog.value == -0.625
                && analog.metadata.window_id == Some(metadata.window_id.clone())
                && analog.metadata.user_id == Some(UiUserId::new(3))
                && analog.metadata.device_id == Some(UiDeviceId::new(9))
    ));

    match keyboard {
        UiInputEvent::Keyboard(keyboard) => {
            assert_eq!(
                keyboard.metadata.window_id,
                Some(metadata.window_id.clone())
            );
            assert_eq!(keyboard.state, UiKeyboardInputState::Repeated);
            assert_eq!(keyboard.key_code, 13);
            assert_eq!(keyboard.scan_code, Some(28));
            assert_eq!(keyboard.logical_key, "Enter");
        }
        _ => panic!("expected keyboard event"),
    }
    assert!(matches!(text, UiInputEvent::Text(text) if text.text == "A"));
    assert!(matches!(
        ime,
        UiInputEvent::Ime(ime)
            if ime.kind == UiImeInputEventKind::Preedit
                && ime.text == "draft"
                && ime.cursor_range == None
                && ime.metadata.window_id == Some(metadata.window_id.clone())
    ));
    assert!(matches!(
        ime_with_cursor,
        UiInputEvent::Ime(ime)
            if ime.kind == UiImeInputEventKind::Preedit
                && ime.text == "draft"
                && ime.cursor_range == Some(UiTextByteRange::new(2, 5))
                && ime.metadata.window_id == Some(metadata.window_id.clone())
    ));
    assert!(matches!(
        touch,
        UiInputEvent::Pointer(pointer)
            if pointer.metadata.pointer_source == UiPointerSource::Touch
                && pointer.metadata.pointer_id == Some(UiPointerId::new(77))
                && pointer.event.kind == UiPointerEventKind::Down
                && pointer.event.button == Some(UiPointerButton::Primary)
    ));
    assert!(matches!(
        touch_started,
        UiInputEvent::Pointer(pointer)
            if pointer.metadata.pointer_source == UiPointerSource::Touch
                && pointer.metadata.pointer_id == Some(UiPointerId::new(78))
                && pointer.event.kind == UiPointerEventKind::Down
                && pointer.event.button == Some(UiPointerButton::Primary)
                && pointer.event.point == UiPoint::new(6.0, 10.0)
    ));
    assert!(matches!(
        touch_moved,
        UiInputEvent::Pointer(pointer)
            if pointer.metadata.pointer_source == UiPointerSource::Touch
                && pointer.metadata.pointer_id == Some(UiPointerId::new(79))
                && pointer.event.kind == UiPointerEventKind::Move
                && pointer.event.button == None
                && pointer.event.point == UiPoint::new(8.0, 12.0)
    ));
    assert!(matches!(
        touch_force_changed,
        UiInputEvent::Pointer(pointer)
            if pointer.metadata.pointer_source == UiPointerSource::Touch
                && pointer.metadata.pointer_id == Some(UiPointerId::new(82))
                && pointer.event.kind == UiPointerEventKind::Move
                && pointer.event.button == None
                && pointer.event.point == UiPoint::new(14.0, 18.0)
    ));
    assert!(matches!(
        touch_first_move,
        UiInputEvent::Pointer(pointer)
            if pointer.metadata.pointer_source == UiPointerSource::Touch
                && pointer.metadata.pointer_id == Some(UiPointerId::new(83))
                && pointer.event.kind == UiPointerEventKind::Move
                && pointer.event.button == None
                && pointer.event.point == UiPoint::new(16.0, 20.0)
    ));
    assert!(matches!(
        touch_ended,
        UiInputEvent::Pointer(pointer)
            if pointer.metadata.pointer_source == UiPointerSource::Touch
                && pointer.metadata.pointer_id == Some(UiPointerId::new(80))
                && pointer.event.kind == UiPointerEventKind::Up
                && pointer.event.button == Some(UiPointerButton::Primary)
                && pointer.event.point == UiPoint::new(10.0, 14.0)
    ));
    assert!(matches!(
        touch_canceled,
        UiInputEvent::Pointer(pointer)
            if pointer.metadata.pointer_source == UiPointerSource::Touch
                && pointer.metadata.pointer_id == Some(UiPointerId::new(81))
                && pointer.event.kind == UiPointerEventKind::Cancel
                && pointer.event.button == None
                && pointer.event.point == UiPoint::new(12.0, 16.0)
    ));
    assert!(matches!(
        analog,
        UiInputEvent::Analog(analog)
            if analog.control == "left_stick_x"
                && analog.value == 0.75
                && analog.metadata.window_id == Some(metadata.window_id.clone())
    ));
    assert!(matches!(
        navigation,
        UiInputEvent::Navigation(navigation)
            if navigation.kind == UiNavigationEventKind::Next
                && navigation.metadata.window_id == Some(metadata.window_id.clone())
    ));
    assert!(matches!(
        drag_drop,
        UiInputEvent::DragDrop(drag_drop)
            if drag_drop.kind == UiDragDropInputEventKind::Drop
                && drag_drop.session_id == Some(UiDragSessionId::new(12))
                && drag_drop.point == UiPoint::new(36.0, 48.0)
                && drag_drop.metadata.window_id == Some(metadata.window_id.clone())
                && drag_drop
                    .payload
                    .as_ref()
                    .is_some_and(|payload| payload.kind == UiDragPayloadKind::Asset)
    ));
    assert!(matches!(
        drag_enter,
        UiInputEvent::DragDrop(drag_drop)
            if drag_drop.kind == UiDragDropInputEventKind::Enter
                && drag_drop.session_id == Some(UiDragSessionId::new(13))
                && drag_drop.point == UiPoint::new(38.0, 50.0)
                && drag_drop.metadata.window_id == Some(metadata.window_id.clone())
                && drag_drop
                    .payload
                    .as_ref()
                    .is_some_and(|payload| payload.kind == UiDragPayloadKind::SceneInstance)
    ));
    assert!(matches!(
        drag_over,
        UiInputEvent::DragDrop(drag_drop)
            if drag_drop.kind == UiDragDropInputEventKind::Over
                && drag_drop.session_id == Some(UiDragSessionId::new(13))
                && drag_drop.point == UiPoint::new(40.0, 52.0)
                && drag_drop.payload.is_none()
                && drag_drop.metadata.window_id == Some(metadata.window_id.clone())
    ));
    assert!(matches!(
        drag_leave,
        UiInputEvent::DragDrop(drag_drop)
            if drag_drop.kind == UiDragDropInputEventKind::Leave
                && drag_drop.session_id == Some(UiDragSessionId::new(13))
                && drag_drop.point == UiPoint::new(42.0, 54.0)
                && drag_drop.payload.is_none()
                && drag_drop.metadata.window_id == Some(metadata.window_id.clone())
    ));
    assert!(matches!(
        drag_drop_at,
        UiInputEvent::DragDrop(drag_drop)
            if drag_drop.kind == UiDragDropInputEventKind::Drop
                && drag_drop.session_id == Some(UiDragSessionId::new(13))
                && drag_drop.point == UiPoint::new(44.0, 56.0)
                && drag_drop
                    .payload
                    .as_ref()
                    .is_some_and(|payload| payload.kind == UiDragPayloadKind::Object)
                && drag_drop.metadata.window_id == Some(metadata.window_id.clone())
    ));
    assert!(matches!(
        drag_end,
        UiInputEvent::DragDrop(drag_drop)
            if drag_drop.kind == UiDragDropInputEventKind::End
                && drag_drop.session_id == Some(UiDragSessionId::new(13))
                && drag_drop.point == UiPoint::new(46.0, 58.0)
                && drag_drop.payload.is_none()
                && drag_drop.metadata.window_id == Some(metadata.window_id.clone())
    ));
    assert!(matches!(
        popup,
        UiInputEvent::Popup(popup)
            if popup.kind == UiPopupInputEventKind::Dismissed
                && popup.popup_id == "main.file"
                && popup.owner == Some(UiNodeId::new(44))
                && popup.anchor == Some(UiPoint::new(18.0, 24.0))
                && popup.metadata.window_id == Some(metadata.window_id.clone())
    ));
    assert!(matches!(
        popup_open,
        UiInputEvent::Popup(popup)
            if popup.kind == UiPopupInputEventKind::OpenRequested
                && popup.popup_id == "main.edit"
                && popup.owner == Some(UiNodeId::new(45))
                && popup.anchor == Some(UiPoint::new(20.0, 26.0))
                && popup.metadata.window_id == Some(metadata.window_id.clone())
    ));
    assert!(matches!(
        popup_close,
        UiInputEvent::Popup(popup)
            if popup.kind == UiPopupInputEventKind::CloseRequested
                && popup.popup_id == "main.edit"
                && popup.owner == Some(UiNodeId::new(45))
                && popup.anchor == None
                && popup.metadata.window_id == Some(metadata.window_id.clone())
    ));
    assert!(matches!(
        popup_dismissed,
        UiInputEvent::Popup(popup)
            if popup.kind == UiPopupInputEventKind::Dismissed
                && popup.popup_id == "main.edit"
                && popup.owner == None
                && popup.anchor == None
                && popup.metadata.window_id == Some(metadata.window_id.clone())
    ));
    assert!(matches!(
        tooltip,
        UiInputEvent::TooltipTimer(tooltip)
            if tooltip.kind == UiTooltipTimerInputEventKind::Elapsed
                && tooltip.tooltip_id == "main.file.tooltip"
                && tooltip.owner == Some(UiNodeId::new(44))
                && tooltip.metadata.window_id == Some(metadata.window_id.clone())
    ));
    assert!(matches!(
        tooltip_armed,
        UiInputEvent::TooltipTimer(tooltip)
            if tooltip.kind == UiTooltipTimerInputEventKind::Armed
                && tooltip.tooltip_id == "main.file.tooltip"
                && tooltip.owner == Some(UiNodeId::new(44))
                && tooltip.metadata.window_id == Some(metadata.window_id.clone())
    ));
    assert!(matches!(
        tooltip_elapsed,
        UiInputEvent::TooltipTimer(tooltip)
            if tooltip.kind == UiTooltipTimerInputEventKind::Elapsed
                && tooltip.tooltip_id == "main.file.tooltip"
                && tooltip.owner == Some(UiNodeId::new(44))
                && tooltip.metadata.window_id == Some(metadata.window_id.clone())
    ));
    assert!(matches!(
        tooltip_canceled,
        UiInputEvent::TooltipTimer(tooltip)
            if tooltip.kind == UiTooltipTimerInputEventKind::Canceled
                && tooltip.tooltip_id == "main.file.tooltip"
                && tooltip.owner == None
                && tooltip.metadata.window_id == Some(metadata.window_id.clone())
    ));
    assert!(matches!(
        accessibility,
        UiInputEvent::Accessibility(accessibility)
            if accessibility.request.target == UiNodeId::new(44)
                && accessibility.request.action == UiAccessibilityAction::Dismiss
                && accessibility.metadata.window_id == Some(metadata.window_id)
    ));
}

#[test]
fn ui_window_cursor_events_can_enter_the_unified_pointer_pipeline() {
    let metadata = sample_window_metadata();
    let cursor_moved = UiWindowEvent::new(
        metadata.clone(),
        UiWindowEventKind::CursorMoved {
            position: UiPoint::new(32.0, 48.0),
            delta: Some(UiPoint::new(2.0, 3.0)),
        },
    );
    let cursor_left = UiWindowEvent::new(metadata.clone(), UiWindowEventKind::CursorLeft);
    let closed = UiWindowEvent::new(metadata, UiWindowEventKind::Closed);

    let moved = cursor_moved.normalized_cursor_move_input().unwrap();
    let left = cursor_left
        .normalized_pointer_cancel_input(UiPoint::new(32.0, 48.0))
        .unwrap();
    let close = closed
        .normalized_pointer_cancel_input(UiPoint::new(32.0, 48.0))
        .unwrap();

    match moved {
        UiInputEvent::Pointer(pointer) => {
            assert_eq!(
                pointer.metadata.window_id,
                Some(UiWindowId::new("editor.main"))
            );
            assert_eq!(pointer.event.kind, UiPointerEventKind::Move);
            assert_eq!(pointer.event.point, UiPoint::new(32.0, 48.0));
        }
        _ => panic!("expected pointer move"),
    }
    assert!(matches!(
        left,
        UiInputEvent::Pointer(pointer)
            if pointer.event.kind == UiPointerEventKind::Cancel
                && pointer.event.point == UiPoint::new(32.0, 48.0)
    ));
    assert!(matches!(
        close,
        UiInputEvent::Pointer(pointer)
            if pointer.event.kind == UiPointerEventKind::Cancel
                && pointer.metadata.window_id == Some(UiWindowId::new("editor.main"))
    ));
    assert!(
        UiWindowEvent::new(sample_window_metadata(), UiWindowEventKind::CursorEntered)
            .normalized_cursor_move_input()
            .is_none()
    );
}
