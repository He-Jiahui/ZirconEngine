use super::*;

#[test]
fn direct_dispatch_reply_populates_focus_route_trace_after_effects() {
    let mut surface = route_surface();

    let result = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::SetFocus {
            target: UiNodeId::new(2),
            reason: UiFocusEffectReason::Input,
        }),
    );

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
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
        result.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(result.diagnostics.route_steps.len(), 3);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_steps[0].disposition,
        UiDispatchDisposition::Passthrough
    );
    assert_eq!(
        result.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[1].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].phase,
        UiDispatchPhase::Target
    );
    assert_eq!(
        result.diagnostics.route_steps[2].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].handler,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(result.diagnostics.route_steps[2].effect_start, 0);
    assert_eq!(result.diagnostics.route_steps[2].effect_count, 1);
    assert!(result.diagnostics.route_steps[2].stopped);
}

#[test]
fn raw_mouse_motion_is_unrouted_by_surface_hit_testing() {
    let mut surface = route_surface();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            raw_mouse_motion_event(-3.5, 2.25),
        )
        .unwrap();

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::Unrouted
    );
    assert_eq!(result.diagnostics.routed, false);
    assert_eq!(result.diagnostics.route_target, None);
    assert_eq!(result.diagnostics.route_trace.target, None);
    assert!(result.diagnostics.route_steps.is_empty());
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "raw_mouse_motion"));
    assert!(matches!(
        result.event,
        UiInputEvent::MouseMotion(motion)
            if motion.delta_x == -3.5 && motion.delta_y == 2.25
    ));
}

#[test]
fn dispatch_reply_steps_report_stopped_preview_and_focus_trace() {
    let mut surface = route_surface();

    let result = surface.apply_dispatch_reply_steps(
        keyboard_event(),
        [
            UiDispatchReplyStep::new(
                UiDispatchPhase::Preprocess,
                None,
                UiDispatchReply::unhandled(),
            ),
            UiDispatchReplyStep::new(
                UiDispatchPhase::PreviewTunnel,
                Some(UiNodeId::new(1)),
                UiDispatchReply::handled().with_effect(UiDispatchEffect::SetFocus {
                    target: UiNodeId::new(2),
                    reason: UiFocusEffectReason::Input,
                }),
            ),
            UiDispatchReplyStep::new(
                UiDispatchPhase::Bubble,
                Some(UiNodeId::new(3)),
                UiDispatchReply::handled().with_effect(UiDispatchEffect::SetFocus {
                    target: UiNodeId::new(3),
                    reason: UiFocusEffectReason::Input,
                }),
            ),
        ],
    );

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(result.applied_effects.len(), 1);
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert_eq!(
        result.diagnostics.handled_phase,
        Some("preview_tunnel".to_string())
    );
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(1)));
    assert_eq!(
        result.diagnostics.route_trace.target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "dispatch_steps=2"));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "propagation_stopped"));
    assert_eq!(result.diagnostics.route_steps.len(), 2);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::Preprocess
    );
    assert_eq!(result.diagnostics.route_steps[0].target, None);
    assert_eq!(result.diagnostics.route_steps[0].handler, None);
    assert_eq!(
        result.diagnostics.route_steps[0].disposition,
        UiDispatchDisposition::Unhandled
    );
    assert_eq!(result.diagnostics.route_steps[0].effect_start, 0);
    assert_eq!(result.diagnostics.route_steps[0].effect_count, 0);
    assert!(!result.diagnostics.route_steps[0].stopped);
    assert_eq!(
        result.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[1].target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_steps[1].handler,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_steps[1].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(result.diagnostics.route_steps[1].effect_start, 0);
    assert_eq!(result.diagnostics.route_steps[1].effect_count, 1);
    assert!(result.diagnostics.route_steps[1].stopped);
}
