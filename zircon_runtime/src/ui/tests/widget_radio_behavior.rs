use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
    tree::UiRuntimeTreeAccessExt,
};
use zircon_runtime_interface::ui::{
    accessibility::{UiA11yCheckedState, UiA11yRole, UiAccessibilityAction},
    binding::{UiBindingSourceKind, UiEventKind},
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiDispatchDisposition, UiInputEvent, UiInputEventMetadata, UiInputSequence,
        UiInputTimestamp, UiKeyboardInputEvent, UiKeyboardInputState, UiPointerEvent,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiFrame, UiPoint},
    surface::{UiPointerButton, UiPointerEventKind},
    template::UiBindingRef,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract},
};

#[test]
fn radio_button_click_updates_group_value_and_unchecks_siblings() {
    let mut surface = radio_group_surface();
    let result = click_node(&mut surface, UiPoint::new(20.0, 54.0));

    assert_checked(&surface, UiNodeId::new(3), false);
    assert_checked(&surface, UiNodeId::new(4), true);
    assert_group_value(&surface, "two");
    assert!(result.component_events.iter().any(|event| {
        event.node_id == UiNodeId::new(4)
            && matches!(
                &event.envelope.event,
                UiComponentEvent::ValueChanged { property, value }
                    if property == "checked" && value == &UiValue::Bool(true)
            )
    }));
    assert!(result.component_events.iter().any(|event| {
        event.node_id == UiNodeId::new(2)
            && matches!(
                &event.envelope.event,
                UiComponentEvent::ValueChanged { property, value }
                    if property == "choice" && value == &UiValue::String("two".to_string())
            )
    }));
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn checked_radio_button_click_is_ignored() {
    let mut surface = radio_group_surface();
    let result = click_node(&mut surface, UiPoint::new(20.0, 22.0));

    assert!(result.component_events.is_empty());
    assert!(result.binding_reports.is_empty());
    assert_checked(&surface, UiNodeId::new(3), true);
    assert_checked(&surface, UiNodeId::new(4), false);
    assert_group_value(&surface, "one");
}

#[test]
fn radio_button_under_disabled_group_is_ignored() {
    let mut surface = radio_group_surface();
    surface
        .tree
        .node_mut(UiNodeId::new(2))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .widget
        .disabled = true;

    let result = click_node(&mut surface, UiPoint::new(20.0, 54.0));

    assert!(result.component_events.is_empty());
    assert!(result.binding_reports.is_empty());
    assert_checked(&surface, UiNodeId::new(3), true);
    assert_checked(&surface, UiNodeId::new(4), false);
    assert_group_value(&surface, "one");
}

#[test]
fn radio_button_keyboard_activation_updates_group_value() {
    let mut surface = radio_group_surface();
    surface.focus_node(UiNodeId::new(4)).unwrap();

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
    assert_checked(&surface, UiNodeId::new(3), false);
    assert_checked(&surface, UiNodeId::new(4), true);
    assert_group_value(&surface, "two");
    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(4)
            && matches!(
                &event.event,
                UiComponentEvent::ValueChanged { property, value }
                    if property == "checked" && value == &UiValue::Bool(true)
            )
    }));
    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && matches!(
                &event.event,
                UiComponentEvent::ValueChanged { property, value }
                    if property == "choice" && value == &UiValue::String("two".to_string())
            )
    }));
    assert_widget_binding_report(&result.binding_reports);
}

fn assert_widget_binding_report(
    reports: &[zircon_runtime_interface::ui::binding::UiBindingUpdateReport],
) {
    assert!(!reports.is_empty());
    assert!(reports.iter().any(|report| {
        report
            .updates
            .first()
            .is_some_and(|update| update.source.kind == UiBindingSourceKind::WidgetBehavior)
    }));
}

#[test]
fn radio_behavior_projects_accessibility_roles_and_activate_action() {
    let surface = radio_group_surface();
    let snapshot = surface.accessibility_snapshot();
    let group = snapshot.node(UiNodeId::new(2)).unwrap();
    let first = snapshot.node(UiNodeId::new(3)).unwrap();
    let second = snapshot.node(UiNodeId::new(4)).unwrap();

    assert_eq!(group.role, UiA11yRole::RadioGroup);
    assert_eq!(group.actions, Vec::<UiAccessibilityAction>::new());
    assert_eq!(first.role, UiA11yRole::Radio);
    assert_eq!(first.state.checked, Some(UiA11yCheckedState::True));
    assert!(first.actions.contains(&UiAccessibilityAction::Activate));
    assert_eq!(second.role, UiA11yRole::Radio);
    assert_eq!(second.state.checked, Some(UiA11yCheckedState::False));
    assert!(second.actions.contains(&UiAccessibilityAction::Activate));
}

fn click_node(
    surface: &mut UiSurface,
    point: UiPoint,
) -> zircon_runtime_interface::ui::dispatch::UiPointerDispatchResult {
    let dispatcher = UiPointerDispatcher::default();
    surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap()
}

fn radio_group_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.widget.radio.behavior"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 180.0, 110.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/group"))
                .with_frame(UiFrame::new(8.0, 8.0, 150.0, 86.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(container_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "ChoiceGroup".to_string(),
                    attributes: [("choice".to_string(), toml::Value::String("one".to_string()))]
                        .into_iter()
                        .collect(),
                    bindings: vec![binding("ChoiceGroup/ValueChanged", UiEventKind::Change)],
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::RadioGroup,
                        value_property: Some("choice".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    insert_radio(
        &mut surface,
        UiNodeId::new(3),
        "root/group/one",
        14.0,
        true,
        "one",
    );
    insert_radio(
        &mut surface,
        UiNodeId::new(4),
        "root/group/two",
        46.0,
        false,
        "two",
    );
    surface.rebuild();
    surface
}

fn insert_radio(
    surface: &mut UiSurface,
    node_id: UiNodeId,
    path: &str,
    y: f32,
    checked: bool,
    value: &str,
) {
    surface
        .tree
        .insert_child(
            UiNodeId::new(2),
            UiTreeNode::new(node_id, UiNodePath::new(path))
                .with_frame(UiFrame::new(14.0, y, 96.0, 24.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(focusable_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Choice".to_string(),
                    attributes: [("checked".to_string(), toml::Value::Boolean(checked))]
                        .into_iter()
                        .collect(),
                    bindings: vec![binding("Choice/ValueChanged", UiEventKind::Change)],
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Radio,
                        value: Some(UiValue::String(value.to_string())),
                        checked_property: Some("checked".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
}

fn assert_checked(surface: &UiSurface, node_id: UiNodeId, expected: bool) {
    let metadata = surface
        .tree
        .node(node_id)
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["checked"].as_bool(), Some(expected));
}

fn assert_group_value(surface: &UiSurface, expected: &str) {
    let metadata = surface
        .tree
        .node(UiNodeId::new(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["choice"].as_str(), Some(expected));
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

fn focusable_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: true,
        ..UiStateFlags::default()
    }
}

fn container_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        ..UiStateFlags::default()
    }
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
