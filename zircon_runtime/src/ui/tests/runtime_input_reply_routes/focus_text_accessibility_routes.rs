use super::*;

#[test]
fn unified_focus_and_capture_dispatch_report_phase_route_steps() {
    let mut surface = route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let keyboard = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            keyboard_event(),
        )
        .unwrap();

    assert_eq!(
        keyboard.diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert_eq!(keyboard.reply.disposition, UiDispatchDisposition::Unhandled);
    assert_eq!(keyboard.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        keyboard.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        keyboard.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(
        keyboard.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(keyboard.diagnostics.route_steps.len(), 4);
    assert_eq!(
        keyboard.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        keyboard.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        keyboard.diagnostics.route_steps[2].phase,
        UiDispatchPhase::Target
    );
    assert_eq!(
        keyboard.diagnostics.route_steps[2].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        keyboard.diagnostics.route_steps[2].disposition,
        UiDispatchDisposition::Unhandled
    );
    assert!(!keyboard.diagnostics.route_steps[2].stopped);
    assert_eq!(
        keyboard.diagnostics.route_steps[3].phase,
        UiDispatchPhase::Bubble
    );
    assert_eq!(
        keyboard.diagnostics.route_steps[3].target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(surface.focus.focused_inputs[0].focused, UiNodeId::new(2));
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Keyboard
    );
    assert_eq!(
        surface.focus.focused_inputs[0].route,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(UiNodeId::new(2))
    );
    assert!(surface.focus.focused_inputs[0].accepted);

    capture_pointer_for_test(&mut surface, UiPointerId::new(7), UiNodeId::new(2));
    let captured = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Move, UiPoint::new(20.0, 60.0)),
        )
        .unwrap();

    assert_eq!(
        captured.diagnostics.route_policy,
        UiInputRoutePolicy::PointerCapture
    );
    assert_eq!(
        captured.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(captured.diagnostics.route_steps.len(), 1);
    assert_eq!(
        captured.diagnostics.route_steps[0].phase,
        UiDispatchPhase::Direct
    );
    assert_eq!(
        captured.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(2))
    );
}

#[test]
fn unified_navigation_dispatch_reports_route_steps_and_focused_input_log() {
    let mut surface = route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            navigation_event(UiNavigationEventKind::Next),
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(3)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.applied_effects.len(), 1);
    assert!(matches!(
        &result.applied_effects[0].effect,
        UiDispatchEffect::SetFocus { target, reason }
            if *target == UiNodeId::new(3)
                && *reason == UiFocusEffectReason::Navigation
    ));
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("navigation")
    );
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
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
    assert_eq!(
        result.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(result.diagnostics.route_steps.len(), 3);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
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
        result.diagnostics.route_steps[2].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(result.diagnostics.route_steps[2].effect_count, 1);
    assert!(result.diagnostics.route_steps[2].stopped);
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(surface.focus.focused_inputs[0].focused, UiNodeId::new(3));
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Navigation
    );
    assert_eq!(
        surface.focus.focused_inputs[0].route,
        vec![UiNodeId::new(3), UiNodeId::new(1)]
    );
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(UiNodeId::new(2))
    );
    assert!(surface.focus.focused_inputs[0].accepted);
}

#[test]
fn unified_text_and_ime_dispatch_report_focus_route_steps_and_focused_input_log() {
    let mut surface = editable_route_surface("Hi", 2);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let text = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            text_event("!"),
        )
        .unwrap();

    assert_eq!(text.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text.diagnostics.route_policy, UiInputRoutePolicy::FocusPath);
    assert_eq!(text.diagnostics.handled_phase.as_deref(), Some("text.edit"));
    assert_eq!(text.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(text.diagnostics.route_trace.target, Some(UiNodeId::new(2)));
    assert_eq!(
        text.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(2)]
    );
    assert_eq!(
        text.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(
        text.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(text.diagnostics.route_steps.len(), 3);
    assert_eq!(
        text.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        text.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        text.diagnostics.route_steps[2].phase,
        UiDispatchPhase::Target
    );
    assert_eq!(
        text.diagnostics.route_steps[2].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        text.diagnostics.route_steps[2].handler,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        text.diagnostics.route_steps[2].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(text.diagnostics.route_steps[2].effect_count, 0);
    assert!(text.diagnostics.route_steps[2].stopped);
    assert_eq!(editable_attr_string(&surface, "value"), "Hi!");
    assert_eq!(text.component_events.len(), 1);
    assert_eq!(
        text.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "value".to_string(),
            value: UiValue::String("Hi!".to_string()),
        }
    );
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Text
    );
    assert_eq!(
        surface.focus.focused_inputs[0].route,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(UiNodeId::new(2))
    );
    assert!(surface.focus.focused_inputs[0].accepted);

    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let ime = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            ime_event(UiImeInputEventKind::Commit, "?"),
        )
        .unwrap();

    assert_eq!(ime.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(ime.diagnostics.route_policy, UiInputRoutePolicy::FocusPath);
    assert_eq!(ime.diagnostics.handled_phase.as_deref(), Some("ime.edit"));
    assert_eq!(ime.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(ime.diagnostics.route_trace.target, Some(UiNodeId::new(2)));
    assert_eq!(
        ime.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(2)]
    );
    assert_eq!(
        ime.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(ime.diagnostics.route_steps.len(), 3);
    assert_eq!(
        ime.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        ime.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        ime.diagnostics.route_steps[2].phase,
        UiDispatchPhase::Target
    );
    assert_eq!(
        ime.diagnostics.route_steps[2].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        ime.diagnostics.route_steps[2].handler,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        ime.diagnostics.route_steps[2].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(ime.diagnostics.route_steps[2].effect_count, 1);
    assert!(ime.diagnostics.route_steps[2].stopped);
    assert_eq!(editable_attr_string(&surface, "value"), "Hi!?");
    assert!(ime.component_events.iter().any(|event| {
        event.event
            == UiComponentEvent::Commit {
                property: "value".to_string(),
                value: UiValue::String("Hi!?".to_string()),
            }
    }));
    assert_eq!(surface.focus.focused_inputs.len(), 2);
    assert_eq!(
        surface.focus.focused_inputs[1].kind,
        UiFocusedInputKind::Ime
    );
    assert_eq!(
        surface.focus.focused_inputs[1].route,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(
        surface.focus.focused_inputs[1].handled_by,
        Some(UiNodeId::new(2))
    );
    assert!(surface.focus.focused_inputs[1].accepted);
}

#[test]
fn captured_pointer_up_preserves_capture_route_trace_after_release() {
    let mut surface = route_surface();
    capture_pointer_for_test(&mut surface, UiPointerId::new(7), UiNodeId::new(2));
    surface.focus.pressed = Some(UiNodeId::new(2));

    let released = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Up, UiPoint::new(20.0, 60.0)),
        )
        .unwrap();

    assert_eq!(surface.focus.captured, None);
    assert_eq!(surface.focus.pressed, None);
    assert_no_pointer_capture(&surface);
    assert_eq!(
        released.diagnostics.route_policy,
        UiInputRoutePolicy::PointerCapture
    );
    assert_eq!(
        released.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        released.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        released.diagnostics.route_trace.capture_target,
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
    assert!(released.reply.effects.iter().any(|effect| matches!(
        effect,
        UiDispatchEffect::ReleasePointerCapture {
            target,
            pointer_id,
            reason
        } if *target == UiNodeId::new(2)
            && *pointer_id == UiPointerId::new(7)
            && *reason == UiPointerCaptureReason::Cancel
    )));
}

#[test]
fn accessibility_activate_dispatch_reports_owner_default_action_route_steps() {
    let mut surface = route_surface();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            accessibility_event(UiNodeId::new(2), UiAccessibilityAction::Activate),
        )
        .unwrap();

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::DefaultAction
    );
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.activate")
    );
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(
        result.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(2)]
    );
    assert_eq!(result.diagnostics.route_steps.len(), 1);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::DefaultAction
    );
    assert_eq!(
        result.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[0].handler,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[0].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(result.diagnostics.route_steps[0].effect_count, 0);
    assert!(result.diagnostics.route_steps[0].stopped);
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.component_events[0].target, UiNodeId::new(2));
}
