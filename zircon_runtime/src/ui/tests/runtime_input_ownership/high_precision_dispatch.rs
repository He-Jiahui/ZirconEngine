use super::*;

#[test]
fn high_precision_requires_capture_and_release_clears_only_matching_owner() {
    let mut surface = two_button_surface();
    let pointer_id = UiPointerId::new(7);

    let rejected_enable = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::UseHighPrecisionPointer {
            target: UiNodeId::new(2),
            enabled: true,
        }),
    );

    assert_eq!(surface.input.high_precision_owner, None);
    assert!(rejected_enable.host_requests.is_empty());
    assert_eq!(rejected_enable.rejected_effects.len(), 1);
    assert_eq!(
        rejected_enable.rejected_effects[0].reason,
        "high precision requires pointer capture"
    );

    capture_pointer_for_test(&mut surface, pointer_id, UiNodeId::new(2));
    let enabled = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::UseHighPrecisionPointer {
            target: UiNodeId::new(2),
            enabled: true,
        }),
    );

    assert_eq!(surface.input.high_precision_owner, Some(UiNodeId::new(2)));
    assert!(enabled.rejected_effects.is_empty());
    assert_eq!(enabled.host_requests.len(), 1);

    let released = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::ReleasePointerCapture {
            target: UiNodeId::new(2),
            pointer_id,
            reason: UiPointerCaptureReason::Cancel,
        }),
    );

    assert_eq!(surface.focus.captured, None);
    assert_no_pointer_capture(&surface);
    assert_eq!(surface.input.high_precision_owner, None);
    assert!(released.rejected_effects.is_empty());

    capture_pointer_for_test(&mut surface, pointer_id, UiNodeId::new(2));
    surface.input.high_precision_owner = Some(UiNodeId::new(3));
    let divergent_release = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::ReleasePointerCapture {
            target: UiNodeId::new(2),
            pointer_id,
            reason: UiPointerCaptureReason::Cancel,
        }),
    );

    assert_eq!(surface.focus.captured, None);
    assert_no_pointer_capture(&surface);
    assert_eq!(surface.input.high_precision_owner, Some(UiNodeId::new(3)));
    assert!(divergent_release.rejected_effects.is_empty());

    capture_pointer_for_test(&mut surface, pointer_id, UiNodeId::new(2));
    surface.input.high_precision_owner = Some(UiNodeId::new(2));
    let transferred_capture = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::CapturePointer {
            target: UiNodeId::new(3),
            pointer_id: UiPointerId::new(9),
            reason: UiPointerCaptureReason::Press,
        }),
    );

    assert_eq!(surface.focus.captured, Some(UiNodeId::new(3)));
    assert_pointer_capture(&surface, UiPointerId::new(9), UiNodeId::new(3));
    assert_eq!(surface.input.high_precision_owner, None);
    assert!(transferred_capture.rejected_effects.is_empty());
}

#[test]
fn reply_step_route_stops_before_later_bubble_effects() {
    let mut surface = two_button_surface();
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
    assert!(result.rejected_effects.is_empty());
    assert_eq!(
        result.diagnostics.handled_phase,
        Some("preview_tunnel".to_string())
    );
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(1)));
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
}
