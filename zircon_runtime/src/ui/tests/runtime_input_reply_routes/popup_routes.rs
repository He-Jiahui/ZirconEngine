use super::*;

#[test]
fn popup_dispatch_reply_trace_includes_capture_and_updated_popup_stack() {
    let mut surface = route_surface();
    surface.focus.captured = Some(UiNodeId::new(2));
    surface.input.captured_pointer_id = Some(UiPointerId::new(7));

    let result = surface.apply_dispatch_reply(
        popup_event(UiPopupInputEventKind::OpenRequested, "menu.file"),
        UiDispatchReply::handled()
            .in_phase(UiDispatchPhase::DefaultAction)
            .with_effect(UiDispatchEffect::Popup {
                kind: UiPopupEffectKind::Open,
                popup_id: "menu.file".to_string(),
                owner: Some(UiNodeId::new(2)),
                anchor: Some(UiPoint::new(8.0, 12.0)),
            }),
    );

    assert!(result.rejected_effects.is_empty());
    assert_eq!(
        surface
            .input
            .popup_stack
            .iter()
            .map(|popup| popup.popup_id.as_str())
            .collect::<Vec<_>>(),
        vec!["menu.file"]
    );
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::DefaultAction
    );
    assert_eq!(
        result.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.capture_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.popup_stack,
        vec!["menu.file".to_string()]
    );
}

#[test]
fn popup_close_reply_uses_open_popup_owner_for_route_trace() {
    let mut surface = route_surface();
    surface.focus.captured = Some(UiNodeId::new(2));
    surface.input.captured_pointer_id = Some(UiPointerId::new(7));

    surface.apply_dispatch_reply(
        popup_event(UiPopupInputEventKind::OpenRequested, "menu.file"),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::Popup {
            kind: UiPopupEffectKind::Open,
            popup_id: "menu.file".to_string(),
            owner: Some(UiNodeId::new(2)),
            anchor: Some(UiPoint::new(8.0, 12.0)),
        }),
    );

    let result = surface.apply_dispatch_reply(
        popup_event_without_owner(UiPopupInputEventKind::CloseRequested, "menu.file"),
        UiDispatchReply::handled()
            .in_phase(UiDispatchPhase::DefaultAction)
            .with_effect(UiDispatchEffect::Popup {
                kind: UiPopupEffectKind::Close,
                popup_id: "menu.file".to_string(),
                owner: None,
                anchor: None,
            }),
    );

    assert!(result.rejected_effects.is_empty());
    assert!(surface.input.popup_stack.is_empty());
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(
        result.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(2)]
    );
    assert_eq!(
        result.diagnostics.route_trace.capture_target,
        Some(UiNodeId::new(2))
    );
}

#[test]
fn stale_popup_close_event_does_not_emit_host_request_after_popup_replaced() {
    let mut surface = route_surface();

    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            popup_event(UiPopupInputEventKind::OpenRequested, "menu.file"),
        )
        .unwrap();
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            popup_event(UiPopupInputEventKind::OpenRequested, "menu.edit"),
        )
        .unwrap();

    let stale_close = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            popup_event(UiPopupInputEventKind::CloseRequested, "menu.view"),
        )
        .unwrap();

    assert_eq!(
        stale_close.reply.disposition,
        UiDispatchDisposition::Unhandled
    );
    assert!(stale_close.applied_effects.is_empty());
    assert!(stale_close.host_requests.is_empty());
    assert!(stale_close.rejected_effects.is_empty());
    assert!(stale_close
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "stale_popup_event_ignored"));
    assert_eq!(
        stale_close.diagnostics.handled_phase.as_deref(),
        Some("popup.stale")
    );
    assert_eq!(
        stale_close.diagnostics.route_policy,
        UiInputRoutePolicy::DefaultAction
    );
    assert_eq!(
        surface
            .input
            .popup_stack
            .iter()
            .map(|popup| popup.popup_id.as_str())
            .collect::<Vec<_>>(),
        vec!["menu.file", "menu.edit"]
    );
}

#[test]
fn transient_dismissal_reply_closes_popup_stack_and_tooltip_with_host_request() {
    let mut surface = route_surface();

    surface.apply_dispatch_reply(
        popup_event(UiPopupInputEventKind::OpenRequested, "menu.file"),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::Popup {
            kind: UiPopupEffectKind::Open,
            popup_id: "menu.file".to_string(),
            owner: Some(UiNodeId::new(2)),
            anchor: Some(UiPoint::new(8.0, 12.0)),
        }),
    );
    surface.apply_dispatch_reply(
        popup_event(UiPopupInputEventKind::OpenRequested, "menu.edit"),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::Popup {
            kind: UiPopupEffectKind::Open,
            popup_id: "menu.edit".to_string(),
            owner: Some(UiNodeId::new(3)),
            anchor: Some(UiPoint::new(24.0, 12.0)),
        }),
    );
    surface.apply_dispatch_reply(
        tooltip_event(
            UiTooltipTimerInputEventKind::Elapsed,
            "status.hint",
            Some(UiNodeId::new(2)),
        ),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::Tooltip {
            kind: UiTooltipEffectKind::Show,
            tooltip_id: "status.hint".to_string(),
            owner: Some(UiNodeId::new(2)),
        }),
    );

    assert_eq!(
        surface
            .input
            .popup_stack
            .iter()
            .map(|popup| popup.popup_id.as_str())
            .collect::<Vec<_>>(),
        vec!["menu.file", "menu.edit"]
    );
    assert!(surface.input.tooltip.is_some());

    let result = surface.apply_dispatch_reply(
        popup_event_without_owner(UiPopupInputEventKind::Dismissed, "host.transient"),
        UiDispatchReply::handled()
            .in_phase(UiDispatchPhase::DefaultAction)
            .with_effect(UiDispatchEffect::DismissTransientUi {
                target: UiTransientDismissalTarget::All,
                reason: UiTransientDismissalReason::WindowAction,
            }),
    );

    assert!(result.rejected_effects.is_empty());
    assert!(surface.input.popup_stack.is_empty());
    assert!(surface.input.tooltip.is_none());
    assert_eq!(result.applied_effects.len(), 1);
    assert!(matches!(
        result.applied_effects[0].effect,
        UiDispatchEffect::DismissTransientUi {
            target: UiTransientDismissalTarget::All,
            reason: UiTransientDismissalReason::WindowAction,
        }
    ));
    assert_eq!(result.host_requests.len(), 1);
    assert!(matches!(
        result.host_requests[0].request,
        UiDispatchHostRequestKind::DismissTransientUi {
            target: UiTransientDismissalTarget::All,
            reason: UiTransientDismissalReason::WindowAction,
        }
    ));
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::DefaultAction
    );
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("default_action")
    );
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(3)));
    assert_eq!(
        result.diagnostics.route_trace.popup_stack,
        Vec::<String>::new()
    );
}
