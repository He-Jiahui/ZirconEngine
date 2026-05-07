use crate::ui::{dispatch::UiPointerDispatcher, surface::UiSurface, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::{UiComponentEvent, UiValue},
    dispatch::{UiPointerComponentEventReason, UiPointerEvent},
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiFrame, UiPoint},
    surface::{UiPointerButton, UiPointerEventKind},
    template::UiBindingRef,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn primary_double_click_emits_double_click_binding_from_shared_route() {
    let mut surface = bound_button_surface(vec![binding(
        "Showcase/ButtonDoubleClick",
        UiEventKind::DoubleClick,
    )]);
    let dispatcher = UiPointerDispatcher::default();

    surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    let result = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary)
                .with_click_count(2),
        )
        .unwrap();

    assert_eq!(result.route.click_target, Some(UiNodeId::new(2)));
    assert_eq!(result.component_events.len(), 1);
    let event = &result.component_events[0];
    assert_eq!(event.node_id, UiNodeId::new(2));
    assert_eq!(event.binding_id, "Showcase/ButtonDoubleClick");
    assert_eq!(event.event_kind, UiEventKind::DoubleClick);
    assert_eq!(
        event.reason,
        UiPointerComponentEventReason::DefaultDoubleClick
    );
    assert_eq!(
        event.envelope.event,
        UiComponentEvent::Commit {
            property: "double_activated".to_string(),
            value: UiValue::Bool(true),
        }
    );
}

fn bound_button_surface(bindings: Vec<UiBindingRef>) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.pointer.clicks"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 100.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/button"))
                .with_frame(UiFrame::new(10.0, 10.0, 80.0, 30.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "MaterialButton".to_string(),
                    control_id: Some("MaterialButton".to_string()),
                    bindings,
                    ..Default::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn binding(id: &str, event: UiEventKind) -> UiBindingRef {
    UiBindingRef {
        id: id.to_string(),
        event,
        route: Some(id.replace('/', ".")),
        action: None,
        targets: Vec::new(),
    }
}

fn pointer_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: true,
        pressed: false,
        checked: false,
        dirty: false,
    }
}
