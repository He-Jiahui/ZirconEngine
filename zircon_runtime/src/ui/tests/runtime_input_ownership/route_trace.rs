use super::*;

#[test]
fn analog_input_suppresses_repeated_values_before_routing() {
    let mut surface = two_button_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let first = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            analog_event("gamepad.left_x", 0.5),
        )
        .unwrap();
    assert!(first.diagnostics.routed);
    assert_eq!(first.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(first.reply.disposition, UiDispatchDisposition::Handled);

    let repeated = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            analog_event("gamepad.left_x", 0.5004),
        )
        .unwrap();
    assert!(!repeated.diagnostics.routed);
    assert_eq!(repeated.reply.disposition, UiDispatchDisposition::Unhandled);
    assert!(repeated
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "analog_repeat_suppressed"));
    assert_eq!(
        surface
            .input
            .analog_controls
            .get("gamepad.left_x")
            .map(|state| state.value),
        Some(0.5)
    );

    let changed = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            analog_event("gamepad.left_x", 0.75),
        )
        .unwrap();
    assert!(changed.diagnostics.routed);
    assert_eq!(changed.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        surface
            .input
            .analog_controls
            .get("gamepad.left_x")
            .map(|state| state.value),
        Some(0.75)
    );
}

#[test]
fn unified_input_dispatch_reports_slate_style_pointer_and_focus_route_trace() {
    let mut surface = two_button_surface();
    let pointer_dispatcher = UiPointerDispatcher::default();
    let navigation_dispatcher = UiNavigationDispatcher::default();

    let pointer = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            pointer_event(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();

    assert_eq!(pointer.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        pointer.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        pointer.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(
        pointer.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(2)]
    );
    assert_eq!(
        pointer.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );

    let keyboard = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            keyboard_event(),
        )
        .unwrap();

    assert_eq!(
        keyboard.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(
        keyboard.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(2)]
    );
    assert_eq!(keyboard.diagnostics.route_trace.capture_target, None);
}

#[test]
fn unified_input_dispatch_trace_reports_capture_and_popup_stack() {
    let mut surface = two_button_surface();
    let pointer_id = UiPointerId::new(7);
    let session_id = UiDragSessionId::new(42);
    let pointer_dispatcher = UiPointerDispatcher::default();
    let navigation_dispatcher = UiNavigationDispatcher::default();

    surface
        .input
        .begin_drag_drop(
            UiNodeId::new(2),
            UiNodeId::new(2),
            pointer_id,
            Some(session_id),
            Some(UiPoint::new(20.0, 20.0)),
            None,
        )
        .unwrap();
    capture_pointer_for_test(&mut surface, pointer_id, UiNodeId::new(2));
    surface.input.open_popup(
        "menu.file".to_string(),
        Some(UiNodeId::new(2)),
        Some(UiPoint::new(8.0, 12.0)),
    );

    let drag = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            drag_drop_input_event(
                UiDragDropInputEventKind::Over,
                Some(session_id),
                UiPoint::new(20.0, 60.0),
                None,
            ),
        )
        .unwrap();

    assert_eq!(
        drag.diagnostics.route_trace.capture_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        drag.diagnostics.route_trace.popup_stack,
        vec!["menu.file".to_string()]
    );
    assert_eq!(
        drag.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(2))
    );

    let popup_close = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            popup_input_event_for_owner(
                UiPopupInputEventKind::CloseRequested,
                "menu.file",
                Some(UiNodeId::new(2)),
                None,
            ),
        )
        .unwrap();

    assert_eq!(
        popup_close.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        popup_close.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(
        popup_close.diagnostics.route_trace.capture_target,
        Some(UiNodeId::new(2))
    );
}
