use super::*;
use zircon_runtime_interface::ui::widget::UiPopupAnchor;

#[test]
fn control_anchored_popup_restores_focus_to_trigger_after_close() {
    let mut surface = popup_focus_surface();
    surface
        .tree
        .insert_child(id(1), focus_node(6, "other", 120.0, 0.0))
        .unwrap();
    surface
        .tree
        .node_mut(id(2))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .control_id = Some("window-menu-trigger".to_string());
    surface
        .tree
        .node_mut(id(3))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .widget
        .popup_anchor = UiPopupAnchor::Control {
        control_id: "window-menu-trigger".to_string(),
    };
    surface.rebuild();

    surface.focus_node(id(6)).unwrap();
    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "popup_open",
            UiValue::Bool(true),
        ))
        .unwrap();
    assert_eq!(surface.input.popup_owner("root/popup"), Some(id(2)));

    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "popup_open",
            UiValue::Bool(false),
        ))
        .unwrap();

    assert_eq!(surface.focus.focused, Some(id(2)));
    assert!(surface.input.popup_stack.is_empty());
}

#[test]
fn nested_control_anchored_popups_keep_each_trigger_owner() {
    let mut surface = popup_focus_surface();
    surface
        .tree
        .node_mut(id(2))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .control_id = Some("window-menu-trigger".to_string());
    surface
        .tree
        .node_mut(id(3))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .widget
        .popup_anchor = UiPopupAnchor::Control {
        control_id: "window-menu-trigger".to_string(),
    };
    surface
        .tree
        .node_mut(id(4))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .control_id = Some("submenu-trigger".to_string());
    surface
        .tree
        .insert_child(
            id(3),
            UiTreeNode::new(id(7), UiNodePath::new("root/popup/nested"))
                .with_frame(UiFrame::new(40.0, 48.0, 80.0, 48.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    ..Default::default()
                })
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "MenuPopup".to_string(),
                    attributes: [("popup_open".to_string(), toml::Value::Boolean(false))]
                        .into_iter()
                        .collect(),
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Popup,
                        open_property: Some("popup_open".to_string()),
                        popup_anchor: UiPopupAnchor::Control {
                            control_id: "submenu-trigger".to_string(),
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "popup_open",
            UiValue::Bool(true),
        ))
        .unwrap();
    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(7),
            "popup_open",
            UiValue::Bool(true),
        ))
        .unwrap();

    assert_eq!(surface.input.popup_owner("root/popup"), Some(id(2)));
    assert_eq!(surface.input.popup_owner("root/popup/nested"), Some(id(4)));

    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(7),
            "popup_open",
            UiValue::Bool(false),
        ))
        .unwrap();

    assert_eq!(surface.input.popup_owner("root/popup"), Some(id(2)));
    assert!(surface.input.popup_owner("root/popup/nested").is_none());
}

#[test]
fn nested_control_anchored_popup_escape_closes_topmost_and_restores_each_trigger() {
    let mut surface = popup_focus_surface();
    surface
        .tree
        .node_mut(id(2))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .control_id = Some("window-menu-trigger".to_string());
    surface
        .tree
        .node_mut(id(3))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .widget
        .popup_anchor = UiPopupAnchor::Control {
        control_id: "window-menu-trigger".to_string(),
    };
    surface
        .tree
        .node_mut(id(4))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .control_id = Some("submenu-trigger".to_string());
    surface
        .tree
        .insert_child(
            id(3),
            UiTreeNode::new(id(7), UiNodePath::new("root/popup/nested"))
                .with_frame(UiFrame::new(40.0, 48.0, 80.0, 48.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    ..Default::default()
                })
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "MenuPopup".to_string(),
                    attributes: [("popup_open".to_string(), toml::Value::Boolean(false))]
                        .into_iter()
                        .collect(),
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Popup,
                        open_property: Some("popup_open".to_string()),
                        popup_anchor: UiPopupAnchor::Control {
                            control_id: "submenu-trigger".to_string(),
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "popup_open",
            UiValue::Bool(true),
        ))
        .unwrap();
    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(7),
            "popup_open",
            UiValue::Bool(true),
        ))
        .unwrap();

    let escape = || {
        UiInputEvent::Keyboard(UiKeyboardInputEvent {
            metadata: input_metadata(),
            state: UiKeyboardInputState::Pressed,
            key_code: 27,
            scan_code: None,
            physical_key: "Escape".to_string(),
            logical_key: "Escape".to_string(),
            text: None,
        })
    };
    let pointer_dispatcher = UiPointerDispatcher::default();
    let navigation_dispatcher = UiNavigationDispatcher::default();

    surface.focus.focused = Some(id(7));
    surface
        .dispatch_input_event(&pointer_dispatcher, &navigation_dispatcher, escape())
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(4)));
    assert_eq!(surface.input.popup_owner("root/popup"), Some(id(2)));
    assert!(surface.input.popup_owner("root/popup/nested").is_none());

    surface
        .dispatch_input_event(&pointer_dispatcher, &navigation_dispatcher, escape())
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(2)));
    assert!(surface.input.popup_stack.is_empty());
}

#[test]
fn invalid_control_anchored_popup_does_not_open_input_or_focus_scope() {
    let mut surface = popup_focus_surface();
    surface
        .tree
        .node_mut(id(3))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .widget
        .popup_anchor = UiPopupAnchor::Control {
        control_id: "missing-trigger".to_string(),
    };
    surface.rebuild();
    surface.focus_node(id(2)).unwrap();

    let report = surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "popup_open",
            UiValue::Bool(true),
        ))
        .unwrap();

    assert!(surface.input.popup_stack.is_empty());
    assert!(surface.focus.modal_restore_stack.is_empty());
    assert!(report.focus_change.is_none());
    assert_eq!(surface.focus.focused, Some(id(2)));
    assert_eq!(
        surface
            .tree
            .node(id(3))
            .unwrap()
            .template_metadata
            .as_ref()
            .unwrap()
            .attributes
            .get("popup_open")
            .and_then(toml::Value::as_bool),
        Some(false)
    );
}
