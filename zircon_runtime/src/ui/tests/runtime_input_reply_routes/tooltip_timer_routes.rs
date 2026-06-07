use super::*;

#[test]
fn tooltip_cancel_reply_uses_armed_tooltip_owner_for_route_trace() {
    let mut surface = route_surface();

    surface.apply_dispatch_reply(
        tooltip_event(
            UiTooltipTimerInputEventKind::Armed,
            "status.hint",
            Some(UiNodeId::new(2)),
        ),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::Tooltip {
            kind: UiTooltipEffectKind::Arm,
            tooltip_id: "status.hint".to_string(),
            owner: Some(UiNodeId::new(2)),
        }),
    );

    let result = surface.apply_dispatch_reply(
        tooltip_event(UiTooltipTimerInputEventKind::Canceled, "status.hint", None),
        UiDispatchReply::handled()
            .in_phase(UiDispatchPhase::DefaultAction)
            .with_effect(UiDispatchEffect::Tooltip {
                kind: UiTooltipEffectKind::Cancel,
                tooltip_id: "status.hint".to_string(),
                owner: None,
            }),
    );

    assert!(result.rejected_effects.is_empty());
    assert_eq!(surface.input.tooltip, None);
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
}

#[test]
fn tooltip_timer_elapsed_dispatch_reports_owner_default_action_route() {
    let mut surface = route_surface();

    arm_status_hint(&mut surface);
    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            tooltip_event(
                UiTooltipTimerInputEventKind::Elapsed,
                "status.hint",
                Some(UiNodeId::new(2)),
            ),
        )
        .unwrap();

    assert!(result.rejected_effects.is_empty());
    assert_eq!(
        surface
            .input
            .tooltip
            .as_ref()
            .map(|tooltip| (tooltip.owner, tooltip.visible)),
        Some((Some(UiNodeId::new(2)), true))
    );
    assert_eq!(result.host_requests.len(), 1);
    assert!(matches!(
        result.host_requests[0].request,
        zircon_runtime_interface::ui::dispatch::UiDispatchHostRequestKind::Tooltip {
            kind: UiTooltipEffectKind::Show,
            ref tooltip_id,
        } if tooltip_id == "status.hint"
    ));
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::DefaultAction
    );
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("tooltip.effect")
    );
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(result.diagnostics.route_steps.len(), 1);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::DefaultAction
    );
    assert_eq!(
        result.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[0].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(result.diagnostics.route_steps[0].effect_count, 1);
}

#[test]
fn stale_tooltip_timer_elapsed_does_not_replace_current_retained_tooltip() {
    let mut surface = route_surface();

    arm_status_hint(&mut surface);
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            tooltip_event(
                UiTooltipTimerInputEventKind::Armed,
                "inspector.hint",
                Some(UiNodeId::new(3)),
            ),
        )
        .unwrap();

    let stale_elapsed = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            tooltip_event(
                UiTooltipTimerInputEventKind::Elapsed,
                "status.hint",
                Some(UiNodeId::new(2)),
            ),
        )
        .unwrap();

    assert_eq!(
        stale_elapsed.reply.disposition,
        UiDispatchDisposition::Unhandled
    );
    assert!(stale_elapsed.applied_effects.is_empty());
    assert!(stale_elapsed.host_requests.is_empty());
    assert!(stale_elapsed.rejected_effects.is_empty());
    assert!(stale_elapsed
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "stale_tooltip_timer_ignored"));
    assert_eq!(
        stale_elapsed.diagnostics.handled_phase.as_deref(),
        Some("tooltip.stale")
    );
    assert_eq!(
        stale_elapsed.diagnostics.route_policy,
        UiInputRoutePolicy::DefaultAction
    );
    assert_eq!(
        surface.input.tooltip.as_ref().map(|tooltip| (
            tooltip.tooltip_id.as_str(),
            tooltip.owner,
            tooltip.visible
        )),
        Some(("inspector.hint", Some(UiNodeId::new(3)), false))
    );
}

#[test]
fn stale_tooltip_timer_cancel_does_not_clear_current_retained_tooltip() {
    let mut surface = route_surface();

    arm_status_hint(&mut surface);

    let stale_cancel = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            tooltip_event(
                UiTooltipTimerInputEventKind::Canceled,
                "other.hint",
                Some(UiNodeId::new(2)),
            ),
        )
        .unwrap();

    assert_eq!(
        stale_cancel.reply.disposition,
        UiDispatchDisposition::Unhandled
    );
    assert!(stale_cancel.applied_effects.is_empty());
    assert!(stale_cancel.host_requests.is_empty());
    assert!(stale_cancel.rejected_effects.is_empty());
    assert!(stale_cancel
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "stale_tooltip_timer_ignored"));
    assert_eq!(
        stale_cancel.diagnostics.handled_phase.as_deref(),
        Some("tooltip.stale")
    );
    assert_eq!(
        stale_cancel.diagnostics.route_policy,
        UiInputRoutePolicy::DefaultAction
    );
    assert_eq!(
        surface.input.tooltip.as_ref().map(|tooltip| (
            tooltip.tooltip_id.as_str(),
            tooltip.owner,
            tooltip.visible
        )),
        Some(("status.hint", Some(UiNodeId::new(2)), false))
    );
}

fn arm_status_hint(surface: &mut UiSurface) {
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            tooltip_event(
                UiTooltipTimerInputEventKind::Armed,
                "status.hint",
                Some(UiNodeId::new(2)),
            ),
        )
        .unwrap();
}
