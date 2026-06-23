use super::*;

#[test]
fn rejected_focus_effect_preserves_current_input_method_owner() {
    let mut surface = two_button_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();
    surface.input.input_method_owner = Some(UiNodeId::new(2));

    let result = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::SetFocus {
            target: UiNodeId::new(99),
            reason: UiFocusEffectReason::Input,
        }),
    );

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(surface.input.input_method_owner, Some(UiNodeId::new(2)));
    assert!(result.applied_effects.is_empty());
    assert_eq!(result.rejected_effects.len(), 1);
    assert!(result.rejected_effects[0]
        .reason
        .starts_with("focus rejected"));
}

#[test]
fn navigation_focus_changes_clear_previous_input_method_owner() {
    let mut surface = two_button_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();
    surface.input.input_method_owner = Some(UiNodeId::new(2));

    let effect_result = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::RequestNavigation {
            kind: UiNavigationEventKind::Next,
            policy: UiNavigationRequestPolicy::Wrap,
        }),
    );

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(3)));
    assert_eq!(surface.input.input_method_owner, None);
    assert!(effect_result.rejected_effects.is_empty());

    surface.input.input_method_owner = Some(UiNodeId::new(3));
    let dispatch_result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Navigation(UiNavigationInputEvent {
                metadata: input_metadata(),
                kind: UiNavigationEventKind::Previous,
            }),
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(surface.input.input_method_owner, None);
    assert_eq!(dispatch_result.applied_effects.len(), 1);
}

#[test]
fn clear_focus_clears_only_the_focused_input_method_owner() {
    let mut surface = two_button_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();
    surface.input.input_method_owner = Some(UiNodeId::new(2));

    surface.clear_focus();

    assert_eq!(surface.focus.focused, None);
    assert_eq!(surface.input.input_method_owner, None);

    surface.focus_node(UiNodeId::new(2)).unwrap();
    surface.input.input_method_owner = Some(UiNodeId::new(3));

    surface.clear_focus();

    assert_eq!(surface.focus.focused, None);
    assert_eq!(surface.input.input_method_owner, Some(UiNodeId::new(3)));
}

#[test]
fn input_method_reset_and_cursor_update_require_current_owner() {
    let mut surface = two_button_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();
    surface.input.input_method_owner = Some(UiNodeId::new(2));

    let stale_reset = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::RequestInputMethod {
            request: input_method_request(UiInputMethodRequestKind::Reset, UiNodeId::new(3)),
        }),
    );

    assert_eq!(surface.input.input_method_owner, Some(UiNodeId::new(2)));
    assert!(stale_reset.host_requests.is_empty());
    assert_eq!(stale_reset.rejected_effects.len(), 1);
    assert_eq!(
        stale_reset.rejected_effects[0].reason,
        "input method owner mismatch"
    );

    let stale_update = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::RequestInputMethod {
            request: input_method_request(UiInputMethodRequestKind::UpdateCursor, UiNodeId::new(3)),
        }),
    );

    assert_eq!(surface.input.input_method_owner, Some(UiNodeId::new(2)));
    assert!(stale_update.host_requests.is_empty());
    assert_eq!(stale_update.rejected_effects.len(), 1);
    assert_eq!(
        stale_update.rejected_effects[0].reason,
        "input method owner mismatch"
    );

    let current_update = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::RequestInputMethod {
            request: input_method_request(UiInputMethodRequestKind::UpdateCursor, UiNodeId::new(2)),
        }),
    );

    assert!(current_update.rejected_effects.is_empty());
    assert_eq!(current_update.host_requests.len(), 1);
}
