use super::*;
use zircon_runtime_interface::ui::widget::{UiWidgetBehavior, UiWidgetContract};

#[test]
fn unified_keyboard_escape_popup_dismiss_reports_focus_route_steps_and_close_event() {
    let mut surface = keyboard_popup_route_surface();
    assert_eq!(
        surface
            .input
            .popup_stack
            .iter()
            .map(|popup| popup.popup_id.as_str())
            .collect::<Vec<_>>(),
        vec!["root/popup"]
    );
    surface.focus_node(UiNodeId::new(3)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            keyboard_popup_event("Escape", 27),
        )
        .unwrap();

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(3)));
    assert_eq!(result.reply.effects.len(), 0);
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.popup_dismiss")
    );
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(3)));
    assert_eq!(
        result.diagnostics.route_trace.target,
        Some(UiNodeId::new(3))
    );
    assert_eq!(
        result.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(2), UiNodeId::new(3)]
    );
    assert_eq!(
        result.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(3), UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(
        result.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(3), UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert!(result.diagnostics.route_trace.popup_stack.is_empty());
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
        Some(UiNodeId::new(3))
    );
    assert_eq!(
        result.diagnostics.route_steps[3].phase,
        UiDispatchPhase::Target
    );
    assert_eq!(
        result.diagnostics.route_steps[3].target,
        Some(UiNodeId::new(3))
    );
    assert_eq!(
        result.diagnostics.route_steps[3].handler,
        Some(UiNodeId::new(3))
    );
    assert_eq!(
        result.diagnostics.route_steps[3].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(result.diagnostics.route_steps[3].effect_count, 0);
    assert!(result.diagnostics.route_steps[3].stopped);
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "focused_route_len=3"));
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ClosePopup
    );
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Keyboard
    );
    assert_eq!(
        surface.focus.focused_inputs[0].route,
        vec![UiNodeId::new(3), UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(UiNodeId::new(3))
    );
    assert!(surface.focus.focused_inputs[0].accepted);
    assert!(surface.input.popup_stack.is_empty());
    assert_eq!(popup_open(&surface), Some(false));
}

#[test]
fn unified_keyboard_escape_context_menu_dismisses_editor_popup_shell() {
    let mut surface = component_keyboard_popup_route_surface("ContextMenu");

    assert_escape_closes_component_popup(&mut surface, "ContextMenu");
}

#[test]
fn unified_keyboard_escape_dropdown_popup_dismisses_selection_popup_shell() {
    let mut surface = component_keyboard_popup_route_surface("DropdownPopup");

    assert_escape_closes_component_popup(&mut surface, "DropdownPopup");
}

#[test]
fn unified_keyboard_virtual_back_routes_to_popup_dismiss_from_focused_path() {
    let mut surface = keyboard_popup_route_surface();
    surface.focus_node(UiNodeId::new(3)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            keyboard_popup_event("Virtual_Back", 0),
        )
        .unwrap();

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(3)));
    match &result.event {
        UiInputEvent::Keyboard(keyboard) => {
            assert_eq!(keyboard.logical_key, "Virtual_Back");
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
        Some("keyboard.popup_dismiss")
    );
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(3)));
    assert!(result.diagnostics.route_trace.popup_stack.is_empty());
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ClosePopup
    );
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Keyboard
    );
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(UiNodeId::new(3))
    );
    assert!(surface.focus.focused_inputs[0].accepted);
    assert!(surface.input.popup_stack.is_empty());
    assert_eq!(popup_open(&surface), Some(false));
}

#[test]
fn modified_virtual_back_still_routes_to_popup_dismissal() {
    let mut surface = keyboard_popup_route_surface();
    surface.focus_node(UiNodeId::new(3)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            modified_keyboard_popup_event("Virtual_Back", 0),
        )
        .unwrap();

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(3)));
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.popup_dismiss")
    );
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(3)));
    match &result.event {
        UiInputEvent::Keyboard(keyboard) => {
            assert_eq!(keyboard.logical_key, "Virtual_Back");
            assert!(keyboard.metadata.modifiers.shift);
        }
        other => panic!("expected original keyboard input event, got {other:?}"),
    }
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ClosePopup
    );
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(UiNodeId::new(3))
    );
    assert!(surface.focus.focused_inputs[0].accepted);
    assert_eq!(
        surface
            .input
            .popup_stack
            .iter()
            .map(|popup| popup.popup_id.as_str())
            .collect::<Vec<_>>(),
        Vec::<&str>::new()
    );
    assert_eq!(popup_open(&surface), Some(false));
}

#[test]
fn unified_keyboard_escape_prefers_semantic_cancel_binding_before_popup_dismissal() {
    let mut surface = semantic_popup_cancel_route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            keyboard_popup_event("Escape", 27),
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
        Some("keyboard.component_action")
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "keyboard_component_action=Cancel"));
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::KeyboardAction {
            action: UiComponentKeyboardAction::Cancel,
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
    assert_eq!(popup_open(&surface), Some(true));
}

fn keyboard_popup_route_surface() -> UiSurface {
    component_keyboard_popup_route_surface("MenuPopup")
}

fn component_keyboard_popup_route_surface(component: &str) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.input.reply_route.popup_keyboard"));
    let close_binding = format!("{component}/ClosePopup");
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 180.0, 110.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/popup"))
                .with_frame(UiFrame::new(8.0, 8.0, 140.0, 74.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    ..UiStateFlags::default()
                })
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: component.to_string(),
                    attributes: [
                        ("open".to_string(), toml::Value::Boolean(true)),
                        ("popup_open".to_string(), toml::Value::Boolean(true)),
                    ]
                    .into_iter()
                    .collect(),
                    bindings: vec![binding(close_binding.as_str(), UiEventKind::Click)],
                    widget: UiWidgetContract {
                        open_property: Some("popup_open".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..Default::default()
                }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(2),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/popup/item"))
                .with_frame(UiFrame::new(16.0, 16.0, 100.0, 24.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "CommandItem".to_string(),
                    bindings: vec![binding("CommandItem/Activate", UiEventKind::Click)],
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::MenuItem,
                        ..UiWidgetContract::default()
                    },
                    ..Default::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn assert_escape_closes_component_popup(surface: &mut UiSurface, component: &str) {
    assert_eq!(
        surface
            .input
            .popup_stack
            .iter()
            .map(|popup| popup.popup_id.as_str())
            .collect::<Vec<_>>(),
        vec!["root/popup"]
    );
    assert_eq!(popup_component(surface), Some(component));
    surface.focus_node(UiNodeId::new(3)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            keyboard_popup_event("Escape", 27),
        )
        .unwrap();

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(3)));
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.popup_dismiss")
    );
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ClosePopup
    );
    assert!(surface.input.popup_stack.is_empty());
    assert_eq!(popup_open(surface), Some(false));
}

fn semantic_popup_cancel_route_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.input.reply_route.popup_keyboard_semantic",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 180.0, 110.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/select"))
                .with_frame(UiFrame::new(8.0, 8.0, 140.0, 32.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Dropdown".to_string(),
                    attributes: [("popup_open".to_string(), toml::Value::Boolean(true))]
                        .into_iter()
                        .collect(),
                    bindings: vec![binding("Dropdown/KeyboardAction", UiEventKind::Click)],
                    widget: UiWidgetContract {
                        open_property: Some("popup_open".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..Default::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn keyboard_popup_event(logical_key: &str, key_code: u32) -> UiInputEvent {
    keyboard_popup_event_with_metadata(input_metadata(), logical_key, key_code)
}

fn modified_keyboard_popup_event(logical_key: &str, key_code: u32) -> UiInputEvent {
    let mut metadata = input_metadata();
    metadata.modifiers.shift = true;
    keyboard_popup_event_with_metadata(metadata, logical_key, key_code)
}

fn keyboard_popup_event_with_metadata(
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

fn popup_open(surface: &UiSurface) -> Option<bool> {
    surface
        .tree
        .node(UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.attributes.get("popup_open"))
        .and_then(toml::Value::as_bool)
}

fn popup_component(surface: &UiSurface) -> Option<&str> {
    surface
        .tree
        .node(UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_ref())
        .map(|metadata| metadata.component.as_str())
}
