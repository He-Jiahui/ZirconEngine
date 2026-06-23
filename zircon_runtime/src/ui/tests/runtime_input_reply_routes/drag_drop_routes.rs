use super::*;

#[test]
fn drag_drop_over_trace_uses_drop_target_path_and_preserves_capture_source() {
    let mut surface = route_surface();
    let session_id = UiDragSessionId::new(42);

    let begin = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            drag_drop_event(
                UiDragDropInputEventKind::Begin,
                Some(session_id),
                UiPoint::new(20.0, 20.0),
            ),
        )
        .unwrap();

    assert!(begin.rejected_effects.is_empty());
    assert_eq!(surface.focus.captured, Some(UiNodeId::new(2)));
    assert_eq!(begin.diagnostics.route_policy, UiInputRoutePolicy::Direct);
    assert_eq!(
        begin.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(2))
    );

    let over = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            drag_drop_event(
                UiDragDropInputEventKind::Over,
                Some(session_id),
                UiPoint::new(20.0, 60.0),
            ),
        )
        .unwrap();

    assert!(over.rejected_effects.is_empty());
    assert_eq!(surface.focus.captured, Some(UiNodeId::new(2)));
    assert_eq!(over.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(over.diagnostics.route_target, Some(UiNodeId::new(3)));
    assert_eq!(over.diagnostics.route_trace.target, Some(UiNodeId::new(3)));
    assert_eq!(over.diagnostics.route_trace.direct_target, None);
    assert_eq!(
        over.diagnostics.route_trace.capture_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        over.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(3), UiNodeId::new(1)]
    );
    assert_eq!(
        over.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(3)]
    );
    assert_eq!(over.diagnostics.route_steps.len(), 3);
    assert_eq!(
        over.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        over.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        over.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        over.diagnostics.route_steps[1].target,
        Some(UiNodeId::new(3))
    );
    assert_eq!(
        over.diagnostics.route_steps[2].phase,
        UiDispatchPhase::Target
    );
    assert_eq!(
        over.diagnostics.route_steps[2].target,
        Some(UiNodeId::new(3))
    );
    assert_eq!(
        over.diagnostics.route_steps[2].handler,
        Some(UiNodeId::new(3))
    );

    let dropped = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            drag_drop_event(
                UiDragDropInputEventKind::Drop,
                Some(session_id),
                UiPoint::new(20.0, 60.0),
            ),
        )
        .unwrap();

    assert!(dropped.rejected_effects.is_empty());
    assert_eq!(dropped.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(
        dropped.diagnostics.route_trace.target,
        Some(UiNodeId::new(3))
    );
    assert_eq!(
        dropped.diagnostics.route_trace.capture_target,
        Some(UiNodeId::new(2))
    );

    let ended = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            drag_drop_event(
                UiDragDropInputEventKind::End,
                Some(session_id),
                UiPoint::new(20.0, 60.0),
            ),
        )
        .unwrap();

    assert!(ended.rejected_effects.is_empty());
    assert_eq!(surface.focus.captured, None);
    assert_no_pointer_capture(&surface);
    assert_eq!(
        ended.diagnostics.route_policy,
        UiInputRoutePolicy::PointerCapture
    );
    assert_eq!(ended.diagnostics.route_trace.target, Some(UiNodeId::new(3)));
    assert_eq!(
        ended.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        ended.diagnostics.route_trace.capture_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(ended.diagnostics.route_steps.len(), 1);
    assert_eq!(
        ended.diagnostics.route_steps[0].phase,
        UiDispatchPhase::Direct
    );
    assert_eq!(
        ended.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(2))
    );
}

#[test]
fn stale_drag_drop_over_event_does_not_emit_rejected_effect_after_session_replaced() {
    let mut surface = route_surface();
    let session_id = UiDragSessionId::new(42);

    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            drag_drop_event(
                UiDragDropInputEventKind::Begin,
                Some(session_id),
                UiPoint::new(20.0, 20.0),
            ),
        )
        .unwrap();

    let stale_over = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            drag_drop_event(
                UiDragDropInputEventKind::Over,
                Some(UiDragSessionId::new(99)),
                UiPoint::new(20.0, 60.0),
            ),
        )
        .unwrap();

    assert_eq!(
        stale_over.reply.disposition,
        UiDispatchDisposition::Unhandled
    );
    assert!(stale_over.applied_effects.is_empty());
    assert!(stale_over.rejected_effects.is_empty());
    assert!(stale_over.host_requests.is_empty());
    assert_eq!(
        surface
            .input
            .drag_drop
            .as_ref()
            .map(|drag| (drag.source, drag.target, drag.session_id)),
        Some((UiNodeId::new(2), UiNodeId::new(2), session_id))
    );
    assert_eq!(surface.focus.captured, Some(UiNodeId::new(2)));
    assert_eq!(
        stale_over.diagnostics.handled_phase.as_deref(),
        Some("drag_drop.stale")
    );
    assert!(stale_over
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "stale_drag_drop_event_ignored"));
    assert_eq!(
        stale_over.diagnostics.route_policy,
        UiInputRoutePolicy::Bubble
    );
    assert_eq!(
        stale_over.diagnostics.route_trace.capture_target,
        Some(UiNodeId::new(2))
    );
}
