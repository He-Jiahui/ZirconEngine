use super::*;
use zircon_runtime_interface::ui::widget::{UiWidgetBehavior, UiWidgetContract};

#[test]
fn unified_pointer_outside_popup_dismiss_reports_hit_route_and_popup_owner_handler() {
    let mut surface = pointer_popup_route_surface();
    assert_popup_stack(&surface, &["root/popup"]);

    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Down, UiPoint::new(170.0, 100.0)),
        )
        .unwrap();
    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            pointer_event(UiPointerEventKind::Up, UiPoint::new(170.0, 100.0)),
        )
        .unwrap();

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(2)));
    assert_eq!(result.reply.effects.len(), 0);
    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(result.diagnostics.handled_phase.as_deref(), Some("pointer"));
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(1)));
    assert_eq!(
        result.diagnostics.route_trace.target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1)]
    );
    assert_eq!(
        result.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(1)]
    );
    assert!(result.diagnostics.route_trace.popup_stack.is_empty());
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
        UiDispatchPhase::Target
    );
    assert_eq!(
        result.diagnostics.route_steps[1].target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_steps[1].disposition,
        UiDispatchDisposition::Passthrough
    );
    assert_eq!(
        result.diagnostics.route_steps[2].phase,
        UiDispatchPhase::DefaultAction
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
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ClosePopup
    );
    assert!(surface.input.popup_stack.is_empty());
    assert_eq!(popup_open(&surface), Some(false));
}

fn pointer_popup_route_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.input.reply_route.popup_pointer"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 180.0, 110.0))
            .with_input_policy(UiInputPolicy::Receive)
            .with_state_flags(root_pointer_state()),
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
                    component: "MenuPopup".to_string(),
                    attributes: [("popup_open".to_string(), toml::Value::Boolean(true))]
                        .into_iter()
                        .collect(),
                    bindings: vec![binding("MenuPopup/ClosePopup", UiEventKind::Click)],
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Popup,
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

fn root_pointer_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        ..UiStateFlags::default()
    }
}

fn assert_popup_stack(surface: &UiSurface, expected: &[&str]) {
    assert_eq!(
        surface
            .input
            .popup_stack
            .iter()
            .map(|popup| popup.popup_id.as_str())
            .collect::<Vec<_>>(),
        expected
    );
}

fn popup_open(surface: &UiSurface) -> Option<bool> {
    surface
        .tree
        .node(UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.attributes.get("popup_open"))
        .and_then(toml::Value::as_bool)
}
