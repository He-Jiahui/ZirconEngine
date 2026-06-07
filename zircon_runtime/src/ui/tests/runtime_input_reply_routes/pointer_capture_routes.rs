use super::*;

#[test]
fn unified_pointer_cancel_routes_to_capture_and_releases_pointer_capture() {
    let mut surface = press_release_route_surface();
    let pressed = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0)),
        )
        .expect("pointer press should dispatch");
    assert_two_node_bubble_handled_at_target(&pressed);
    surface.focus.captured = Some(UiNodeId::new(2));
    surface.input.captured_pointer_id = Some(UiPointerId::new(7));
    assert_eq!(surface.focus.pressed, Some(UiNodeId::new(2)));

    let canceled = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Cancel, UiPoint::new(200.0, 200.0)),
        )
        .expect("pointer cancel should dispatch");

    assert_eq!(
        canceled.diagnostics.route_policy,
        UiInputRoutePolicy::PointerCapture
    );
    assert_eq!(canceled.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        canceled.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        canceled.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        canceled.diagnostics.route_trace.capture_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(canceled.diagnostics.route_steps.len(), 1);
    assert_eq!(
        canceled.diagnostics.route_steps[0].phase,
        UiDispatchPhase::Direct
    );
    assert_eq!(
        canceled.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        canceled.diagnostics.route_steps[0].handler,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        canceled.diagnostics.route_steps[0].disposition,
        UiDispatchDisposition::Handled
    );
    assert!(canceled.diagnostics.route_steps[0].stopped);
    assert!(canceled.reply.effects.iter().any(|effect| matches!(
        effect,
        UiDispatchEffect::ReleasePointerCapture {
            target,
            pointer_id,
            reason,
        } if *target == UiNodeId::new(2)
            && *pointer_id == UiPointerId::new(7)
            && *reason == UiPointerCaptureReason::Cancel
    )));
    assert!(!canceled.component_events.iter().any(|event| matches!(
        &event.event,
        UiComponentEvent::Commit { property, .. } if property == "activated"
    )));
    assert_eq!(surface.focus.pressed, None);
    assert!(
        !surface
            .tree
            .node(UiNodeId::new(2))
            .expect("button should exist")
            .state_flags
            .pressed
    );
    assert_eq!(surface.focus.captured, None);
    assert_eq!(surface.input.captured_pointer_id, None);
}

#[test]
fn pointer_capture_release_rejects_owner_mismatch_even_when_pointer_id_is_active() {
    let mut surface = press_release_route_surface();
    let first_pointer = UiPointerId::new(11);
    let second_pointer = UiPointerId::new(12);

    let first_capture = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::CapturePointer {
            target: UiNodeId::new(2),
            pointer_id: first_pointer,
            reason: UiPointerCaptureReason::Press,
        }),
    );
    assert!(first_capture.rejected_effects.is_empty());
    let second_capture = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::CapturePointer {
            target: UiNodeId::new(3),
            pointer_id: second_pointer,
            reason: UiPointerCaptureReason::Press,
        }),
    );
    assert!(second_capture.rejected_effects.is_empty());
    assert_eq!(
        surface.input.pointer_capture_owner(first_pointer),
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        surface.input.pointer_capture_owner(second_pointer),
        Some(UiNodeId::new(3))
    );

    surface.focus.captured = surface.input.activate_pointer_capture_for_id(first_pointer);
    let stale_release = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::ReleasePointerCapture {
            target: UiNodeId::new(3),
            pointer_id: first_pointer,
            reason: UiPointerCaptureReason::Cancel,
        }),
    );

    assert!(stale_release.applied_effects.is_empty());
    assert_eq!(stale_release.rejected_effects.len(), 1);
    assert_eq!(
        stale_release.rejected_effects[0].reason,
        "pointer capture belongs to a different or unknown pointer"
    );
    assert_eq!(
        surface.input.pointer_capture_owner(first_pointer),
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        surface.input.pointer_capture_owner(second_pointer),
        Some(UiNodeId::new(3))
    );
    assert_eq!(surface.focus.captured, Some(UiNodeId::new(2)));
    assert_eq!(surface.input.captured_pointer_id, Some(first_pointer));
}

#[test]
fn direct_pointer_reply_release_preserves_capture_route_trace_after_cleanup() {
    let mut surface = press_release_route_surface();
    let pointer_id = UiPointerId::new(7);
    let capture = surface.apply_dispatch_reply(
        pointer_event(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0)),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::CapturePointer {
            target: UiNodeId::new(2),
            pointer_id,
            reason: UiPointerCaptureReason::Press,
        }),
    );
    assert!(capture.rejected_effects.is_empty());
    assert_eq!(surface.focus.captured, Some(UiNodeId::new(2)));
    assert_eq!(surface.input.captured_pointer_id, Some(pointer_id));

    let released = surface.apply_dispatch_reply(
        pointer_event(UiPointerEventKind::Up, UiPoint::new(200.0, 200.0)),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::ReleasePointerCapture {
            target: UiNodeId::new(2),
            pointer_id,
            reason: UiPointerCaptureReason::Cancel,
        }),
    );

    assert!(released.rejected_effects.is_empty());
    assert_eq!(released.applied_effects.len(), 1);
    assert_eq!(surface.focus.captured, None);
    assert_eq!(surface.input.captured_pointer_id, None);
    assert_eq!(
        released.diagnostics.route_policy,
        UiInputRoutePolicy::PointerCapture
    );
    assert_eq!(released.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        released.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        released.diagnostics.route_trace.capture_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        released.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(released.diagnostics.route_steps.len(), 1);
    assert_eq!(
        released.diagnostics.route_steps[0].phase,
        UiDispatchPhase::Direct
    );
    assert_eq!(
        released.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        released.diagnostics.route_steps[0].handler,
        Some(UiNodeId::new(2))
    );
    assert_eq!(released.diagnostics.route_steps[0].effect_count, 1);
    assert!(released.diagnostics.route_steps[0].stopped);
}

#[test]
fn direct_pointer_reply_capture_high_precision_and_lock_emit_host_requests() {
    let mut surface = press_release_route_surface();
    let pointer_id = UiPointerId::new(7);

    let result = surface.apply_dispatch_reply(
        pointer_event(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0)),
        UiDispatchReply::handled()
            .from_handler(UiNodeId::new(2))
            .in_phase(UiDispatchPhase::Target)
            .with_effects([
                UiDispatchEffect::CapturePointer {
                    target: UiNodeId::new(2),
                    pointer_id,
                    reason: UiPointerCaptureReason::Press,
                },
                UiDispatchEffect::UseHighPrecisionPointer {
                    target: UiNodeId::new(2),
                    enabled: true,
                },
                UiDispatchEffect::LockPointer {
                    target: UiNodeId::new(2),
                    policy: UiPointerLockPolicy::RawDelta,
                },
            ]),
    );

    assert!(result.rejected_effects.is_empty());
    assert_eq!(result.applied_effects.len(), 3);
    assert_eq!(surface.focus.captured, Some(UiNodeId::new(2)));
    assert_eq!(surface.input.captured_pointer_id, Some(pointer_id));
    assert_eq!(
        surface.input.pointer_capture_owner(pointer_id),
        Some(UiNodeId::new(2))
    );
    assert_eq!(surface.input.high_precision_owner, Some(UiNodeId::new(2)));
    assert_eq!(surface.input.pointer_lock_owner, Some(UiNodeId::new(2)));
    assert_eq!(
        surface.input.pointer_lock_policy,
        Some(UiPointerLockPolicy::RawDelta)
    );
    assert_eq!(result.host_requests.len(), 2);
    assert_eq!(result.host_requests[0].effect_index, 1);
    assert!(matches!(
        result.host_requests[0].request,
        UiDispatchHostRequestKind::HighPrecisionPointer {
            target,
            enabled: true,
        } if target == UiNodeId::new(2)
    ));
    assert_eq!(result.host_requests[1].effect_index, 2);
    assert!(matches!(
        result.host_requests[1].request,
        UiDispatchHostRequestKind::PointerLock { target, policy }
            if target == UiNodeId::new(2) && policy == UiPointerLockPolicy::RawDelta
    ));
    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_trace.capture_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(result.diagnostics.route_steps.len(), 3);
    assert_eq!(result.diagnostics.route_steps[2].effect_count, 3);
    assert!(result.diagnostics.route_steps[2].stopped);
}

#[test]
fn direct_pointer_reply_release_capture_disables_high_precision_host_mode() {
    let mut surface = press_release_route_surface();
    let pointer_id = UiPointerId::new(7);
    let capture = surface.apply_dispatch_reply(
        pointer_event(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0)),
        UiDispatchReply::handled().with_effects([
            UiDispatchEffect::CapturePointer {
                target: UiNodeId::new(2),
                pointer_id,
                reason: UiPointerCaptureReason::Press,
            },
            UiDispatchEffect::UseHighPrecisionPointer {
                target: UiNodeId::new(2),
                enabled: true,
            },
        ]),
    );
    assert!(capture.rejected_effects.is_empty());
    assert_eq!(surface.input.high_precision_owner, Some(UiNodeId::new(2)));

    let released = surface.apply_dispatch_reply(
        pointer_event(UiPointerEventKind::Up, UiPoint::new(200.0, 200.0)),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::ReleasePointerCapture {
            target: UiNodeId::new(2),
            pointer_id,
            reason: UiPointerCaptureReason::Cancel,
        }),
    );

    assert!(released.rejected_effects.is_empty());
    assert_eq!(released.applied_effects.len(), 1);
    assert_eq!(surface.focus.captured, None);
    assert_eq!(surface.input.captured_pointer_id, None);
    assert_eq!(surface.input.high_precision_owner, None);
    assert_eq!(
        released.diagnostics.route_policy,
        UiInputRoutePolicy::PointerCapture
    );
    assert_eq!(
        released.diagnostics.route_trace.capture_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(released.host_requests.len(), 1);
    assert_eq!(released.host_requests[0].effect_index, 0);
    assert!(matches!(
        released.host_requests[0].request,
        UiDispatchHostRequestKind::HighPrecisionPointer {
            target,
            enabled: false,
        } if target == UiNodeId::new(2)
    ));
    assert!(released.host_requests[0]
        .reason
        .contains("release pointer capture disabled high precision"));
}
