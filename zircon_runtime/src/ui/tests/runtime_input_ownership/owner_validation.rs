use super::*;

#[test]
fn focus_and_capture_reject_hidden_ancestor_owners_without_clearing_current_owner() {
    let mut surface = two_button_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();
    surface.input.input_method_owner = Some(UiNodeId::new(2));
    surface.focus.captured = Some(UiNodeId::new(2));
    surface.input.high_precision_owner = Some(UiNodeId::new(2));
    surface
        .tree
        .nodes
        .get_mut(&UiNodeId::new(1))
        .unwrap()
        .visibility = UiVisibility::Collapsed;

    let rejected_focus = surface.focus_node(UiNodeId::new(3));

    assert!(rejected_focus.is_err());
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(surface.input.input_method_owner, Some(UiNodeId::new(2)));

    let rejected_capture = surface.capture_pointer(UiNodeId::new(3));

    assert!(rejected_capture.is_err());
    assert_eq!(surface.focus.captured, Some(UiNodeId::new(2)));
    assert_eq!(surface.input.high_precision_owner, Some(UiNodeId::new(2)));
}

#[test]
fn mutating_disabled_ancestor_clears_focus_and_transient_input_owners() {
    let mut surface = two_button_surface();
    let root = surface
        .tree
        .node_mut(UiNodeId::new(1))
        .expect("root should exist");
    root.template_metadata = Some(UiTemplateNodeMetadata {
        component: "Panel".to_string(),
        ..UiTemplateNodeMetadata::default()
    });
    surface.focus_node(UiNodeId::new(2)).unwrap();
    surface.focus.captured = Some(UiNodeId::new(2));
    surface.focus.pressed = Some(UiNodeId::new(2));
    surface.focus.hovered = vec![UiNodeId::new(2), UiNodeId::new(3)];
    capture_pointer_for_test(&mut surface, UiPointerId::new(7), UiNodeId::new(2));
    surface.input.high_precision_owner = Some(UiNodeId::new(2));
    surface.input.input_method_owner = Some(UiNodeId::new(2));
    surface.input.pointer_lock_owner = Some(UiNodeId::new(2));
    surface.input.pointer_lock_policy = Some(UiPointerLockPolicy::RawDelta);
    surface
        .input
        .begin_drag_drop(
            UiNodeId::new(2),
            UiNodeId::new(2),
            UiPointerId::new(7),
            Some(UiDragSessionId::new(31)),
            Some(UiPoint::new(10.0, 10.0)),
            None,
        )
        .unwrap();

    let report = surface
        .mutate_property(UiPropertyMutationRequest::new(
            UiNodeId::new(1),
            "disabled",
            UiValue::Bool(true),
        ))
        .unwrap();

    assert_eq!(report.status, UiPropertyMutationStatus::Accepted);
    let focus_change = report
        .focus_change
        .expect("disabled ancestor should clear focused descendant");
    assert_eq!(focus_change.previous, Some(UiNodeId::new(2)));
    assert_eq!(focus_change.current, None);
    assert_eq!(focus_change.reason, UiFocusChangeReason::Disabled);
    assert_eq!(surface.focus.focused, None);
    assert_eq!(surface.focus.captured, None);
    assert_eq!(surface.focus.pressed, None);
    assert!(surface.focus.hovered.is_empty());
    assert_no_pointer_capture(&surface);
    assert_eq!(surface.input.high_precision_owner, None);
    assert_eq!(surface.input.input_method_owner, None);
    assert_eq!(surface.input.pointer_lock_owner, None);
    assert_eq!(surface.input.pointer_lock_policy, None);
    assert_eq!(surface.input.drag_drop, None);
    assert!(
        surface
            .component_state(UiNodeId::new(1))
            .expect("disabled mutation should mirror component state")
            .flags
            .disabled
    );
}

#[test]
fn direct_capture_without_pointer_id_does_not_enable_high_precision() {
    let mut surface = two_button_surface();
    surface.focus.captured = Some(UiNodeId::new(2));
    surface.input.high_precision_owner = Some(UiNodeId::new(2));

    surface.capture_pointer(UiNodeId::new(3)).unwrap();

    assert_eq!(surface.focus.captured, Some(UiNodeId::new(3)));
    assert_eq!(surface.input.high_precision_owner, None);

    let high_precision = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::UseHighPrecisionPointer {
            target: UiNodeId::new(3),
            enabled: true,
        }),
    );

    assert_eq!(surface.input.high_precision_owner, None);
    assert!(high_precision.host_requests.is_empty());
    assert_eq!(high_precision.rejected_effects.len(), 1);
    assert_eq!(
        high_precision.rejected_effects[0].reason,
        "high precision requires pointer capture"
    );
}
