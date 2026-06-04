use super::*;

#[test]
fn unified_keyboard_tab_routes_to_navigation_next_from_focused_path() {
    let mut surface = route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            keyboard_navigation_event(false),
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
    match &result.event {
        UiInputEvent::Keyboard(keyboard) => {
            assert_eq!(keyboard.logical_key, "Tab");
            assert!(!keyboard.metadata.modifiers.shift);
        }
        other => panic!("expected original keyboard input event, got {other:?}"),
    }
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.navigation")
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "keyboard_navigation=Next"));
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
fn unified_keyboard_shift_tab_routes_to_navigation_previous_from_focused_path() {
    let mut surface = route_surface();
    surface.focus_node(UiNodeId::new(3)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            keyboard_navigation_event(true),
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert!(matches!(
        &result.applied_effects[0].effect,
        UiDispatchEffect::SetFocus { target, reason }
            if *target == UiNodeId::new(2)
                && *reason == UiFocusEffectReason::Navigation
    ));
    match &result.event {
        UiInputEvent::Keyboard(keyboard) => {
            assert_eq!(keyboard.logical_key, "Tab");
            assert!(keyboard.metadata.modifiers.shift);
        }
        other => panic!("expected original keyboard input event, got {other:?}"),
    }
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.navigation")
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "keyboard_navigation=Previous"));
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(3)));
    assert_eq!(
        result.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(3), UiNodeId::new(1)]
    );
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(surface.focus.focused_inputs[0].focused, UiNodeId::new(2));
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Navigation
    );
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(UiNodeId::new(3))
    );
    assert!(surface.focus.focused_inputs[0].accepted);
}

#[test]
fn unified_keyboard_arrow_down_routes_to_directional_navigation_from_focused_path() {
    let mut surface = route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            keyboard_navigation_key_event("ArrowDown", 40, None, false),
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(3)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert!(matches!(
        &result.applied_effects[0].effect,
        UiDispatchEffect::SetFocus { target, reason }
            if *target == UiNodeId::new(3)
                && *reason == UiFocusEffectReason::Navigation
    ));
    match &result.event {
        UiInputEvent::Keyboard(keyboard) => {
            assert_eq!(keyboard.logical_key, "ArrowDown");
            assert_eq!(keyboard.key_code, 40);
        }
        other => panic!("expected original keyboard input event, got {other:?}"),
    }
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.navigation")
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "keyboard_navigation=Down"));
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(surface.focus.focused_inputs[0].focused, UiNodeId::new(3));
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Navigation
    );
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(UiNodeId::new(2))
    );
    assert!(surface.focus.focused_inputs[0].accepted);
}

#[test]
fn unified_keyboard_arrow_up_routes_to_directional_navigation_from_focused_path() {
    let mut surface = route_surface();
    surface.focus_node(UiNodeId::new(3)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            keyboard_navigation_key_event("ArrowUp", 38, None, false),
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert!(matches!(
        &result.applied_effects[0].effect,
        UiDispatchEffect::SetFocus { target, reason }
            if *target == UiNodeId::new(2)
                && *reason == UiFocusEffectReason::Navigation
    ));
    match &result.event {
        UiInputEvent::Keyboard(keyboard) => {
            assert_eq!(keyboard.logical_key, "ArrowUp");
            assert_eq!(keyboard.key_code, 38);
        }
        other => panic!("expected original keyboard input event, got {other:?}"),
    }
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.navigation")
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "keyboard_navigation=Up"));
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(3)));
    assert_eq!(
        result.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(3), UiNodeId::new(1)]
    );
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(surface.focus.focused_inputs[0].focused, UiNodeId::new(2));
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Navigation
    );
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(UiNodeId::new(3))
    );
    assert!(surface.focus.focused_inputs[0].accepted);
}

fn keyboard_navigation_event(shift: bool) -> UiInputEvent {
    let mut metadata = input_metadata();
    metadata.modifiers.shift = shift;
    keyboard_navigation_key_event_with_metadata(metadata, "Tab", 9, Some("\t"))
}

fn keyboard_navigation_key_event(
    logical_key: &str,
    key_code: u32,
    text: Option<&str>,
    shift: bool,
) -> UiInputEvent {
    let mut metadata = input_metadata();
    metadata.modifiers.shift = shift;
    keyboard_navigation_key_event_with_metadata(metadata, logical_key, key_code, text)
}

fn keyboard_navigation_key_event_with_metadata(
    metadata: UiInputEventMetadata,
    logical_key: &str,
    key_code: u32,
    text: Option<&str>,
) -> UiInputEvent {
    UiInputEvent::Keyboard(UiKeyboardInputEvent {
        metadata,
        state: UiKeyboardInputState::Pressed,
        key_code,
        scan_code: None,
        physical_key: logical_key.to_string(),
        logical_key: logical_key.to_string(),
        text: text.map(str::to_string),
    })
}
