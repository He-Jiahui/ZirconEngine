use super::*;

#[test]
fn authored_focus_contract_makes_node_focusable_without_legacy_state_flag() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.focus.contract"));
    surface.tree.insert_root(root_node());
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/authored_focus"))
                .with_frame(UiFrame::new(0.0, 0.0, 32.0, 24.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: false,
                    hoverable: false,
                    focusable: false,
                    ..Default::default()
                })
                .with_focus_contract({
                    let mut focus = zircon_runtime_interface::ui::focus::UiFocusContract::default();
                    focus.focusable = true;
                    focus
                }),
        )
        .unwrap();
    surface.rebuild();

    surface.focus_node(id(2)).unwrap();
    let arranged = surface.arranged_tree.get(id(2)).unwrap();

    assert_eq!(surface.focus.focused, Some(id(2)));
    assert!(arranged.focusable);
    assert!(arranged.supports_pointer());
}

#[test]
fn focus_is_cleared_when_focused_node_stops_accepting_input() {
    let mut surface = focus_surface();
    surface.focus_node(id(2)).unwrap();
    assert!(surface.component_state(id(2)).unwrap().flags.focused);
    surface.input.input_method_owner = Some(id(2));
    surface.focus.captured = Some(id(2));
    surface
        .input
        .set_pointer_capture_for_id(UiPointerId::new(7), id(2));

    let event = surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(2),
            "enabled",
            zircon_runtime_interface::ui::component::UiValue::Bool(false),
        ))
        .unwrap()
        .focus_change
        .expect("focus cleared");

    assert_eq!(surface.focus.focused, None);
    assert_eq!(surface.input.input_method_owner, None);
    assert_eq!(surface.focus.captured, None);
    assert_eq!(surface.input.active_pointer_capture(), None);
    assert!(!surface.component_state(id(2)).unwrap().flags.focused);
    assert_eq!(event.previous, Some(id(2)));
    assert_eq!(event.current, None);
    assert_eq!(event.reason, UiFocusChangeReason::Disabled);
}

#[test]
fn focus_is_cleared_when_focused_node_ancestor_is_disabled() {
    let mut surface = focus_surface();
    surface.focus_node(id(2)).unwrap();
    surface.input.input_method_owner = Some(id(2));
    surface.focus.captured = Some(id(2));
    surface.input.pointer_lock_owner = Some(id(2));

    let event = surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(1),
            "enabled",
            UiValue::Bool(false),
        ))
        .unwrap()
        .focus_change
        .expect("ancestor disabled clears descendant focus");

    assert_eq!(surface.focus.focused, None);
    assert_eq!(surface.input.input_method_owner, None);
    assert_eq!(surface.focus.captured, None);
    assert_eq!(surface.input.pointer_lock_owner, None);
    assert_eq!(event.previous, Some(id(2)));
    assert_eq!(event.current, None);
    assert_eq!(event.reason, UiFocusChangeReason::Disabled);
}

#[test]
fn unchanged_or_rejected_focus_related_mutations_do_not_emit_focus_changes() {
    let mut surface = focus_surface();
    surface.focus_node(id(2)).unwrap();

    let unchanged = surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(2),
            "enabled",
            UiValue::Bool(true),
        ))
        .unwrap();
    let rejected = surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(2),
            "focusable",
            UiValue::String("false".to_string()),
        ))
        .unwrap();

    assert_eq!(surface.focus.focused, Some(id(2)));
    assert!(unchanged.focus_change.is_none());
    assert!(rejected.focus_change.is_none());
}
