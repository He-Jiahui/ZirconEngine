use super::*;

#[test]
fn unified_pointer_dispatch_reports_phase_route_steps() {
    let mut surface = route_surface();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert!(result.reply.effects.is_empty());
    assert_eq!(result.diagnostics.route_steps.len(), 4);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_steps[0].disposition,
        UiDispatchDisposition::Passthrough
    );
    assert_eq!(
        result.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[1].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].phase,
        UiDispatchPhase::Target
    );
    assert_eq!(
        result.diagnostics.route_steps[2].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].handler,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].disposition,
        UiDispatchDisposition::Unhandled
    );
    assert_eq!(result.diagnostics.route_steps[2].effect_count, 0);
    assert!(!result.diagnostics.route_steps[2].stopped);
    assert_eq!(
        result.diagnostics.route_steps[3].phase,
        UiDispatchPhase::Bubble
    );
    assert_eq!(
        result.diagnostics.route_steps[3].target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_steps[3].disposition,
        UiDispatchDisposition::Passthrough
    );
}

#[test]
fn pointer_preview_tunnel_handler_stops_before_target_and_bubble_handlers() {
    let mut surface = route_surface();
    let target_calls = Arc::new(AtomicUsize::new(0));
    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register_phase(
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
    dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Down, move |_| {
        target_calls_for_handler.fetch_add(1, Ordering::SeqCst);
        UiPointerDispatchEffect::handled()
    });

    let result = surface
        .dispatch_input_event(
            &dispatcher,
            &UiNavigationDispatcher::default(),
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
    assert_eq!(
        result.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_steps[0].handler,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_steps[0].disposition,
        UiDispatchDisposition::Handled
    );
    assert!(result.diagnostics.route_steps[0].stopped);
}

#[test]
fn unified_pointer_press_release_report_bubble_route_steps_and_component_events() {
    let mut surface = press_release_route_surface();

    let down = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();

    assert_two_node_bubble_handled_at_target(&down);
    assert_eq!(down.component_events.len(), 1);
    assert_eq!(down.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        down.component_events[0].event,
        UiComponentEvent::Press { pressed: true }
    );
    assert_eq!(surface.focus.pressed, Some(UiNodeId::new(2)));
    assert!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .state_flags
            .pressed
    );

    let up = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();

    assert_two_node_bubble_handled_at_target(&up);
    assert_eq!(up.component_events.len(), 2);
    assert_eq!(up.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        up.component_events[0].event,
        UiComponentEvent::Press { pressed: false }
    );
    assert_eq!(up.component_events[1].target, UiNodeId::new(2));
    assert_eq!(
        up.component_events[1].event,
        UiComponentEvent::Commit {
            property: "activated".to_string(),
            value: UiValue::Bool(true),
        }
    );
    assert_eq!(surface.focus.pressed, None);
    assert!(
        !surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .state_flags
            .pressed
    );
}

#[test]
fn unified_pointer_double_click_reports_bubble_route_steps_and_default_binding() {
    let mut surface = double_click_route_surface();

    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event_with_click_count(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0), 2),
        )
        .unwrap();

    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(2)));
    assert!(result.reply.effects.is_empty());
    assert_eq!(result.diagnostics.handled_phase.as_deref(), Some("pointer"));
    assert_eq!(
        result.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(2)]
    );
    assert_eq!(
        result.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::Commit {
            property: "double_activated".to_string(),
            value: UiValue::Bool(true),
        }
    );
    match &result.event {
        UiInputEvent::Pointer(pointer) => assert_eq!(pointer.event.click_count, 2),
        _ => panic!("expected pointer input event"),
    }
    assert_eq!(result.diagnostics.route_steps.len(), 3);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_steps[0].disposition,
        UiDispatchDisposition::Passthrough
    );
    assert_eq!(
        result.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[1].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].phase,
        UiDispatchPhase::Target
    );
    assert_eq!(
        result.diagnostics.route_steps[2].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].handler,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(result.diagnostics.route_steps[2].effect_count, 0);
    assert!(result.diagnostics.route_steps[2].stopped);
}

#[test]
fn unified_pointer_scroll_reports_bubble_route_steps_and_precise_delta() {
    let mut surface = scroll_route_surface();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            scroll_event(UiPoint::new(20.0, 20.0), 50.0),
        )
        .unwrap();

    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(20)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(2)));
    assert_eq!(result.diagnostics.handled_phase.as_deref(), Some("pointer"));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "scroll_delta=50"));
    assert_eq!(
        result.diagnostics.route_trace.target,
        Some(UiNodeId::new(20))
    );
    assert_eq!(
        result.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(2), UiNodeId::new(20)]
    );
    assert_eq!(
        result.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(20), UiNodeId::new(2), UiNodeId::new(1)]
    );
    match &result.event {
        UiInputEvent::Pointer(pointer) => {
            assert_eq!(pointer.event.scroll_delta, 50.0);
            assert_eq!(
                pointer.precise_scroll,
                Some(UiPreciseScrollDelta::pixels(0.0, 50.0))
            );
        }
        _ => panic!("expected pointer input event"),
    }
    assert_eq!(result.diagnostics.route_steps.len(), 5);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[1].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[2].target,
        Some(UiNodeId::new(20))
    );
    assert_eq!(
        result.diagnostics.route_steps[3].phase,
        UiDispatchPhase::Target
    );
    assert_eq!(
        result.diagnostics.route_steps[3].target,
        Some(UiNodeId::new(20))
    );
    assert_eq!(
        result.diagnostics.route_steps[3].disposition,
        UiDispatchDisposition::Passthrough
    );
    assert!(!result.diagnostics.route_steps[3].stopped);
    assert_eq!(
        result.diagnostics.route_steps[4].phase,
        UiDispatchPhase::Bubble
    );
    assert_eq!(
        result.diagnostics.route_steps[4].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[4].handler,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[4].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(result.diagnostics.route_steps[4].effect_count, 0);
    assert!(result.diagnostics.route_steps[4].stopped);
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .scroll_state
            .unwrap()
            .offset,
        50.0
    );
}
