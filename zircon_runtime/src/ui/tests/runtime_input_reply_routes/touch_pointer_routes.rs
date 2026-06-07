use super::*;

fn assert_touch_notes(result: &UiInputDispatchResult, pointer_id: UiPointerId) {
    assert!(
        result
            .diagnostics
            .notes
            .iter()
            .any(|note| note == "touch_like_pointer"),
        "expected touch_like_pointer note, got {:?}",
        result.diagnostics.notes
    );
    assert!(
        result
            .diagnostics
            .notes
            .iter()
            .any(|note| note == "pointer_source=Touch"),
        "expected pointer_source=Touch note, got {:?}",
        result.diagnostics.notes
    );
    match &result.event {
        UiInputEvent::Pointer(pointer) => {
            assert_eq!(pointer.metadata.pointer_source, UiPointerSource::Touch);
            assert_eq!(pointer.metadata.pointer_id, Some(pointer_id));
        }
        other => panic!("expected normalized pointer event, got {other:?}"),
    }
}

fn dispatch_touch_pointer(
    surface: &mut UiSurface,
    pointer_id: UiPointerId,
    kind: UiPointerEventKind,
    point: UiPoint,
) -> UiInputDispatchResult {
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            touch_pointer_event_with_id(pointer_id, kind, point),
        )
        .expect("touch pointer event should dispatch")
}

#[test]
fn unified_touch_start_move_end_share_pointer_routes_and_preserve_touch_identity() {
    let mut surface = press_release_route_surface();
    let touch_id = UiPointerId::new(11);

    let down = dispatch_touch_pointer(
        &mut surface,
        touch_id,
        UiPointerEventKind::Down,
        UiPoint::new(20.0, 20.0),
    );
    assert_two_node_bubble_handled_at_target(&down);
    assert_touch_notes(&down, touch_id);
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
            .expect("button should exist")
            .state_flags
            .pressed
    );
    assert_eq!(surface.input.captured_pointer_id, None);

    let moved = dispatch_touch_pointer(
        &mut surface,
        touch_id,
        UiPointerEventKind::Move,
        UiPoint::new(20.0, 60.0),
    );
    assert_eq!(moved.diagnostics.route_policy, UiInputRoutePolicy::Direct);
    assert_eq!(moved.diagnostics.route_target, Some(UiNodeId::new(3)));
    assert_eq!(
        moved.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(3))
    );
    assert_eq!(moved.diagnostics.route_trace.target, Some(UiNodeId::new(3)));
    assert_eq!(
        moved.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(3)]
    );
    assert_eq!(
        moved.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(3), UiNodeId::new(1)]
    );
    assert_eq!(moved.diagnostics.route_steps.len(), 1);
    assert_eq!(
        moved.diagnostics.route_steps[0].phase,
        UiDispatchPhase::Direct
    );
    assert_eq!(
        moved.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(3))
    );
    assert_touch_notes(&moved, touch_id);
    assert_eq!(
        surface.focus.hovered,
        vec![UiNodeId::new(3), UiNodeId::new(1)]
    );
    assert_eq!(surface.focus.pressed, Some(UiNodeId::new(2)));
    assert!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .expect("button should exist")
            .state_flags
            .pressed
    );

    let up = dispatch_touch_pointer(
        &mut surface,
        touch_id,
        UiPointerEventKind::Up,
        UiPoint::new(20.0, 20.0),
    );
    assert_two_node_bubble_handled_at_target(&up);
    assert_touch_notes(&up, touch_id);
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
            value: UiValue::Bool(true)
        }
    );
    assert_eq!(surface.focus.pressed, None);
    assert!(
        !surface
            .tree
            .node(UiNodeId::new(2))
            .expect("button should exist")
            .state_flags
            .pressed
    );
}

#[test]
fn unified_touch_cancel_routes_to_capture_and_releases_pointer_capture() {
    let mut surface = press_release_route_surface();
    let pressed = dispatch_touch_pointer(
        &mut surface,
        UiPointerId::new(7),
        UiPointerEventKind::Down,
        UiPoint::new(20.0, 20.0),
    );
    assert_two_node_bubble_handled_at_target(&pressed);
    surface.focus.captured = Some(UiNodeId::new(2));
    surface.input.captured_pointer_id = Some(UiPointerId::new(7));
    assert_eq!(surface.focus.pressed, Some(UiNodeId::new(2)));

    let result = dispatch_touch_pointer(
        &mut surface,
        UiPointerId::new(7),
        UiPointerEventKind::Cancel,
        UiPoint::new(200.0, 200.0),
    );

    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::PointerCapture
    );
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_trace.capture_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.direct_target,
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
    assert_eq!(result.diagnostics.route_steps.len(), 1);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::Direct
    );
    assert_eq!(
        result.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[0].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(
        result.diagnostics.route_steps[0].handler,
        Some(UiNodeId::new(2))
    );
    assert!(result.diagnostics.route_steps[0].stopped);
    assert_eq!(result.diagnostics.handled_phase, None);
    assert_touch_notes(&result, UiPointerId::new(7));
    assert!(result.reply.effects.iter().any(|effect| matches!(
        effect,
        UiDispatchEffect::ReleasePointerCapture {
            target,
            pointer_id,
            reason,
        } if *target == UiNodeId::new(2)
            && *pointer_id == UiPointerId::new(7)
            && *reason == UiPointerCaptureReason::Cancel
    )));
    assert!(!result.component_events.iter().any(|event| matches!(
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
fn unified_touch_move_with_different_pointer_id_bypasses_existing_capture() {
    let mut surface = press_release_route_surface();
    surface.focus.captured = Some(UiNodeId::new(2));
    surface.input.captured_pointer_id = Some(UiPointerId::new(4));

    let moved = dispatch_touch_pointer(
        &mut surface,
        UiPointerId::new(9),
        UiPointerEventKind::Move,
        UiPoint::new(20.0, 60.0),
    );

    assert_eq!(moved.diagnostics.route_policy, UiInputRoutePolicy::Direct);
    assert_eq!(moved.diagnostics.route_target, Some(UiNodeId::new(3)));
    assert_eq!(
        moved.diagnostics.route_trace.capture_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        moved.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(3))
    );
    assert_eq!(moved.diagnostics.route_steps.len(), 1);
    assert_eq!(
        moved.diagnostics.route_steps[0].phase,
        UiDispatchPhase::Direct
    );
    assert_eq!(
        moved.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(3))
    );
    assert!(moved.reply.effects.is_empty());
    assert_touch_notes(&moved, UiPointerId::new(9));
    assert_eq!(surface.focus.captured, Some(UiNodeId::new(2)));
    assert_eq!(surface.input.captured_pointer_id, Some(UiPointerId::new(4)));

    let released = dispatch_touch_pointer(
        &mut surface,
        UiPointerId::new(4),
        UiPointerEventKind::Cancel,
        UiPoint::new(20.0, 20.0),
    );

    assert_eq!(
        released.diagnostics.route_policy,
        UiInputRoutePolicy::PointerCapture
    );
    assert!(released.reply.effects.iter().any(|effect| matches!(
        effect,
        UiDispatchEffect::ReleasePointerCapture {
            target,
            pointer_id,
            reason,
        } if *target == UiNodeId::new(2)
            && *pointer_id == UiPointerId::new(4)
            && *reason == UiPointerCaptureReason::Cancel
    )));
    assert_touch_notes(&released, UiPointerId::new(4));
    assert_eq!(surface.focus.captured, None);
    assert_eq!(surface.input.captured_pointer_id, None);
}

#[test]
fn unified_touch_pointer_capture_is_indexed_by_pointer_id() {
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
    assert_eq!(surface.focus.captured, Some(UiNodeId::new(3)));

    let first_move = dispatch_touch_pointer(
        &mut surface,
        first_pointer,
        UiPointerEventKind::Move,
        UiPoint::new(200.0, 200.0),
    );
    assert_eq!(
        first_move.diagnostics.route_policy,
        UiInputRoutePolicy::PointerCapture
    );
    assert_eq!(
        first_move.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        first_move.diagnostics.route_trace.capture_target,
        Some(UiNodeId::new(2))
    );
    assert_touch_notes(&first_move, first_pointer);

    let second_move = dispatch_touch_pointer(
        &mut surface,
        second_pointer,
        UiPointerEventKind::Move,
        UiPoint::new(200.0, 200.0),
    );
    assert_eq!(
        second_move.diagnostics.route_policy,
        UiInputRoutePolicy::PointerCapture
    );
    assert_eq!(
        second_move.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(3))
    );
    assert_eq!(
        second_move.diagnostics.route_trace.capture_target,
        Some(UiNodeId::new(3))
    );
    assert_touch_notes(&second_move, second_pointer);

    let first_cancel = dispatch_touch_pointer(
        &mut surface,
        first_pointer,
        UiPointerEventKind::Cancel,
        UiPoint::new(200.0, 200.0),
    );
    assert_eq!(
        first_cancel.diagnostics.route_policy,
        UiInputRoutePolicy::PointerCapture
    );
    assert!(first_cancel.reply.effects.iter().any(|effect| matches!(
        effect,
        UiDispatchEffect::ReleasePointerCapture {
            target,
            pointer_id,
            reason,
        } if *target == UiNodeId::new(2)
            && *pointer_id == first_pointer
            && *reason == UiPointerCaptureReason::Cancel
    )));
    assert_eq!(surface.input.pointer_capture_owner(first_pointer), None);
    assert_eq!(
        surface.input.pointer_capture_owner(second_pointer),
        Some(UiNodeId::new(3))
    );
    assert_eq!(surface.focus.captured, Some(UiNodeId::new(3)));
    assert_eq!(surface.input.captured_pointer_id, Some(second_pointer));
    assert_touch_notes(&first_cancel, first_pointer);

    let second_cancel = dispatch_touch_pointer(
        &mut surface,
        second_pointer,
        UiPointerEventKind::Cancel,
        UiPoint::new(200.0, 200.0),
    );
    assert_eq!(
        second_cancel.diagnostics.route_policy,
        UiInputRoutePolicy::PointerCapture
    );
    assert_eq!(surface.input.pointer_capture_owner(second_pointer), None);
    assert_eq!(surface.focus.captured, None);
    assert_eq!(surface.input.captured_pointer_id, None);
    assert_touch_notes(&second_cancel, second_pointer);
}
