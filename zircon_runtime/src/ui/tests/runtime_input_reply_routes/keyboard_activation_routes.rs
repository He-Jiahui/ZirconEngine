use super::*;

#[test]
fn unified_keyboard_default_activation_reports_focus_route_steps_and_component_event() {
    let mut surface = keyboard_activation_route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            keyboard_activation_event("Enter", 13),
        )
        .unwrap();

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(2)));
    assert_eq!(result.reply.effects.len(), 0);
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.widget")
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
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "focused_route_len=2"));
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::Commit {
            property: "activated".to_string(),
            value: UiValue::Bool(true),
        }
    );
    assert_eq!(surface.focus.focused_inputs.len(), 1);
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
}

#[test]
fn unified_keyboard_virtual_accept_routes_to_default_activation_from_focused_path() {
    let mut surface = keyboard_activation_route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            keyboard_activation_event("Virtual_Accept", 0),
        )
        .unwrap();

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(2)));
    match &result.event {
        UiInputEvent::Keyboard(keyboard) => {
            assert_eq!(keyboard.logical_key, "Virtual_Accept");
            assert_eq!(keyboard.key_code, 0);
        }
        other => panic!("expected original keyboard input event, got {other:?}"),
    }
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.widget")
    );
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(
        result.diagnostics.route_steps[2].handler,
        Some(UiNodeId::new(2))
    );
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::Commit {
            property: "activated".to_string(),
            value: UiValue::Bool(true),
        }
    );
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Keyboard
    );
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(UiNodeId::new(2))
    );
    assert!(surface.focus.focused_inputs[0].accepted);
}

#[test]
fn modified_virtual_accept_still_routes_to_default_activation() {
    let mut surface = keyboard_activation_route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            modified_keyboard_activation_event("Virtual_Accept", 0),
        )
        .unwrap();

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.widget")
    );
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_steps[2].handler,
        Some(UiNodeId::new(2))
    );
    match &result.event {
        UiInputEvent::Keyboard(keyboard) => {
            assert_eq!(keyboard.logical_key, "Virtual_Accept");
            assert!(keyboard.metadata.modifiers.shift);
        }
        other => panic!("expected original keyboard input event, got {other:?}"),
    }
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::Commit {
            property: "activated".to_string(),
            value: UiValue::Bool(true),
        }
    );
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(UiNodeId::new(2))
    );
    assert!(surface.focus.focused_inputs[0].accepted);
}

fn keyboard_activation_route_surface() -> UiSurface {
    let mut surface = route_surface();
    let target = surface.tree.nodes.get_mut(&UiNodeId::new(2)).unwrap();
    target.template_metadata = Some(UiTemplateNodeMetadata {
        component: "MaterialButton".to_string(),
        control_id: Some("KeyboardButton".to_string()),
        bindings: vec![binding("KeyboardButton/Activate", UiEventKind::Click)],
        ..Default::default()
    });
    surface.rebuild();
    surface
}

fn keyboard_activation_event(logical_key: &str, key_code: u32) -> UiInputEvent {
    keyboard_activation_event_with_metadata(input_metadata(), logical_key, key_code)
}

fn modified_keyboard_activation_event(logical_key: &str, key_code: u32) -> UiInputEvent {
    let mut metadata = input_metadata();
    metadata.modifiers.shift = true;
    keyboard_activation_event_with_metadata(metadata, logical_key, key_code)
}

fn keyboard_activation_event_with_metadata(
    metadata: UiInputEventMetadata,
    logical_key: &str,
    key_code: u32,
) -> UiInputEvent {
    UiInputEvent::Keyboard(UiKeyboardInputEvent {
        metadata,
        state: UiKeyboardInputState::Pressed,
        key_code,
        scan_code: None,
        physical_key: logical_key.to_string(),
        logical_key: logical_key.to_string(),
        text: None,
    })
}
