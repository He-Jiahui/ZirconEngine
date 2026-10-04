//! 捕获后的取消与释放仍报告旧捕获路由；所有者不匹配拒绝释放，高精度及指针锁请求转交宿主。

use super::*;

mod ownership_lifecycle;

fn owned_pointer_event(
    pointer_id: u64,
    button: UiPointerButton,
    kind: UiPointerEventKind,
    point: UiPoint,
) -> UiInputEvent {
    let mut event = pointer_event(kind, point);
    if let UiInputEvent::Pointer(pointer) = &mut event {
        pointer.metadata.pointer_id = Some(UiPointerId::new(pointer_id));
        pointer.event.button = Some(button);
    }
    event
}

#[test]
fn pointer_button_ownership_is_independent_for_two_pointers_capturing_one_node() {
    let mut surface = route_surface();
    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Down, |_| {
        UiPointerDispatchEffect::capture()
    });
    dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Move, |_| {
        UiPointerDispatchEffect::handled()
    });
    dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Up, |_| {
        UiPointerDispatchEffect::handled()
    });
    for (pointer_id, button) in [
        (11, UiPointerButton::Secondary),
        (12, UiPointerButton::Primary),
    ] {
        surface
            .dispatch_input_event(
                &dispatcher,
                &UiNavigationDispatcher::default(),
                owned_pointer_event(
                    pointer_id,
                    button,
                    UiPointerEventKind::Down,
                    UiPoint::new(20.0, 20.0),
                ),
            )
            .unwrap();
    }
    surface
        .dispatch_input_event(
            &dispatcher,
            &UiNavigationDispatcher::default(),
            owned_pointer_event(
                11,
                UiPointerButton::Primary,
                UiPointerEventKind::Up,
                UiPoint::new(200.0, 200.0),
            ),
        )
        .unwrap();
    assert_pointer_capture(&surface, UiPointerId::new(11), UiNodeId::new(2));
    assert_pointer_capture(&surface, UiPointerId::new(12), UiNodeId::new(2));
    surface
        .dispatch_input_event(
            &dispatcher,
            &UiNavigationDispatcher::default(),
            owned_pointer_event(
                12,
                UiPointerButton::Primary,
                UiPointerEventKind::Up,
                UiPoint::new(200.0, 200.0),
            ),
        )
        .unwrap();
    assert_pointer_capture(&surface, UiPointerId::new(11), UiNodeId::new(2));
    assert_eq!(
        surface.input.pointer_capture_owner(UiPointerId::new(12)),
        None
    );
    let moved = surface
        .dispatch_input_event(
            &dispatcher,
            &UiNavigationDispatcher::default(),
            owned_pointer_event(
                11,
                UiPointerButton::Secondary,
                UiPointerEventKind::Move,
                UiPoint::new(220.0, 220.0),
            ),
        )
        .unwrap();
    assert_eq!(moved.reply.handler, Some(UiNodeId::new(2)));
    surface
        .dispatch_input_event(
            &dispatcher,
            &UiNavigationDispatcher::default(),
            owned_pointer_event(
                11,
                UiPointerButton::Secondary,
                UiPointerEventKind::Cancel,
                UiPoint::new(220.0, 220.0),
            ),
        )
        .unwrap();
    assert_no_pointer_capture(&surface);
}

#[test]
fn pointer_button_ownership_qualifies_reply_capture_and_resets_after_explicit_release() {
    let mut surface = route_surface();
    let pointer_id = UiPointerId::new(7);
    let capture = |button| {
        owned_pointer_event(
            7,
            button,
            UiPointerEventKind::Down,
            UiPoint::new(20.0, 20.0),
        )
    };
    let reply = || {
        UiDispatchReply::handled().with_effect(UiDispatchEffect::CapturePointer {
            target: UiNodeId::new(2),
            pointer_id,
            reason: UiPointerCaptureReason::Press,
        })
    };
    assert!(surface
        .apply_dispatch_reply(capture(UiPointerButton::Secondary), reply())
        .rejected_effects
        .is_empty());
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            owned_pointer_event(
                7,
                UiPointerButton::Primary,
                UiPointerEventKind::Up,
                UiPoint::new(200.0, 200.0),
            ),
        )
        .unwrap();
    assert_pointer_capture(&surface, pointer_id, UiNodeId::new(2));
    let released = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::ReleasePointerCapture {
            target: UiNodeId::new(2),
            pointer_id,
            reason: UiPointerCaptureReason::Cancel,
        }),
    );
    assert!(released.rejected_effects.is_empty());
    assert_no_pointer_capture(&surface);
    assert!(surface
        .apply_dispatch_reply(capture(UiPointerButton::Primary), reply())
        .rejected_effects
        .is_empty());
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            owned_pointer_event(
                7,
                UiPointerButton::Secondary,
                UiPointerEventKind::Up,
                UiPoint::new(200.0, 200.0),
            ),
        )
        .unwrap();
    assert_pointer_capture(&surface, pointer_id, UiNodeId::new(2));
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            owned_pointer_event(
                7,
                UiPointerButton::Primary,
                UiPointerEventKind::Up,
                UiPoint::new(200.0, 200.0),
            ),
        )
        .unwrap();
    assert_no_pointer_capture(&surface);
}

#[test]
fn pointer_button_ownership_text_drag_survives_secondary_release_without_context_popup() {
    let mut surface = editable_route_surface("drag selection across words", 0);
    let offset = |surface: &UiSurface, property: &str| {
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .template_metadata
            .as_ref()
            .unwrap()
            .attributes
            .get(property)
            .and_then(toml::Value::as_integer)
            .unwrap()
    };
    let dispatch = |surface: &mut UiSurface, button, kind, point| {
        surface
            .dispatch_input_event(
                &UiPointerDispatcher::default(),
                &UiNavigationDispatcher::default(),
                owned_pointer_event(7, button, kind, point),
            )
            .unwrap()
    };
    dispatch(
        &mut surface,
        UiPointerButton::Primary,
        UiPointerEventKind::Down,
        UiPoint::new(20.0, 20.0),
    );
    assert_pointer_capture(&surface, UiPointerId::new(7), UiNodeId::new(2));
    let anchor = offset(&surface, "caret_offset");
    dispatch(
        &mut surface,
        UiPointerButton::Secondary,
        UiPointerEventKind::Down,
        UiPoint::new(25.0, 20.0),
    );
    let foreign = dispatch(
        &mut surface,
        UiPointerButton::Secondary,
        UiPointerEventKind::Up,
        UiPoint::new(25.0, 20.0),
    );
    assert_pointer_capture(&surface, UiPointerId::new(7), UiNodeId::new(2));
    assert!(surface.input.popup_stack.is_empty());
    assert_eq!(offset(&surface, "caret_offset"), anchor);
    assert!(!foreign
        .reply
        .effects
        .iter()
        .any(|effect| matches!(effect, UiDispatchEffect::ReleasePointerCapture { .. })));
    let moved = dispatch(
        &mut surface,
        UiPointerButton::Primary,
        UiPointerEventKind::Move,
        UiPoint::new(140.0, 20.0),
    );
    assert_eq!(moved.reply.handler, Some(UiNodeId::new(2)));
    assert!(surface.input.pointer_drags.contains_key(&UiNodeId::new(2)));
    let focus = offset(&surface, "selection_focus");
    assert_eq!(offset(&surface, "selection_anchor"), anchor);
    assert_eq!(offset(&surface, "caret_offset"), focus);
    assert!(
        focus > anchor,
        "outside movement must extend the real text selection"
    );
    dispatch(
        &mut surface,
        UiPointerButton::Primary,
        UiPointerEventKind::Up,
        UiPoint::new(140.0, 20.0),
    );
    assert_no_pointer_capture(&surface);
    assert!(!surface.input.pointer_drags.contains_key(&UiNodeId::new(2)));
    assert_eq!(offset(&surface, "selection_anchor"), anchor);
    assert_eq!(offset(&surface, "selection_focus"), focus);
}

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
    capture_pointer_for_test(&mut surface, UiPointerId::new(7), UiNodeId::new(2));
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
    assert_no_pointer_capture(&surface);
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
    assert_pointer_capture(&surface, first_pointer, UiNodeId::new(2));
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
    assert_pointer_capture(&surface, pointer_id, UiNodeId::new(2));

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
    assert_no_pointer_capture(&surface);
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
    assert_no_pointer_capture(&surface);
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
