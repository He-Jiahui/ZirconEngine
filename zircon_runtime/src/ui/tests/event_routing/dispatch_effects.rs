use super::*;

#[test]
fn bound_custom_template_component_dispatches_click_envelope_after_build() {
    let mut surface = template_surface_from_root_toml(root_with_inline_node(
        r#"{ component = "ScriptActionChip", control_id = "ActionChip", bindings = [{ id = "Demo/Action", event = "Click", route = "Demo.Action" }], attributes = { layout = { width = { min = 80.0, preferred = 80.0, max = 80.0, stretch = "Fixed" }, height = { min = 30.0, preferred = 30.0, max = 30.0, stretch = "Fixed" } } } }"#,
    ));
    surface.compute_layout(UiSize::new(100.0, 50.0)).unwrap();

    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(10.0, 10.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    let result = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(10.0, 10.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(result.route.click_target, Some(UiNodeId::new(1)));
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.component_events[0].node_id, UiNodeId::new(1));
    assert_eq!(result.component_events[0].envelope.control_id, "ActionChip");
    assert_eq!(result.component_events[0].binding_id, "Demo/Action");
    assert_eq!(result.component_events[0].event_kind, UiEventKind::Click);
    assert_eq!(
        result.component_events[0].reason,
        UiPointerComponentEventReason::DefaultClick
    );
}

#[test]
fn dispatch_reply_applies_focus_capture_high_precision_and_release_effects() {
    let mut surface = button_surface();
    let pointer_id = UiPointerId::new(7);
    let reply = UiDispatchReply::handled().with_effects([
        UiDispatchEffect::SetFocus {
            target: UiNodeId::new(2),
            reason: UiFocusEffectReason::Input,
        },
        UiDispatchEffect::CapturePointer {
            target: UiNodeId::new(2),
            pointer_id,
            reason: UiPointerCaptureReason::Press,
        },
        UiDispatchEffect::UseHighPrecisionPointer {
            target: UiNodeId::new(2),
            enabled: true,
        },
        UiDispatchEffect::ReleasePointerCapture {
            target: UiNodeId::new(2),
            pointer_id,
            reason: UiPointerCaptureReason::Cancel,
        },
    ]);

    let result = surface.apply_dispatch_reply(keyboard_event(), reply);

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(surface.focus.captured, None);
    assert_no_pointer_capture(&surface);
    assert_eq!(surface.input.high_precision_owner, None);
    assert_eq!(result.applied_effects.len(), 4);
    assert!(result.rejected_effects.is_empty());

    surface.focus.captured = Some(UiNodeId::new(2));
    surface.input.high_precision_owner = Some(UiNodeId::new(2));
    let stale_release = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::ReleasePointerCapture {
            target: UiNodeId::new(2),
            pointer_id,
            reason: UiPointerCaptureReason::Cancel,
        }),
    );

    assert_eq!(surface.focus.captured, Some(UiNodeId::new(2)));
    assert_eq!(surface.input.high_precision_owner, Some(UiNodeId::new(2)));
    assert!(stale_release.applied_effects.is_empty());
    assert_eq!(stale_release.rejected_effects.len(), 1);
    assert_eq!(
        stale_release.rejected_effects[0].reason,
        "pointer capture belongs to a different or unknown pointer"
    );

    let release_reply = UiDispatchReply::handled().with_effects([
        UiDispatchEffect::CapturePointer {
            target: UiNodeId::new(2),
            pointer_id,
            reason: UiPointerCaptureReason::Press,
        },
        UiDispatchEffect::UseHighPrecisionPointer {
            target: UiNodeId::new(2),
            enabled: true,
        },
        UiDispatchEffect::ReleasePointerCapture {
            target: UiNodeId::new(2),
            pointer_id,
            reason: UiPointerCaptureReason::Cancel,
        },
    ]);

    let release_result = surface.apply_dispatch_reply(keyboard_event(), release_reply);

    assert_eq!(surface.focus.captured, None);
    assert_no_pointer_capture(&surface);
    assert_eq!(surface.input.high_precision_owner, None);
    assert_eq!(release_result.applied_effects.len(), 3);
    assert!(release_result.rejected_effects.is_empty());

    surface.input.high_precision_owner = Some(UiNodeId::new(2));
    let stale_high_precision_disable = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::UseHighPrecisionPointer {
            target: UiNodeId::new(1),
            enabled: false,
        }),
    );

    assert_eq!(surface.input.high_precision_owner, Some(UiNodeId::new(2)));
    assert!(stale_high_precision_disable.host_requests.is_empty());
    assert_eq!(stale_high_precision_disable.rejected_effects.len(), 1);
    assert_eq!(
        stale_high_precision_disable.rejected_effects[0].reason,
        "high precision owner mismatch"
    );

    capture_pointer_for_test(&mut surface, pointer_id, UiNodeId::new(1));
    surface.input.high_precision_owner = Some(UiNodeId::new(1));
    let stale_capture_release = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::ReleasePointerCapture {
            target: UiNodeId::new(2),
            pointer_id,
            reason: UiPointerCaptureReason::Cancel,
        }),
    );

    assert_eq!(surface.focus.captured, Some(UiNodeId::new(1)));
    assert_eq!(surface.input.high_precision_owner, Some(UiNodeId::new(1)));
    assert!(stale_capture_release.applied_effects.is_empty());
    assert_eq!(stale_capture_release.rejected_effects.len(), 1);

    surface.input.pointer_lock_owner = Some(UiNodeId::new(2));
    surface.input.pointer_lock_policy = Some(UiPointerLockPolicy::RawDelta);
    let stale_unlock = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::UnlockPointer {
            target: UiNodeId::new(1),
            policy: UiPointerLockPolicy::RawDelta,
        }),
    );

    assert_eq!(surface.input.pointer_lock_owner, Some(UiNodeId::new(2)));
    assert!(stale_unlock.host_requests.is_empty());
    assert_eq!(
        stale_unlock.rejected_effects[0].reason,
        "pointer lock owner mismatch"
    );
}

#[test]
fn focus_effects_clear_only_their_current_input_owner() {
    let mut surface = two_button_surface(None, None);
    surface.focus_node(UiNodeId::new(2)).unwrap();
    surface.input.input_method_owner = Some(UiNodeId::new(2));

    let stale_clear = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::ClearFocus {
            target: UiNodeId::new(3),
            reason: UiFocusEffectReason::Dismissal,
        }),
    );

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(surface.input.input_method_owner, Some(UiNodeId::new(2)));
    assert!(stale_clear.applied_effects.is_empty());
    assert_eq!(
        stale_clear.rejected_effects[0].reason,
        "focus owner mismatch"
    );

    let clear = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::ClearFocus {
            target: UiNodeId::new(2),
            reason: UiFocusEffectReason::Dismissal,
        }),
    );

    assert_eq!(surface.focus.focused, None);
    assert_eq!(surface.input.input_method_owner, None);
    assert!(clear.rejected_effects.is_empty());

    surface.focus_node(UiNodeId::new(2)).unwrap();
    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let focus_change = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::SetFocus {
            target: UiNodeId::new(3),
            reason: UiFocusEffectReason::Input,
        }),
    );

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(3)));
    assert!(
        surface
            .component_state(UiNodeId::new(3))
            .unwrap()
            .flags
            .focused
    );
    assert!(
        !surface
            .component_state(UiNodeId::new(3))
            .unwrap()
            .flags
            .focus_visible
    );
    assert_eq!(surface.input.input_method_owner, None);
    assert!(focus_change.rejected_effects.is_empty());
}

#[test]
fn dispatch_reply_applies_navigation_and_host_owned_input_effects() {
    let mut surface = two_button_surface(None, None);
    surface.focus_node(UiNodeId::new(2)).unwrap();
    let request = UiInputMethodRequest {
        kind: UiInputMethodRequestKind::Enable,
        owner: UiNodeId::new(3),
        cursor_rect: Some(UiFrame::new(10.0, 50.0, 1.0, 20.0)),
        composition_rects: vec![UiFrame::new(10.0, 50.0, 30.0, 20.0)],
        surrounding_text: Some(UiInputMethodSurroundingText::new("foobar", 3, 3).unwrap()),
    };
    let reply = UiDispatchReply::handled().with_effects([
        UiDispatchEffect::RequestNavigation {
            kind: UiNavigationEventKind::Next,
            policy: UiNavigationRequestPolicy::Wrap,
        },
        UiDispatchEffect::LockPointer {
            target: UiNodeId::new(3),
            policy: UiPointerLockPolicy::RawDelta,
        },
        UiDispatchEffect::RequestInputMethod {
            request: request.clone(),
        },
    ]);

    let result = surface.apply_dispatch_reply(keyboard_event(), reply);

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(3)));
    assert_eq!(surface.input.pointer_lock_owner, Some(UiNodeId::new(3)));
    assert_eq!(surface.input.input_method_owner, Some(UiNodeId::new(3)));
    assert_eq!(surface.input.input_method_request, Some(request));
    assert_eq!(result.host_requests.len(), 2);
    assert!(matches!(
        result.host_requests[0].request,
        UiDispatchHostRequestKind::PointerLock { .. }
    ));
    assert!(matches!(
        result.host_requests[1].request,
        UiDispatchHostRequestKind::InputMethod(_)
    ));

    let disable = UiInputMethodRequest {
        kind: UiInputMethodRequestKind::Disable,
        owner: UiNodeId::new(3),
        cursor_rect: None,
        composition_rects: Vec::new(),
        surrounding_text: None,
    };
    let disabled = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::RequestInputMethod {
            request: disable.clone(),
        }),
    );

    assert_eq!(surface.input.input_method_owner, None);
    assert_eq!(surface.input.input_method_request, None);
    assert!(matches!(
        &disabled.host_requests[0].request,
        UiDispatchHostRequestKind::InputMethod(request) if request == &disable
    ));

    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let stale_disable = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled()
            .with_effect(UiDispatchEffect::RequestInputMethod { request: disable }),
    );

    assert_eq!(surface.input.input_method_owner, Some(UiNodeId::new(2)));
    assert!(stale_disable.host_requests.is_empty());
    assert_eq!(stale_disable.rejected_effects.len(), 1);
    assert_eq!(
        stale_disable.rejected_effects[0].reason,
        "input method owner mismatch"
    );

    surface
        .tree
        .nodes
        .get_mut(&UiNodeId::new(3))
        .unwrap()
        .state_flags
        .enabled = false;
    let invalid_enable = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::RequestInputMethod {
            request: UiInputMethodRequest {
                kind: UiInputMethodRequestKind::Enable,
                owner: UiNodeId::new(3),
                cursor_rect: None,
                composition_rects: Vec::new(),
                surrounding_text: None,
            },
        }),
    );

    assert_eq!(surface.input.input_method_owner, Some(UiNodeId::new(2)));
    assert!(invalid_enable.host_requests.is_empty());
    assert_eq!(invalid_enable.rejected_effects.len(), 1);
    assert!(invalid_enable.rejected_effects[0]
        .reason
        .starts_with("invalid input owner"));
}

#[test]
fn input_method_request_rejects_invalid_surrounding_text_before_host_request() {
    let mut surface = two_button_surface(None, None);
    let invalid = UiInputMethodRequest {
        kind: UiInputMethodRequestKind::Enable,
        owner: UiNodeId::new(3),
        cursor_rect: Some(UiFrame::new(10.0, 50.0, 1.0, 20.0)),
        composition_rects: Vec::new(),
        surrounding_text: Some(UiInputMethodSurroundingText {
            text: "你好".to_string(),
            cursor_byte: 1,
            anchor_byte: 1,
            composition_range: None,
        }),
    };

    let rejected = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled()
            .with_effect(UiDispatchEffect::RequestInputMethod { request: invalid }),
    );

    assert_eq!(surface.input.input_method_owner, None);
    assert!(rejected.host_requests.is_empty());
    assert_eq!(rejected.rejected_effects.len(), 1);
    assert!(
        rejected.rejected_effects[0]
            .reason
            .starts_with("invalid input method surrounding text"),
        "{}",
        rejected.rejected_effects[0].reason
    );
}
