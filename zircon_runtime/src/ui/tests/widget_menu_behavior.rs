use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::{hit_test_surface_frame, hit_test_surface_frame_with_query, UiSurface},
};
use zircon_runtime_interface::ui::{
    binding::{UiBindingSourceKind, UiEventKind},
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiDispatchDisposition, UiDispatchEffect, UiDispatchReply, UiInputEvent,
        UiInputEventMetadata, UiInputSequence, UiInputTimestamp, UiKeyboardInputEvent,
        UiKeyboardInputState, UiPointerEvent,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiFrame, UiPoint, UiSize},
    surface::{UiHitTestQuery, UiPointerButton, UiPointerEventKind, UiVirtualPointerPosition},
    template::UiBindingRef,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiPopupAnchor, UiWidgetBehavior, UiWidgetContract},
};

use super::popup_test_support::assert_popup_node_open;

mod control_anchored_overlays;

#[test]
fn menu_item_pointer_activation_closes_nearest_popup() {
    let mut surface = menu_surface();
    assert_popup_stack(&surface, &["root/popup"]);
    let result = click_menu_item(&mut surface);

    assert_popup_open(&surface, false);
    assert_popup_stack(&surface, &[]);
    assert!(result.component_events.iter().any(|event| {
        event.node_id == UiNodeId::new(3)
            && matches!(
                &event.envelope.event,
                UiComponentEvent::Commit { property, value }
                    if property == "activated" && value == &UiValue::Bool(true)
            )
    }));
    assert!(result.component_events.iter().any(|event| {
        event.node_id == UiNodeId::new(2)
            && matches!(&event.envelope.event, UiComponentEvent::ClosePopup)
    }));
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn menu_item_keyboard_activation_closes_nearest_popup() {
    let mut surface = menu_surface();
    surface.focus_node(UiNodeId::new(3)).unwrap();

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
    assert_popup_open(&surface, false);
    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(3)
            && matches!(
                &event.event,
                UiComponentEvent::Commit { property, value }
                    if property == "activated" && value == &UiValue::Bool(true)
            )
    }));
    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2) && matches!(&event.event, UiComponentEvent::ClosePopup)
    }));
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn menu_item_keyboard_activation_without_item_binding_still_closes_popup() {
    let mut surface = menu_surface();
    surface
        .tree
        .node_mut(UiNodeId::new(3))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .bindings
        .clear();
    surface.focus_node(UiNodeId::new(3)).unwrap();

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
    assert_popup_open(&surface, false);
    assert!(result.component_events.iter().all(|event| {
        !matches!(
            &event.event,
            UiComponentEvent::Commit { property, .. } if property == "activated"
        )
    }));
    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2) && matches!(&event.event, UiComponentEvent::ClosePopup)
    }));
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn escape_on_focused_menu_item_closes_nearest_popup_without_activation() {
    let mut surface = menu_surface();
    assert_popup_stack(&surface, &["root/popup"]);
    surface.focus_node(UiNodeId::new(3)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Keyboard(keyboard_pressed("Escape", 27)),
        )
        .unwrap();

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.popup_dismiss")
    );
    assert_popup_open(&surface, false);
    assert_popup_stack(&surface, &[]);
    assert!(result.component_events.iter().all(|event| {
        !matches!(
            &event.event,
            UiComponentEvent::Commit { property, .. } if property == "activated"
        )
    }));
    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2) && matches!(&event.event, UiComponentEvent::ClosePopup)
    }));
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn escape_respects_disable_escape_key_down_on_popup_owner() {
    let mut surface = menu_surface();
    set_popup_attribute(
        &mut surface,
        "disable_escape_key_down",
        toml::Value::Boolean(true),
    );
    surface.focus_node(UiNodeId::new(3)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Keyboard(keyboard_pressed("Escape", 27)),
        )
        .unwrap();

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert_popup_open(&surface, true);
    assert!(result.binding_reports.is_empty());
    assert!(result
        .component_events
        .iter()
        .all(|event| { !matches!(&event.event, UiComponentEvent::ClosePopup) }));
}

#[test]
fn escape_on_descendant_does_not_close_disabled_popup_owner() {
    let mut surface = menu_surface();
    surface
        .tree
        .node_mut(UiNodeId::new(2))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .attributes
        .insert("disabled".to_string(), toml::Value::Boolean(true));
    assert!(surface.focus_node(UiNodeId::new(3)).is_err());
    surface.focus.focused = Some(UiNodeId::new(3));

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Keyboard(keyboard_pressed("Escape", 27)),
        )
        .unwrap();

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert_popup_open(&surface, true);
    assert!(result.binding_reports.is_empty());
    assert!(result
        .component_events
        .iter()
        .all(|event| { !matches!(&event.event, UiComponentEvent::ClosePopup) }));
}

#[test]
fn outside_pointer_click_closes_open_popup_without_menu_item_activation() {
    let mut surface = menu_surface();
    assert_popup_stack(&surface, &["root/popup"]);
    let result = click_point(&mut surface, UiPoint::new(170.0, 100.0));

    assert_popup_open(&surface, false);
    assert_popup_stack(&surface, &[]);
    assert!(result.component_events.iter().any(|event| {
        event.node_id == UiNodeId::new(2)
            && matches!(&event.envelope.event, UiComponentEvent::ClosePopup)
    }));
    assert!(result.component_events.iter().all(|event| {
        !matches!(
            &event.envelope.event,
            UiComponentEvent::Commit { property, .. } if property == "activated"
        )
    }));
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn outside_pointer_click_respects_close_on_backdrop_click_false() {
    let mut surface = menu_surface();
    set_popup_attribute(
        &mut surface,
        "close_on_backdrop_click",
        toml::Value::Boolean(false),
    );

    let result = click_point(&mut surface, UiPoint::new(170.0, 100.0));

    assert_popup_open(&surface, true);
    assert!(result.binding_reports.is_empty());
    assert!(result
        .component_events
        .iter()
        .all(|event| { !matches!(&event.envelope.event, UiComponentEvent::ClosePopup) }));
}

#[test]
fn outside_pointer_click_does_not_close_disabled_popup_owner() {
    let mut surface = menu_surface();
    surface
        .tree
        .node_mut(UiNodeId::new(2))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .attributes
        .insert("disabled".to_string(), toml::Value::Boolean(true));

    let result = click_point(&mut surface, UiPoint::new(170.0, 100.0));

    assert_popup_open(&surface, true);
    assert!(result.binding_reports.is_empty());
    assert!(result
        .component_events
        .iter()
        .all(|event| { !matches!(&event.envelope.event, UiComponentEvent::ClosePopup) }));
}

#[test]
fn outside_pointer_click_closes_topmost_popup_only() {
    let mut surface = menu_surface();
    insert_nested_popup(&mut surface);
    assert_popup_stack(&surface, &["root/popup", "root/popup/nested"]);

    let result = click_point(&mut surface, UiPoint::new(170.0, 100.0));

    assert_popup_node_open(&surface, UiNodeId::new(2), true);
    assert_popup_node_open(&surface, UiNodeId::new(4), false);
    assert_popup_stack(&surface, &["root/popup"]);
    assert!(result.component_events.iter().any(|event| {
        event.node_id == UiNodeId::new(4)
            && matches!(&event.envelope.event, UiComponentEvent::ClosePopup)
    }));
    assert!(result.component_events.iter().all(|event| {
        event.node_id != UiNodeId::new(2)
            || !matches!(&event.envelope.event, UiComponentEvent::ClosePopup)
    }));
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn pointer_click_inside_popup_empty_space_does_not_dismiss_popup() {
    let mut surface = menu_surface();
    let result = click_point(&mut surface, UiPoint::new(132.0, 70.0));

    assert_popup_open(&surface, true);
    assert!(result.binding_reports.is_empty());
    assert!(result
        .component_events
        .iter()
        .all(|event| { !matches!(&event.envelope.event, UiComponentEvent::ClosePopup) }));
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

fn set_popup_attribute(surface: &mut UiSurface, key: &str, value: toml::Value) {
    surface
        .tree
        .node_mut(UiNodeId::new(2))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .attributes
        .insert(key.to_string(), value);
}

fn click_menu_item(
    surface: &mut UiSurface,
) -> zircon_runtime_interface::ui::dispatch::UiPointerDispatchResult {
    click_point(surface, UiPoint::new(22.0, 28.0))
}

fn click_point(
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

fn menu_surface() -> UiSurface {
    menu_surface_with_popup_open(true)
}

pub(super) fn menu_surface_closed() -> UiSurface {
    menu_surface_with_popup_open(false)
}

fn menu_surface_with_popup_open(popup_open: bool) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.widget.menu.behavior"));
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
                .with_state_flags(container_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "MenuPopup".to_string(),
                    attributes: [("popup_open".to_string(), toml::Value::Boolean(popup_open))]
                        .into_iter()
                        .collect(),
                    bindings: vec![binding("MenuPopup/ClosePopup", UiEventKind::Click)],
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Popup,
                        open_property: Some("popup_open".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
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
                .with_state_flags(focusable_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "CommandItem".to_string(),
                    bindings: vec![binding("CommandItem/Activate", UiEventKind::Click)],
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::MenuItem,
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn control_anchored_menu_surface() -> UiSurface {
    let mut surface = menu_surface_with_popup_open(false);
    surface
        .tree
        .node_mut(UiNodeId::new(2))
        .unwrap()
        .layout_cache
        .frame = UiFrame::new(8.0, 8.0, 40.0, 24.0);
    let popup_metadata = surface
        .tree
        .node_mut(UiNodeId::new(2))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap();
    popup_metadata.attributes.insert(
        "menu_items".to_string(),
        toml::Value::Array(vec![
            toml::Value::String("Open".to_string()),
            toml::Value::String("Close".to_string()),
        ]),
    );
    popup_metadata.widget.popup_anchor = UiPopupAnchor::Control {
        control_id: "window-menu-trigger".to_string(),
    };
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(4), UiNodePath::new("root/trigger"))
                .with_frame(UiFrame::new(100.0, 5.0, 24.0, 20.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(focusable_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Button".to_string(),
                    control_id: Some("window-menu-trigger".to_string()),
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Button,
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn control_anchored_dropdown_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.widget.control_anchored_dropdown"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 180.0, 110.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/dropdown"))
                .with_frame(UiFrame::new(8.0, 8.0, 40.0, 24.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(container_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Dropdown".to_string(),
                    attributes: [
                        ("popup_open".to_string(), toml::Value::Boolean(false)),
                        (
                            "options".to_string(),
                            toml::Value::Array(vec![
                                toml::Value::String("Open".to_string()),
                                toml::Value::String("Close".to_string()),
                            ]),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                    bindings: vec![binding("Dropdown/ClosePopup", UiEventKind::Click)],
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Popup,
                        open_property: Some("popup_open".to_string()),
                        popup_anchor: UiPopupAnchor::Control {
                            control_id: "dropdown-trigger".to_string(),
                        },
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(4), UiNodePath::new("root/trigger"))
                .with_frame(UiFrame::new(100.0, 5.0, 24.0, 20.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(focusable_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Button".to_string(),
                    control_id: Some("dropdown-trigger".to_string()),
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Button,
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn insert_nested_popup(surface: &mut UiSurface) {
    insert_popup(surface, UiNodeId::new(2), "root/popup/nested");
}

fn insert_popup(surface: &mut UiSurface, parent_id: UiNodeId, path: &str) {
    surface
        .tree
        .insert_child(
            parent_id,
            UiTreeNode::new(UiNodeId::new(4), UiNodePath::new(path))
                .with_frame(UiFrame::new(72.0, 40.0, 76.0, 40.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(container_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "NestedMenuPopup".to_string(),
                    attributes: [("popup_open".to_string(), toml::Value::Boolean(true))]
                        .into_iter()
                        .collect(),
                    bindings: vec![binding("NestedMenuPopup/ClosePopup", UiEventKind::Click)],
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Popup,
                        open_property: Some("popup_open".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();
}

pub(super) fn assert_popup_open(surface: &UiSurface, expected: bool) {
    assert_popup_node_open(surface, UiNodeId::new(2), expected);
}

pub(super) fn assert_popup_stack(surface: &UiSurface, expected: &[&str]) {
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

fn binding(id: &str, event: UiEventKind) -> UiBindingRef {
    UiBindingRef {
        component_event: super::typed_component_event_kind_for_test(id),
        id: id.to_string(),
        event,
        mode: Default::default(),
        route: Some(id.replace('/', ".")),
        action: None,
        targets: Vec::new(),
    }
}

fn container_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        ..UiStateFlags::default()
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

pub(super) fn keyboard_pressed(logical_key: &str, key_code: u32) -> UiKeyboardInputEvent {
    UiKeyboardInputEvent {
        metadata: UiInputEventMetadata::new(
            UiInputTimestamp::from_micros(40),
            UiInputSequence::new(4),
        ),
        state: UiKeyboardInputState::Pressed,
        key_code,
        scan_code: None,
        physical_key: logical_key.to_string(),
        logical_key: logical_key.to_string(),
        text: None,
    }
}
