use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
    tree::UiRuntimeTreeAccessExt,
};
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiDispatchDisposition, UiInputEvent, UiInputEventMetadata, UiInputSequence,
        UiInputTimestamp, UiKeyboardInputEvent, UiKeyboardInputState,
        UiPointerComponentEventReason, UiPointerEvent,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiFrame, UiPoint},
    surface::{UiPointerButton, UiPointerEventKind},
    template::UiBindingRef,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract},
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

#[test]
fn authored_toggle_behavior_uses_widget_contract_instead_of_component_name() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.pointer.widget.behavior"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 100.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/favorite"))
                .with_frame(UiFrame::new(10.0, 10.0, 80.0, 30.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "FavoritePill".to_string(),
                    control_id: Some("FavoritePill".to_string()),
                    attributes: [("selected".to_string(), toml::Value::Boolean(false))]
                        .into_iter()
                        .collect(),
                    bindings: vec![binding("FavoritePill/ValueChanged", UiEventKind::Change)],
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Toggle,
                        checked_property: Some("selected".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..Default::default()
                }),
        )
        .unwrap();
    surface.rebuild();

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
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    let metadata = surface
        .tree
        .node(UiNodeId::new(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["selected"].as_bool(), Some(true));
    assert!(result.component_events.iter().any(|event| {
        event.node_id == UiNodeId::new(2)
            && matches!(
                &event.envelope.event,
                UiComponentEvent::ValueChanged { property, value }
                    if property == "selected" && value == &UiValue::Bool(true)
            )
    }));
}

#[test]
fn authored_toggle_behavior_uses_widget_contract_for_keyboard_activation() {
    let mut surface = authored_toggle_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Keyboard(keyboard_pressed("Space", 32)),
        )
        .unwrap();

    let metadata = surface
        .tree
        .node(UiNodeId::new(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["selected"].as_bool(), Some(true));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.widget")
    );
    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && matches!(
                &event.event,
                UiComponentEvent::ValueChanged { property, value }
                    if property == "selected" && value == &UiValue::Bool(true)
            )
    }));
}

#[test]
fn authored_button_behavior_uses_widget_contract_for_keyboard_activation() {
    let mut surface =
        bound_button_surface(vec![binding("MaterialButton/Activate", UiEventKind::Click)]);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Keyboard(keyboard_pressed("Enter", 13)),
        )
        .unwrap();

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.widget")
    );
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::Commit {
            property: "activated".to_string(),
            value: UiValue::Bool(true),
        }
    );
}

#[test]
fn runtime_disabled_button_suppresses_pointer_press_and_activation() {
    let mut surface = bound_button_surface(vec![
        binding("MaterialButton/Press", UiEventKind::Press),
        binding("MaterialButton/Activate", UiEventKind::Click),
        binding("MaterialButton/DoubleActivate", UiEventKind::DoubleClick),
    ]);
    surface
        .component_states
        .set_value(UiNodeId::new(2), "disabled", UiValue::Bool(true));
    let dispatcher = UiPointerDispatcher::default();

    let down = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    assert_eq!(down.route.target, Some(UiNodeId::new(2)));
    assert!(down.component_events.is_empty());
    assert!(
        !surface
            .component_states
            .get(UiNodeId::new(2))
            .unwrap()
            .flags
            .pressed
    );

    let up = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary)
                .with_click_count(2),
        )
        .unwrap();
    assert_eq!(up.route.click_target, Some(UiNodeId::new(2)));
    assert!(up.component_events.is_empty());
}

#[test]
fn runtime_disabled_button_suppresses_keyboard_activation() {
    let mut surface =
        bound_button_surface(vec![binding("MaterialButton/Activate", UiEventKind::Click)]);
    surface
        .component_states
        .set_value(UiNodeId::new(2), "disabled", UiValue::Bool(true));
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Keyboard(keyboard_pressed("Enter", 13)),
        )
        .unwrap();

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert!(result.component_events.is_empty());
}

#[test]
fn runtime_disabled_toggle_suppresses_pointer_and_keyboard_mutation() {
    let mut surface = authored_toggle_surface();
    surface
        .component_states
        .set_value(UiNodeId::new(2), "disabled", UiValue::Bool(true));
    let dispatcher = UiPointerDispatcher::default();

    surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    let pointer_result = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    let metadata = surface
        .tree
        .node(UiNodeId::new(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["selected"].as_bool(), Some(false));
    assert!(pointer_result.component_events.is_empty());

    surface.focus_node(UiNodeId::new(2)).unwrap();
    let keyboard_result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Keyboard(keyboard_pressed("Space", 32)),
        )
        .unwrap();
    let metadata = surface
        .tree
        .node(UiNodeId::new(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["selected"].as_bool(), Some(false));
    assert_eq!(
        keyboard_result.reply.disposition,
        UiDispatchDisposition::Unhandled
    );
    assert!(keyboard_result.component_events.is_empty());
}

fn authored_toggle_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.widget.behavior"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 100.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/favorite"))
                .with_frame(UiFrame::new(10.0, 10.0, 80.0, 30.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "FavoritePill".to_string(),
                    control_id: Some("FavoritePill".to_string()),
                    attributes: [("selected".to_string(), toml::Value::Boolean(false))]
                        .into_iter()
                        .collect(),
                    bindings: vec![binding("FavoritePill/ValueChanged", UiEventKind::Change)],
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Toggle,
                        checked_property: Some("selected".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..Default::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface
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
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Button,
                        ..UiWidgetContract::default()
                    },
                    ..Default::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn keyboard_pressed(logical_key: &str, key_code: u32) -> UiKeyboardInputEvent {
    UiKeyboardInputEvent {
        metadata: UiInputEventMetadata::new(
            UiInputTimestamp::from_micros(20),
            UiInputSequence::new(2),
        ),
        state: UiKeyboardInputState::Pressed,
        key_code,
        scan_code: None,
        physical_key: logical_key.to_string(),
        logical_key: logical_key.to_string(),
        text: None,
    }
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
