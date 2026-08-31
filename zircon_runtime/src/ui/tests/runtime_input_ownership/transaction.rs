use super::*;
use zircon_runtime_interface::ui::dispatch::{
    UiInputDispatchResult, UiInputRoutePolicy, UiPopupEffectKind,
};
use zircon_runtime_interface::ui::text::UiRichLinkTarget;

fn assert_transaction_rejected(
    result: &UiInputDispatchResult,
    effects: &[UiDispatchEffect],
    failed_effect_index: usize,
    failed_reason_fragment: &str,
) {
    assert!(result.applied_effects.is_empty());
    assert!(result.host_requests.is_empty());
    assert!(result.component_events.is_empty());
    assert_eq!(result.rejected_effects.len(), effects.len());
    for (effect_index, rejected) in result.rejected_effects.iter().enumerate() {
        assert_eq!(rejected.effect_index, effect_index);
        assert_eq!(rejected.effect, effects[effect_index]);
        if effect_index == failed_effect_index {
            assert!(rejected.reason.contains(failed_reason_fragment));
        } else {
            assert!(rejected.reason.contains(&format!(
                "input transaction aborted because effect {failed_effect_index} was rejected"
            )));
            assert!(rejected.reason.contains(failed_reason_fragment));
        }
    }
    assert!(result.diagnostics.notes.iter().any(|note| {
        note.starts_with("input_transaction=aborted")
            && note.contains("base_generation=")
            && note.contains(&format!("failed_effect={failed_effect_index}"))
    }));
}

#[test]
fn input_transaction_single_effect_hot_path_captures_no_domains() {
    let mut surface = two_button_surface();

    let result = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::CapturePointer {
            target: UiNodeId::new(2),
            pointer_id: UiPointerId::new(7),
            reason: UiPointerCaptureReason::Programmatic,
        }),
    );

    assert_eq!(result.applied_effects.len(), 1);
    assert!(result.rejected_effects.is_empty());
    assert!(
        result
            .diagnostics
            .notes
            .iter()
            .all(|note| !note.starts_with("input_transaction=")),
        "ordinary single effects must not capture transaction domains"
    );
}

#[test]
fn input_transaction_tail_rejection_rolls_back_focus_and_capture_prefix() {
    let mut surface = two_button_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();
    capture_pointer_for_test(&mut surface, UiPointerId::new(7), UiNodeId::new(2));
    surface.input.high_precision_owner = Some(UiNodeId::new(2));
    let surface_before = surface.clone();
    let effects = vec![
        UiDispatchEffect::SetFocus {
            target: UiNodeId::new(3),
            reason: UiFocusEffectReason::Input,
        },
        UiDispatchEffect::CapturePointer {
            target: UiNodeId::new(3),
            pointer_id: UiPointerId::new(7),
            reason: UiPointerCaptureReason::Programmatic,
        },
        drag_effect(
            UiDragDropEffectKind::Begin,
            UiNodeId::new(99),
            UiPointerId::new(7),
            Some(UiDragSessionId::new(41)),
            None,
            None,
        ),
    ];

    let result = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled()
            .from_handler(UiNodeId::new(2))
            .with_effects(effects.clone()),
    );

    assert_transaction_rejected(&result, &effects, 2, "missing node");
    assert_eq!(surface, surface_before);
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert_eq!(
        result.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
}

#[test]
fn input_transaction_drag_composite_failure_restores_all_mutated_domains() {
    let mut surface = two_button_surface();
    let missing_source = UiNodeId::new(99);
    let pointer_id = UiPointerId::new(7);
    let session_id = UiDragSessionId::new(42);
    surface
        .input
        .begin_drag_drop(
            missing_source,
            missing_source,
            pointer_id,
            Some(session_id),
            None,
            None,
        )
        .unwrap();
    surface.focus.captured = Some(missing_source);
    capture_pointer_for_test(&mut surface, pointer_id, missing_source);
    surface.component_states.set_dragging(missing_source, true);
    let surface_before = surface.clone();
    let effects = vec![drag_effect(
        UiDragDropEffectKind::Complete,
        UiNodeId::new(2),
        pointer_id,
        Some(session_id),
        None,
        None,
    )];

    let result = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effects(effects.clone()),
    );

    assert_transaction_rejected(&result, &effects, 0, "missing node");
    assert_eq!(surface, surface_before);
}

#[test]
fn input_transaction_read_only_owner_failure_restores_deferred_input_lifecycle() {
    let mut surface = two_button_surface();
    surface.input.queue_focus_input_lifecycle(
        None,
        input_method_request(UiInputMethodRequestKind::Enable, UiNodeId::new(2)),
    );
    let surface_before = surface.clone();
    let effects = vec![
        UiDispatchEffect::RequestLinkActivation {
            target: UiNodeId::new(2),
            link_target: UiRichLinkTarget::parse("res://docs/accepted").unwrap(),
        },
        UiDispatchEffect::RequestLinkActivation {
            target: UiNodeId::new(99),
            link_target: UiRichLinkTarget::parse("res://docs/missing-owner").unwrap(),
        },
    ];

    let result = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effects(effects.clone()),
    );

    assert_transaction_rejected(&result, &effects, 1, "invalid input owner");
    assert_eq!(surface, surface_before);
}

#[test]
fn input_transaction_popup_prefix_restores_surface_and_route_trace() {
    let mut surface = two_button_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();
    let surface_before = surface.clone();
    let effects = vec![
        UiDispatchEffect::Popup {
            kind: UiPopupEffectKind::Open,
            popup_id: "transaction.popup".to_string(),
            owner: Some(UiNodeId::new(2)),
            anchor: Some(UiPoint::new(16.0, 24.0)),
        },
        UiDispatchEffect::RequestLinkActivation {
            target: UiNodeId::new(99),
            link_target: UiRichLinkTarget::parse("res://docs/missing-owner").unwrap(),
        },
    ];

    let result = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled()
            .from_handler(UiNodeId::new(2))
            .with_effects(effects.clone()),
    );

    assert_transaction_rejected(&result, &effects, 1, "invalid input owner");
    assert_eq!(surface, surface_before);
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert!(result.diagnostics.route_trace.popup_stack.is_empty());
}
