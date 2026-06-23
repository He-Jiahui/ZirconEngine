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
