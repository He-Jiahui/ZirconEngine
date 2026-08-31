use super::*;

mod control_anchored;

#[test]
fn generic_modal_group_open_traps_inherited_scope_and_restores_focus() {
    let mut surface = generic_modal_group_surface();
    surface.focus_node(id(2)).unwrap();

    let open_report = surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "open",
            UiValue::Bool(true),
        ))
        .unwrap();

    assert_eq!(open_report.focus_change.unwrap().current, Some(id(4)));
    assert_eq!(surface.focus.focused, Some(id(4)));

    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Next,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(5)));

    surface.focus_node(id(2)).unwrap();
    assert_eq!(surface.focus.focused, Some(id(4)));

    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "open",
            UiValue::Bool(true),
        ))
        .unwrap();

    let close_report = surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "open",
            UiValue::Bool(false),
        ))
        .unwrap();

    assert_eq!(close_report.focus_change.unwrap().current, Some(id(2)));
    assert_eq!(surface.focus.focused, Some(id(2)));
    assert!(surface.focus.modal_restore_stack.is_empty());
}

#[test]
fn higher_z_open_scope_takes_focus_from_existing_sibling_scope() {
    let mut surface = stacked_generic_modal_group_surface();
    surface.focus_node(id(2)).unwrap();
    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "open",
            UiValue::Bool(true),
        ))
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(4)));

    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(8),
            "open",
            UiValue::Bool(true),
        ))
        .unwrap();

    assert_eq!(surface.focus.focused, Some(id(9)));

    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "open",
            UiValue::Bool(false),
        ))
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(9)));

    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(8),
            "open",
            UiValue::Bool(false),
        ))
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(2)));
}

#[test]
fn higher_z_scope_closes_correctly_when_opened_before_lower_z_scope() {
    let mut surface = stacked_generic_modal_group_surface();
    surface.focus_node(id(2)).unwrap();
    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(8),
            "open",
            UiValue::Bool(true),
        ))
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(9)));

    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "open",
            UiValue::Bool(true),
        ))
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(9)));

    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(8),
            "open",
            UiValue::Bool(false),
        ))
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(4)));

    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "open",
            UiValue::Bool(false),
        ))
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(2)));
}

#[test]
fn background_scope_disable_restore_is_preserved_through_upper_scope_close() {
    let mut surface = stacked_generic_modal_group_surface();
    surface
        .tree
        .node_mut(id(3))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .attributes
        .insert(
            "disable_restore_focus".to_string(),
            toml::Value::Boolean(true),
        );
    surface.rebuild();
    surface.focus_node(id(2)).unwrap();
    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "open",
            UiValue::Bool(true),
        ))
        .unwrap();
    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(8),
            "open",
            UiValue::Bool(true),
        ))
        .unwrap();

    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "open",
            UiValue::Bool(false),
        ))
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(9)));

    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(8),
            "open",
            UiValue::Bool(false),
        ))
        .unwrap();
    assert_eq!(surface.focus.focused, None);
}

#[test]
fn closing_disable_restore_scope_focuses_remaining_modal_scope() {
    let mut surface = stacked_generic_modal_group_surface();
    surface
        .tree
        .node_mut(id(8))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .attributes
        .insert(
            "disable_restore_focus".to_string(),
            toml::Value::Boolean(true),
        );
    surface.rebuild();
    surface.focus_node(id(2)).unwrap();
    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "open",
            UiValue::Bool(true),
        ))
        .unwrap();
    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(8),
            "open",
            UiValue::Bool(true),
        ))
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(9)));

    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(8),
            "open",
            UiValue::Bool(false),
        ))
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(4)));
}

#[test]
fn sibling_modal_group_close_without_restore_clears_group_focus() {
    let mut surface = navigation_surface();
    let metadata = surface
        .tree
        .node_mut(id(5))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap();
    metadata
        .attributes
        .insert("open".to_string(), toml::Value::Boolean(false));
    metadata.attributes.insert(
        "disable_restore_focus".to_string(),
        toml::Value::Boolean(true),
    );
    metadata.widget.open_property = Some("open".to_string());
    surface.rebuild();
    surface.focus_node(id(2)).unwrap();

    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(5),
            "open",
            UiValue::Bool(true),
        ))
        .unwrap();
    surface.focus_node(id(6)).unwrap();
    assert_eq!(surface.focus.focused, Some(id(6)));

    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(5),
            "open",
            UiValue::Bool(false),
        ))
        .unwrap();

    assert_eq!(surface.focus.focused, None);
    assert!(surface.focus.modal_restore_stack.is_empty());
}

#[test]
fn modal_restore_resolves_stable_path_after_node_id_rebuild() {
    let mut surface = generic_modal_group_surface();
    surface.focus_node(id(2)).unwrap();
    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "open",
            UiValue::Bool(true),
        ))
        .unwrap();

    surface.tree.nodes.remove(&id(2));
    surface
        .tree
        .node_mut(id(1))
        .unwrap()
        .children
        .retain(|child| *child != id(2));
    surface
        .tree
        .insert_child(id(1), focus_node(2, "replacement", 120.0, 0.0))
        .unwrap();
    surface
        .tree
        .insert_child(id(1), focus_node(7, "outside", 0.0, 0.0))
        .unwrap();
    surface.rebuild();

    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(3),
            "open",
            UiValue::Bool(false),
        ))
        .unwrap();

    assert_eq!(surface.focus.focused, Some(id(7)));
    assert!(surface.focus.modal_restore_stack.is_empty());
}

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
