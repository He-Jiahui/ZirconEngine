use super::*;

#[test]
fn mui_modal_open_autofocus_traps_navigation_and_restores_previous_focus() {
    let mut surface = mui_modal_surface(false, false, false);
    surface.focus_node(id(2)).unwrap();

    let open_report = surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "open",
            UiValue::Bool(true),
        ))
        .unwrap();

    assert_eq!(surface.focus.focused, Some(id(4)));
    assert_eq!(open_report.focus_change.unwrap().current, Some(id(4)));
    assert_eq!(surface.focus.modal_restore_stack.len(), 1);
    assert_eq!(surface.focus.modal_restore_stack[0].restore, Some(id(2)));

    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Next,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(5)));
    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Next,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(4)));

    surface.focus_node(id(2)).unwrap();
    assert_eq!(surface.focus.focused, Some(id(4)));

    let close_report = surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "open",
            UiValue::Bool(false),
        ))
        .unwrap();

    assert_eq!(surface.focus.focused, Some(id(2)));
    assert_eq!(close_report.focus_change.unwrap().current, Some(id(2)));
    assert!(surface.focus.modal_restore_stack.is_empty());
}

#[test]
fn confirm_dialog_popup_open_autofocus_traps_navigation_and_restores_previous_focus() {
    let mut surface =
        mui_modal_component_surface("ConfirmDialog", "popup_open", false, false, false);
    surface.focus_node(id(2)).unwrap();

    let open_report = surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "popup_open",
            UiValue::Bool(true),
        ))
        .unwrap();

    assert_eq!(surface.focus.focused, Some(id(4)));
    assert_eq!(open_report.focus_change.unwrap().current, Some(id(4)));
    assert_eq!(surface.focus.modal_restore_stack.len(), 1);
    assert_eq!(surface.focus.modal_restore_stack[0].restore, Some(id(2)));
    assert_eq!(
        surface
            .tree
            .node(id(3))
            .unwrap()
            .template_metadata
            .as_ref()
            .unwrap()
            .widget
            .resolved_behavior("ConfirmDialog"),
        UiWidgetBehavior::Passive
    );
    assert_eq!(
        surface
            .input
            .popup_stack
            .iter()
            .map(|popup| popup.popup_id.as_str())
            .collect::<Vec<_>>(),
        vec!["root/modal"]
    );

    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Next,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(5)));
    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Next,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(4)));
    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Previous,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(5)));

    surface.focus_node(id(2)).unwrap();
    assert_eq!(surface.focus.focused, Some(id(4)));

    let close_report = surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "popup_open",
            UiValue::Bool(false),
        ))
        .unwrap();

    assert_eq!(surface.focus.focused, Some(id(2)));
    assert_eq!(close_report.focus_change.unwrap().current, Some(id(2)));
    assert!(surface.focus.modal_restore_stack.is_empty());
    assert!(surface.input.popup_stack.is_empty());
}

#[test]
fn mui_modal_focus_flags_can_disable_auto_enforce_and_restore() {
    let mut surface = mui_modal_surface(true, true, true);
    surface.focus_node(id(2)).unwrap();

    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "open",
            UiValue::Bool(true),
        ))
        .unwrap();

    assert_eq!(surface.focus.focused, Some(id(2)));
    assert!(surface.focus_node(id(2)).is_ok());
    assert_eq!(surface.focus.focused, Some(id(2)));

    surface.focus_node(id(4)).unwrap();
    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "open",
            UiValue::Bool(false),
        ))
        .unwrap();

    assert_eq!(surface.focus.focused, None);
    assert!(surface.focus.modal_restore_stack.is_empty());
}

#[test]
fn widget_popup_open_traps_focus_loop_and_restores_previous_focus() {
    let mut surface = popup_focus_surface();
    surface.focus_node(id(2)).unwrap();

    let open_report = surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "popup_open",
            UiValue::Bool(true),
        ))
        .unwrap();

    assert_eq!(surface.focus.focused, Some(id(4)));
    assert_eq!(open_report.focus_change.unwrap().current, Some(id(4)));
    assert_eq!(
        surface
            .input
            .popup_stack
            .iter()
            .map(|popup| popup.popup_id.as_str())
            .collect::<Vec<_>>(),
        vec!["root/popup"]
    );

    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Next,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(5)));
    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Next,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(4)));
    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Previous,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(5)));

    surface.focus_node(id(2)).unwrap();
    assert_eq!(surface.focus.focused, Some(id(4)));

    let close_report = surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "popup_open",
            UiValue::Bool(false),
        ))
        .unwrap();

    assert_eq!(surface.focus.focused, Some(id(2)));
    assert_eq!(close_report.focus_change.unwrap().current, Some(id(2)));
    assert!(surface.input.popup_stack.is_empty());
}

#[test]
fn widget_popup_without_focusable_descendants_opens_without_stealing_focus() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.popup.focus.empty"));
    surface.tree.insert_root(root_node());
    surface
        .tree
        .insert_child(id(1), focus_node(2, "outside", 0.0, 0.0))
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(3), UiNodePath::new("root/popup"))
                .with_frame(UiFrame::new(0.0, 40.0, 120.0, 24.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    ..Default::default()
                })
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Dropdown".to_string(),
                    attributes: [("popup_open".to_string(), toml::Value::Boolean(false))]
                        .into_iter()
                        .collect(),
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Popup,
                        open_property: Some("popup_open".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface.focus_node(id(2)).unwrap();

    let open_report = surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "popup_open",
            UiValue::Bool(true),
        ))
        .unwrap();

    assert_eq!(surface.focus.focused, Some(id(2)));
    assert!(open_report.focus_change.is_none());
    assert_eq!(
        surface
            .input
            .popup_stack
            .iter()
            .map(|popup| popup.popup_id.as_str())
            .collect::<Vec<_>>(),
        vec!["root/popup"]
    );
    assert_eq!(surface.focus.modal_restore_stack.len(), 1);
    assert_eq!(surface.focus.modal_restore_stack[0].restore, Some(id(2)));

    let focused_report = surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "focused",
            UiValue::Bool(true),
        ))
        .unwrap();

    assert_eq!(surface.focus.focused, Some(id(2)));
    assert!(focused_report.focus_change.is_none());

    let close_report = surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "popup_open",
            UiValue::Bool(false),
        ))
        .unwrap();

    assert_eq!(surface.focus.focused, Some(id(2)));
    assert!(close_report.focus_change.is_none());
    assert!(surface.input.popup_stack.is_empty());
    assert!(surface.focus.modal_restore_stack.is_empty());
}

#[test]
fn widget_popup_under_hidden_ancestor_opens_without_focus_error() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.popup.focus.hidden_ancestor"));
    surface.tree.insert_root(root_node());
    surface
        .tree
        .insert_child(id(1), focus_node(2, "outside", 0.0, 0.0))
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(3), UiNodePath::new("root/hidden"))
                .with_visibility(UiVisibility::Collapsed)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    ..Default::default()
                }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(3),
            UiTreeNode::new(id(4), UiNodePath::new("root/hidden/popup"))
                .with_frame(UiFrame::new(0.0, 40.0, 120.0, 24.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    ..Default::default()
                })
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Dropdown".to_string(),
                    attributes: [("popup_open".to_string(), toml::Value::Boolean(false))]
                        .into_iter()
                        .collect(),
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Popup,
                        open_property: Some("popup_open".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface.focus_node(id(2)).unwrap();

    let open_report = surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(4),
            "popup_open",
            UiValue::Bool(true),
        ))
        .unwrap();

    assert_eq!(surface.focus.focused, Some(id(2)));
    assert!(open_report.focus_change.is_none());
    assert_eq!(
        surface
            .input
            .popup_stack
            .iter()
            .map(|popup| popup.popup_id.as_str())
            .collect::<Vec<_>>(),
        vec!["root/hidden/popup"]
    );
}
