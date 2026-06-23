use super::*;

#[test]
fn input_manager_route_matrix_capture_preempts_hit_target() {
    let mut surface = route_matrix_surface();
    let mut manager = UiInputManager::default();
    manager
        .pointer_dispatcher_mut()
        .register(UiNodeId::new(2), UiPointerEventKind::Down, |_| {
            UiPointerDispatchEffect::capture()
        });
    manager
        .pointer_dispatcher_mut()
        .register(UiNodeId::new(2), UiPointerEventKind::Move, |_| {
            UiPointerDispatchEffect::handled()
        });

    let down = manager
        .dispatch_input_event(
            &mut surface,
            pointer_event(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();

    assert_eq!(down.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(down.reply.handler, Some(UiNodeId::new(2)));
    assert_eq!(
        surface.input.pointer_capture_owner(UiPointerId::new(7)),
        Some(UiNodeId::new(2))
    );
    let active_down = manager
        .active_pointers()
        .entry(UiPointerId::new(7))
        .unwrap();
    assert_eq!(active_down.last_point, Some(UiPoint::new(20.0, 20.0)));
    assert_eq!(
        active_down.hovered,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(active_down.pressed_buttons, 0b001);
    assert_eq!(active_down.pressed_target, Some(UiNodeId::new(2)));
    assert_eq!(active_down.capture_target, Some(UiNodeId::new(2)));

    let moved = manager
        .dispatch_input_event(
            &mut surface,
            pointer_event(UiPointerEventKind::Move, UiPoint::new(120.0, 20.0)),
        )
        .unwrap();

    assert_eq!(
        moved.diagnostics.route_policy,
        UiInputRoutePolicy::PointerCapture
    );
    assert_eq!(moved.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(moved.reply.handler, Some(UiNodeId::new(2)));
    assert_eq!(
        moved.diagnostics.route_trace.capture_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        moved.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(moved.diagnostics.route_steps.len(), 1);
    assert_eq!(
        moved.diagnostics.route_steps[0].phase,
        UiDispatchPhase::Direct
    );
    let active_move = manager
        .active_pointers()
        .entry(UiPointerId::new(7))
        .unwrap();
    assert_eq!(active_move.last_point, Some(UiPoint::new(120.0, 20.0)));
    assert_eq!(
        active_move.hovered,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(active_move.pressed_buttons, 0b001);
    assert_eq!(active_move.capture_target, Some(UiNodeId::new(2)));
}

#[test]
fn input_manager_route_matrix_popup_outside_closes_top_only() {
    let mut surface = popup_matrix_surface();
    let mut manager = UiInputManager::default();
    assert_popup_stack(&surface, &["root/popup", "root/popup/nested"]);

    manager
        .dispatch_input_event(
            &mut surface,
            pointer_event(UiPointerEventKind::Down, UiPoint::new(170.0, 100.0)),
        )
        .unwrap();
    assert_popup_stack(&surface, &["root/popup", "root/popup/nested"]);

    let released = manager
        .dispatch_input_event(
            &mut surface,
            pointer_event(UiPointerEventKind::Up, UiPoint::new(170.0, 100.0)),
        )
        .unwrap();

    assert_eq!(released.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        released.diagnostics.route_policy,
        UiInputRoutePolicy::Bubble
    );
    assert_popup_node_open(&surface, UiNodeId::new(2), true);
    assert_popup_node_open(&surface, UiNodeId::new(4), false);
    assert_popup_stack(&surface, &["root/popup"]);
    assert!(released.component_events.iter().any(|event| {
        event.target == UiNodeId::new(4) && matches!(&event.event, UiComponentEvent::ClosePopup)
    }));
    assert!(released.component_events.iter().all(|event| {
        event.target != UiNodeId::new(2) || !matches!(&event.event, UiComponentEvent::ClosePopup)
    }));
}

#[test]
fn input_manager_route_matrix_preview_stops_before_bubble() {
    let mut surface = route_matrix_surface();
    let mut manager = UiInputManager::default();
    let target_calls = Arc::new(AtomicUsize::new(0));
    manager.pointer_dispatcher_mut().register_phase(
        UiNodeId::new(1),
        UiPointerEventKind::Down,
        UiDispatchPhase::PreviewTunnel,
        |context| {
            assert_eq!(context.node_id, UiNodeId::new(1));
            assert_eq!(context.phase, UiDispatchPhase::PreviewTunnel);
            assert_eq!(context.route.target, Some(UiNodeId::new(2)));
            UiPointerDispatchEffect::handled()
        },
    );
    let target_calls_for_handler = Arc::clone(&target_calls);
    manager.pointer_dispatcher_mut().register(
        UiNodeId::new(2),
        UiPointerEventKind::Down,
        move |_| {
            target_calls_for_handler.fetch_add(1, Ordering::SeqCst);
            UiPointerDispatchEffect::handled()
        },
    );

    let result = manager
        .dispatch_input_event(
            &mut surface,
            pointer_event(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();

    assert_eq!(target_calls.load(Ordering::SeqCst), 0);
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(1)));
    assert_eq!(result.reply.phase, Some(UiDispatchPhase::PreviewTunnel));
    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(2)]
    );
    assert_eq!(result.diagnostics.route_steps.len(), 1);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
}

#[test]
fn input_manager_route_matrix_keyboard_uses_focus_path() {
    let mut surface = route_matrix_surface();
    let mut manager = UiInputManager::default();
    surface.focus.focused = Some(UiNodeId::new(2));

    let result = manager
        .dispatch_input_event(&mut surface, keyboard_event())
        .unwrap();

    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert!(result.diagnostics.routed);
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
}

#[test]
fn input_manager_route_matrix_popup_open_uses_default_action() {
    let mut surface = route_matrix_surface();
    let mut manager = UiInputManager::default();

    let result = manager
        .dispatch_input_event(
            &mut surface,
            popup_event(
                UiPopupInputEventKind::OpenRequested,
                "matrix.popup",
                Some(UiNodeId::new(2)),
            ),
        )
        .unwrap();

    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::DefaultAction
    );
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("popup.effect")
    );
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::DefaultAction
    );
    assert_eq!(popup_stack_ids(&surface), vec!["matrix.popup"]);
}
