use super::*;

#[test]
fn window_input_pump_cursor_move_dispatches_unified_pointer_hover_route() {
    let mut surface = route_surface_with_hover_bindings();

    let result = dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::new(
            window_metadata(11, true),
            UiWindowEventKind::CursorMoved {
                position: UiPoint::new(20.0, 60.0),
                delta: Some(UiPoint::new(1.0, 2.0)),
            },
        )),
    )
    .unwrap();

    let UiInputEvent::Pointer(pointer) = &result.event else {
        panic!("expected cursor move to normalize into pointer input");
    };
    assert_eq!(
        pointer.metadata.window_id,
        Some(UiWindowId::new("main-window"))
    );
    assert!(pointer.metadata.synthetic);
    assert_eq!(pointer.event.kind, UiPointerEventKind::Move);
    assert_eq!(pointer.event.point, UiPoint::new(20.0, 60.0));
    assert_eq!(pointer.precise_scroll, None);
    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Direct);
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(3)));
    assert_eq!(result.diagnostics.handled_phase.as_deref(), Some("pointer"));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(3)));
    assert_eq!(
        result.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(3))
    );
    assert_eq!(
        surface.focus.hovered,
        vec![UiNodeId::new(3), UiNodeId::new(1)]
    );
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.component_events[0].target, UiNodeId::new(3));
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::Hover { hovered: true }
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_input_pump"));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_normalized_input"));
}

#[test]
fn window_input_pump_cursor_left_replays_pointer_cancel_and_clears_hover() {
    let mut surface = route_surface_with_hover_bindings();

    dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::new(
            window_metadata(12, true),
            UiWindowEventKind::CursorMoved {
                position: UiPoint::new(20.0, 60.0),
                delta: None,
            },
        )),
    )
    .unwrap();

    assert_eq!(
        surface.input.last_cursor_point(),
        Some(UiPoint::new(20.0, 60.0))
    );
    assert_eq!(
        surface.focus.hovered,
        vec![UiNodeId::new(3), UiNodeId::new(1)]
    );
    assert!(surface
        .component_state(UiNodeId::new(3))
        .is_some_and(|state| state.flags.hovered));

    let result = dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::new(
            window_metadata(13, true),
            UiWindowEventKind::CursorLeft,
        )),
    )
    .unwrap();

    let UiInputEvent::Pointer(pointer) = &result.event else {
        panic!("expected cursor leave to normalize into pointer cancel");
    };
    assert_eq!(pointer.event.kind, UiPointerEventKind::Cancel);
    assert_eq!(pointer.event.point, UiPoint::new(20.0, 60.0));
    assert!(pointer.metadata.synthetic);
    assert_eq!(surface.input.last_cursor_point(), None);
    assert!(surface.focus.hovered.is_empty());
    assert!(!surface
        .component_state(UiNodeId::new(3))
        .is_some_and(|state| state.flags.hovered));
    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Direct);
    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(3)
            && matches!(&event.event, UiComponentEvent::Hover { hovered: false })
    }));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_pointer_cancel"));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_hover_cleared"));
}

#[test]
fn window_input_pump_touch_move_does_not_replace_last_mouse_cursor_point() {
    let mut surface = route_surface_with_hover_bindings();

    dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::new(
            window_metadata(14, true),
            UiWindowEventKind::CursorMoved {
                position: UiPoint::new(20.0, 60.0),
                delta: None,
            },
        )),
    )
    .unwrap();

    let touch_input = UiWindowPlatformInputEvent::pointer(
        UiWindowInputContext::from_window_metadata(&window_metadata(15, true))
            .with_pointer_source(UiPointerSource::Touch),
        UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(70.0, 70.0)),
        None,
    )
    .normalize();
    dispatch_window_input_pump_event(&mut surface, UiWindowInputPumpEvent::Input(touch_input))
        .unwrap();

    assert_eq!(
        surface.input.last_cursor_point(),
        Some(UiPoint::new(20.0, 60.0))
    );

    let result = dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::new(
            window_metadata(16, true),
            UiWindowEventKind::CursorLeft,
        )),
    )
    .unwrap();

    let UiInputEvent::Pointer(pointer) = &result.event else {
        panic!("expected cursor leave to normalize into pointer cancel");
    };
    assert_eq!(pointer.event.kind, UiPointerEventKind::Cancel);
    assert_eq!(pointer.event.point, UiPoint::new(20.0, 60.0));
}

#[test]
fn window_input_pump_closed_without_cursor_point_clears_hover_without_fake_pointer_cancel() {
    let mut surface = route_surface_with_hover_bindings();
    surface.focus.hovered = vec![UiNodeId::new(3), UiNodeId::new(1)];
    let _ = surface.component_states.set_hovered(UiNodeId::new(3), true);

    let result = dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::new(
            window_metadata(17, true),
            UiWindowEventKind::Closed,
        )),
    )
    .unwrap();

    assert!(surface.window_state.closed);
    assert!(surface.surface_frame().window_state.closed);
    assert!(matches!(
        &result.event,
        UiInputEvent::Popup(UiPopupInputEvent {
            kind: UiPopupInputEventKind::Dismissed,
            popup_id,
            ..
        }) if popup_id == "window.transient"
    ));
    assert!(surface.focus.hovered.is_empty());
    assert_eq!(surface.input.last_cursor_point(), None);
    assert!(!surface
        .component_state(UiNodeId::new(3))
        .is_some_and(|state| state.flags.hovered));
    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(3)
            && matches!(&event.event, UiComponentEvent::Hover { hovered: false })
    }));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_closed"));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_pointer_cancel_missing_point"));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_hover_cleared"));
}
