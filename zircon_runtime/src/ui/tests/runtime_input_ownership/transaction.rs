use super::*;

#[test]
fn input_transaction_single_effect_hot_path_skips_atomic_snapshot() {
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
        "ordinary single effects must not enter the snapshot transaction path"
    );
}

#[test]
fn input_transaction_tail_rejection_rolls_back_focus_and_capture_prefix() {
    let mut surface = two_button_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();
    capture_pointer_for_test(&mut surface, UiPointerId::new(7), UiNodeId::new(2));
    surface.input.high_precision_owner = Some(UiNodeId::new(2));
    let focus_before = surface.focus.clone();
    let input_before = surface.input.clone();
    let component_states_before = surface.component_states.clone();

    let result = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled()
            .with_effect(UiDispatchEffect::SetFocus {
                target: UiNodeId::new(3),
                reason: UiFocusEffectReason::Input,
            })
            .with_effect(UiDispatchEffect::CapturePointer {
                target: UiNodeId::new(3),
                pointer_id: UiPointerId::new(7),
                reason: UiPointerCaptureReason::Programmatic,
            })
            .with_effect(drag_effect(
                UiDragDropEffectKind::Begin,
                UiNodeId::new(99),
                UiPointerId::new(7),
                Some(UiDragSessionId::new(41)),
                None,
                None,
            )),
    );

    assert!(result.applied_effects.is_empty());
    assert!(result.host_requests.is_empty());
    assert!(result.component_events.is_empty());
    assert_eq!(result.rejected_effects.len(), 3);
    assert_eq!(surface.focus, focus_before);
    assert_eq!(surface.input, input_before);
    assert_eq!(surface.component_states, component_states_before);
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
    let focus_before = surface.focus.clone();
    let input_before = surface.input.clone();
    let component_states_before = surface.component_states.clone();

    let result = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(drag_effect(
            UiDragDropEffectKind::Complete,
            UiNodeId::new(2),
            pointer_id,
            Some(session_id),
            None,
            None,
        )),
    );

    assert!(result.applied_effects.is_empty());
    assert_eq!(result.rejected_effects.len(), 1);
    assert_eq!(surface.focus, focus_before);
    assert_eq!(surface.input, input_before);
    assert_eq!(surface.component_states, component_states_before);
}
