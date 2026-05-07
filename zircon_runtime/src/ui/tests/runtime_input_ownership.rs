use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
    tree::UiRuntimeTreeAccessExt,
};
use zircon_runtime_interface::ui::dispatch::{
    UiAnalogInputEvent, UiDispatchDisposition, UiDispatchEffect, UiDispatchPhase, UiDispatchReply,
    UiDispatchReplyStep, UiDragDropEffectKind, UiDragDropInputEvent, UiDragDropInputEventKind,
    UiDragSessionId, UiFocusEffectReason, UiInputEvent, UiInputEventMetadata, UiInputMethodRequest,
    UiInputMethodRequestKind, UiInputSequence, UiInputTimestamp, UiKeyboardInputEvent,
    UiKeyboardInputState, UiNavigationInputEvent, UiNavigationRequestPolicy,
    UiPointerCaptureReason, UiPointerId, UiPopupInputEvent, UiPopupInputEventKind,
    UiTooltipTimerInputEvent, UiTooltipTimerInputEventKind,
};
use zircon_runtime_interface::ui::{
    component::{UiDragPayload, UiDragPayloadKind},
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiFrame, UiPoint},
    surface::UiNavigationEventKind,
    tree::{UiInputPolicy, UiTreeNode, UiVisibility},
};

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
fn direct_capture_clears_stale_pointer_id_before_high_precision_can_enable() {
    let mut surface = two_button_surface();
    let stale_pointer_id = UiPointerId::new(7);
    surface.focus.captured = Some(UiNodeId::new(2));
    surface.input.captured_pointer_id = Some(stale_pointer_id);
    surface.input.high_precision_owner = Some(UiNodeId::new(2));

    surface.capture_pointer(UiNodeId::new(3)).unwrap();

    assert_eq!(surface.focus.captured, Some(UiNodeId::new(3)));
    assert_eq!(surface.input.captured_pointer_id, None);
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

    surface.focus.captured = Some(UiNodeId::new(2));
    surface.input.captured_pointer_id = Some(pointer_id);
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
    assert_eq!(surface.input.captured_pointer_id, None);
    assert_eq!(surface.input.high_precision_owner, None);
    assert!(released.rejected_effects.is_empty());

    surface.focus.captured = Some(UiNodeId::new(2));
    surface.input.captured_pointer_id = Some(pointer_id);
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
    assert_eq!(surface.input.captured_pointer_id, None);
    assert_eq!(surface.input.high_precision_owner, Some(UiNodeId::new(3)));
    assert!(divergent_release.rejected_effects.is_empty());

    surface.focus.captured = Some(UiNodeId::new(2));
    surface.input.captured_pointer_id = Some(pointer_id);
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
    assert_eq!(surface.input.captured_pointer_id, Some(UiPointerId::new(9)));
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

#[test]
fn drag_drop_lifecycle_tracks_shared_state_and_cleans_capture_on_end() {
    let mut surface = two_button_surface();
    let pointer_id = UiPointerId::new(7);
    let session_id = UiDragSessionId::new(42);
    let payload = UiDragPayload::new(UiDragPayloadKind::Asset, "res://assets/materials/brick.mat");

    let begin = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(drag_effect(
            UiDragDropEffectKind::Begin,
            UiNodeId::new(2),
            pointer_id,
            Some(session_id),
            Some(UiPoint::new(14.0, 18.0)),
            Some(payload.clone()),
        )),
    );

    assert!(begin.rejected_effects.is_empty());
    assert_eq!(surface.focus.captured, Some(UiNodeId::new(2)));
    assert_eq!(surface.input.captured_pointer_id, Some(pointer_id));
    let drag = surface.input.drag_drop.as_ref().expect("active drag");
    assert_eq!(drag.session_id, session_id);
    assert_eq!(drag.source, UiNodeId::new(2));
    assert_eq!(drag.target, UiNodeId::new(2));
    assert_eq!(drag.point, Some(UiPoint::new(14.0, 18.0)));
    assert_eq!(drag.payload, Some(payload.clone()));

    let update = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(drag_effect(
            UiDragDropEffectKind::Update,
            UiNodeId::new(3),
            pointer_id,
            Some(session_id),
            Some(UiPoint::new(44.0, 68.0)),
            None,
        )),
    );
    assert!(update.rejected_effects.is_empty());
    let drag = surface.input.drag_drop.as_ref().expect("updated drag");
    assert_eq!(drag.target, UiNodeId::new(3));
    assert_eq!(drag.point, Some(UiPoint::new(44.0, 68.0)));
    assert_eq!(drag.payload, Some(payload));

    let accept = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(drag_effect(
            UiDragDropEffectKind::Accept,
            UiNodeId::new(3),
            pointer_id,
            Some(session_id),
            None,
            None,
        )),
    );
    assert!(accept.rejected_effects.is_empty());
    assert!(
        surface
            .input
            .drag_drop
            .as_ref()
            .expect("accepted drag")
            .accepted
    );

    surface.input.high_precision_owner = Some(UiNodeId::new(2));
    let complete = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(drag_effect(
            UiDragDropEffectKind::Complete,
            UiNodeId::new(3),
            pointer_id,
            Some(session_id),
            None,
            None,
        )),
    );

    assert!(complete.rejected_effects.is_empty());
    assert_eq!(surface.input.drag_drop, None);
    assert_eq!(surface.focus.captured, None);
    assert_eq!(surface.input.captured_pointer_id, None);
    assert_eq!(surface.input.high_precision_owner, None);
}

#[test]
fn drag_drop_rejects_stale_pointer_or_session_without_clearing_active_drag() {
    let mut surface = two_button_surface();
    let pointer_id = UiPointerId::new(7);
    let session_id = UiDragSessionId::new(42);

    let begin = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(drag_effect(
            UiDragDropEffectKind::Begin,
            UiNodeId::new(2),
            pointer_id,
            Some(session_id),
            Some(UiPoint::new(12.0, 16.0)),
            None,
        )),
    );
    assert!(begin.rejected_effects.is_empty());

    let stale_session = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(drag_effect(
            UiDragDropEffectKind::Update,
            UiNodeId::new(3),
            pointer_id,
            Some(UiDragSessionId::new(99)),
            Some(UiPoint::new(50.0, 70.0)),
            None,
        )),
    );
    assert_eq!(stale_session.rejected_effects.len(), 1);
    assert_eq!(
        stale_session.rejected_effects[0].reason,
        "drag session owner mismatch"
    );
    let drag = surface
        .input
        .drag_drop
        .as_ref()
        .expect("drag remains active");
    assert_eq!(drag.target, UiNodeId::new(2));
    assert_eq!(drag.point, Some(UiPoint::new(12.0, 16.0)));

    let stale_pointer = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(drag_effect(
            UiDragDropEffectKind::Cancel,
            UiNodeId::new(2),
            UiPointerId::new(99),
            Some(session_id),
            None,
            None,
        )),
    );
    assert_eq!(stale_pointer.rejected_effects.len(), 1);
    assert_eq!(
        stale_pointer.rejected_effects[0].reason,
        "drag pointer owner mismatch"
    );
    assert!(surface.input.drag_drop.is_some());
}

#[test]
fn shared_input_dispatch_applies_drag_drop_popup_and_tooltip_events_through_effects() {
    let mut surface = two_button_surface();
    let pointer_dispatcher = UiPointerDispatcher::default();
    let navigation_dispatcher = UiNavigationDispatcher::default();
    let pointer_id = UiPointerId::new(7);
    let session_id = UiDragSessionId::new(42);
    let payload = UiDragPayload::new(UiDragPayloadKind::Asset, "res://assets/materials/brick.mat");

    let begin = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            drag_drop_input_event(
                UiDragDropInputEventKind::Begin,
                Some(session_id),
                UiPoint::new(20.0, 20.0),
                Some(payload.clone()),
            ),
        )
        .unwrap();

    assert!(begin.rejected_effects.is_empty());
    assert_eq!(begin.applied_effects.len(), 1);
    assert_eq!(surface.focus.captured, Some(UiNodeId::new(2)));
    assert_eq!(surface.input.captured_pointer_id, Some(pointer_id));
    let drag = surface.input.drag_drop.as_ref().expect("active drag");
    assert_eq!(drag.session_id, session_id);
    assert_eq!(drag.source, UiNodeId::new(2));
    assert_eq!(drag.target, UiNodeId::new(2));
    assert_eq!(drag.point, Some(UiPoint::new(20.0, 20.0)));
    assert_eq!(drag.payload, Some(payload.clone()));

    let over = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            drag_drop_input_event(
                UiDragDropInputEventKind::Over,
                Some(session_id),
                UiPoint::new(20.0, 60.0),
                None,
            ),
        )
        .unwrap();
    assert!(over.rejected_effects.is_empty());
    let drag = surface.input.drag_drop.as_ref().expect("updated drag");
    assert_eq!(drag.target, UiNodeId::new(3));
    assert_eq!(drag.point, Some(UiPoint::new(20.0, 60.0)));
    assert_eq!(drag.payload, Some(payload));

    let drop = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            drag_drop_input_event(
                UiDragDropInputEventKind::Drop,
                Some(session_id),
                UiPoint::new(20.0, 60.0),
                None,
            ),
        )
        .unwrap();
    assert!(drop.rejected_effects.is_empty());
    assert!(
        surface
            .input
            .drag_drop
            .as_ref()
            .expect("accepted drag")
            .accepted
    );

    let end = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            drag_drop_input_event(
                UiDragDropInputEventKind::End,
                Some(session_id),
                UiPoint::new(20.0, 60.0),
                None,
            ),
        )
        .unwrap();
    assert!(end.rejected_effects.is_empty());
    assert_eq!(surface.input.drag_drop, None);
    assert_eq!(surface.focus.captured, None);
    assert_eq!(surface.input.captured_pointer_id, None);

    let popup_open = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            popup_input_event(
                UiPopupInputEventKind::OpenRequested,
                "menu.file",
                Some(UiPoint::new(8.0, 12.0)),
            ),
        )
        .unwrap();
    assert!(popup_open.rejected_effects.is_empty());
    assert_eq!(popup_open.host_requests.len(), 1);
    assert_eq!(surface.input.popup_stack.len(), 1);
    assert_eq!(surface.input.popup_stack[0].popup_id, "menu.file");

    surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            popup_input_event(UiPopupInputEventKind::Dismissed, "menu.file", None),
        )
        .unwrap();
    assert!(surface.input.popup_stack.is_empty());

    surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            tooltip_input_event(UiTooltipTimerInputEventKind::Armed, "asset.tooltip"),
        )
        .unwrap();
    assert_eq!(
        surface
            .input
            .tooltip
            .as_ref()
            .map(|tooltip| tooltip.visible),
        Some(false)
    );

    let tooltip_shown = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            tooltip_input_event(UiTooltipTimerInputEventKind::Elapsed, "asset.tooltip"),
        )
        .unwrap();
    assert!(tooltip_shown.rejected_effects.is_empty());
    assert_eq!(
        surface
            .input
            .tooltip
            .as_ref()
            .map(|tooltip| tooltip.visible),
        Some(true)
    );

    surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            tooltip_input_event(UiTooltipTimerInputEventKind::Canceled, "asset.tooltip"),
        )
        .unwrap();
    assert_eq!(surface.input.tooltip, None);
}

#[test]
fn analog_input_suppresses_repeated_values_before_routing() {
    let mut surface = two_button_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let first = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            analog_event("gamepad.left_x", 0.5),
        )
        .unwrap();
    assert!(first.diagnostics.routed);
    assert_eq!(first.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(first.reply.disposition, UiDispatchDisposition::Handled);

    let repeated = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            analog_event("gamepad.left_x", 0.5004),
        )
        .unwrap();
    assert!(!repeated.diagnostics.routed);
    assert_eq!(repeated.reply.disposition, UiDispatchDisposition::Unhandled);
    assert!(repeated
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "analog_repeat_suppressed"));
    assert_eq!(
        surface
            .input
            .analog_controls
            .get("gamepad.left_x")
            .map(|state| state.value),
        Some(0.5)
    );

    let changed = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            analog_event("gamepad.left_x", 0.75),
        )
        .unwrap();
    assert!(changed.diagnostics.routed);
    assert_eq!(changed.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        surface
            .input
            .analog_controls
            .get("gamepad.left_x")
            .map(|state| state.value),
        Some(0.75)
    );
}

fn two_button_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.input.owner"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 100.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/first"))
                .with_frame(UiFrame::new(10.0, 10.0, 80.0, 30.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state()),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/second"))
                .with_frame(UiFrame::new(10.0, 50.0, 80.0, 30.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state()),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn input_metadata() -> UiInputEventMetadata {
    let mut metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(10), UiInputSequence::new(1));
    metadata.pointer_id = Some(UiPointerId::new(7));
    metadata
}

fn keyboard_event() -> UiInputEvent {
    UiInputEvent::Keyboard(UiKeyboardInputEvent {
        metadata: input_metadata(),
        state: UiKeyboardInputState::Pressed,
        key_code: 65,
        scan_code: Some(30),
        physical_key: "KeyA".to_string(),
        logical_key: "A".to_string(),
        text: Some("a".to_string()),
    })
}

fn analog_event(control: &str, value: f32) -> UiInputEvent {
    UiInputEvent::Analog(UiAnalogInputEvent {
        metadata: input_metadata(),
        control: control.to_string(),
        value,
    })
}

fn drag_drop_input_event(
    kind: UiDragDropInputEventKind,
    session_id: Option<UiDragSessionId>,
    point: UiPoint,
    payload: Option<UiDragPayload>,
) -> UiInputEvent {
    UiInputEvent::DragDrop(UiDragDropInputEvent {
        metadata: input_metadata(),
        kind,
        session_id,
        point,
        payload,
    })
}

fn popup_input_event(
    kind: UiPopupInputEventKind,
    popup_id: &str,
    anchor: Option<UiPoint>,
) -> UiInputEvent {
    UiInputEvent::Popup(UiPopupInputEvent {
        metadata: input_metadata(),
        kind,
        popup_id: popup_id.to_string(),
        anchor,
    })
}

fn tooltip_input_event(kind: UiTooltipTimerInputEventKind, tooltip_id: &str) -> UiInputEvent {
    UiInputEvent::TooltipTimer(UiTooltipTimerInputEvent {
        metadata: input_metadata(),
        kind,
        tooltip_id: tooltip_id.to_string(),
    })
}

fn drag_effect(
    kind: UiDragDropEffectKind,
    target: UiNodeId,
    pointer_id: UiPointerId,
    session_id: Option<UiDragSessionId>,
    point: Option<UiPoint>,
    payload: Option<UiDragPayload>,
) -> UiDispatchEffect {
    UiDispatchEffect::DragDrop {
        kind,
        target,
        pointer_id,
        session_id,
        point,
        payload,
    }
}

fn input_method_request(kind: UiInputMethodRequestKind, owner: UiNodeId) -> UiInputMethodRequest {
    UiInputMethodRequest {
        kind,
        owner,
        cursor_rect: None,
        composition_rects: Vec::new(),
    }
}

fn input_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: true,
        pressed: false,
        checked: false,
        dirty: false,
    }
}
