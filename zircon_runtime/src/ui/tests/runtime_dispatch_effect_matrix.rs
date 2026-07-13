use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiClipboardRequest, UiClipboardRequestKind, UiComponentEmissionPolicy, UiDispatchEffect,
        UiDispatchHostRequestKind, UiDispatchReply, UiDragDropEffectKind, UiDragSessionId,
        UiFocusEffectReason, UiInputDispatchResult, UiInputEvent, UiInputEventMetadata,
        UiInputMethodRequest, UiInputMethodRequestKind, UiInputMethodSurroundingText,
        UiInputSequence, UiInputTimestamp, UiKeyboardInputEvent, UiKeyboardInputState,
        UiNavigationRequestPolicy, UiPointerCaptureReason, UiPointerId, UiPointerLockPolicy,
        UiPopupEffectKind, UiRedrawRequestReason, UiTooltipEffectKind, UiTransientDismissalReason,
        UiTransientDismissalTarget,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiFrame, UiPoint},
    surface::UiNavigationEventKind,
    tree::{UiDirtyFlags, UiInputPolicy, UiTreeNode},
};

#[test]
fn dispatch_effect_matrix_applies_focus_pointer_and_redraw_variants() {
    let mut surface = effect_matrix_surface("runtime.ui.effect_matrix.pointer");

    let result = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effects([
            UiDispatchEffect::SetFocus {
                target: node(2),
                reason: UiFocusEffectReason::Input,
            },
            UiDispatchEffect::CapturePointer {
                target: node(2),
                pointer_id: pointer(7),
                reason: UiPointerCaptureReason::Press,
            },
            UiDispatchEffect::UseHighPrecisionPointer {
                target: node(2),
                enabled: true,
            },
            UiDispatchEffect::ReleasePointerCapture {
                target: node(2),
                pointer_id: pointer(7),
                reason: UiPointerCaptureReason::Cancel,
            },
            UiDispatchEffect::LockPointer {
                target: node(2),
                policy: UiPointerLockPolicy::RawDelta,
            },
            UiDispatchEffect::UnlockPointer {
                target: node(2),
                policy: UiPointerLockPolicy::RawDelta,
            },
            UiDispatchEffect::DirtyRedraw {
                target: node(2),
                dirty: render_dirty(),
                reason: UiRedrawRequestReason::Input,
            },
            UiDispatchEffect::ClearFocus {
                target: node(2),
                reason: UiFocusEffectReason::Dismissal,
            },
        ]),
    );

    assert!(result.rejected_effects.is_empty());
    assert_eq!(applied_indices(&result), (0..8).collect::<Vec<_>>());
    assert_eq!(surface.focus.focused, None);
    assert_eq!(surface.focus.captured, None);
    assert_eq!(surface.input.pointer_capture_owner(pointer(7)), None);
    assert_eq!(surface.input.high_precision_owner, None);
    assert_eq!(surface.input.pointer_lock_owner, None);
    assert_eq!(surface.input.pointer_lock_policy, None);
    assert!(surface.tree.node(node(2)).unwrap().dirty.render);
    assert!(has_host_request(&result, |request| matches!(
        request,
        UiDispatchHostRequestKind::HighPrecisionPointer { target, enabled }
            if *target == node(2) && *enabled
    )));
    assert!(has_host_request(&result, |request| matches!(
        request,
        UiDispatchHostRequestKind::HighPrecisionPointer { target, enabled }
            if *target == node(2) && !*enabled
    )));
    assert!(has_host_request(&result, |request| matches!(
        request,
        UiDispatchHostRequestKind::PointerLock { target, policy }
            if *target == node(2) && *policy == UiPointerLockPolicy::RawDelta
    )));
    assert!(has_host_request(&result, |request| matches!(
        request,
        UiDispatchHostRequestKind::PointerUnlock { policy }
            if *policy == UiPointerLockPolicy::RawDelta
    )));
}

#[test]
fn dispatch_effect_matrix_applies_route_service_and_component_variants() {
    let mut surface = effect_matrix_surface("runtime.ui.effect_matrix.services");
    surface.focus_node(node(2)).unwrap();

    let component_event = UiComponentEvent::Commit {
        property: "effect".to_string(),
        value: UiValue::Bool(true),
    };
    let result = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effects([
            UiDispatchEffect::DragDrop {
                kind: UiDragDropEffectKind::Begin,
                target: node(2),
                pointer_id: pointer(8),
                session_id: Some(UiDragSessionId::new(80)),
                point: Some(UiPoint::new(14.0, 18.0)),
                payload: None,
            },
            UiDispatchEffect::RequestNavigation {
                kind: UiNavigationEventKind::Next,
                policy: UiNavigationRequestPolicy::Wrap,
            },
            UiDispatchEffect::Popup {
                kind: UiPopupEffectKind::Open,
                popup_id: "matrix.popup".to_string(),
                owner: Some(node(2)),
                anchor: Some(UiPoint::new(16.0, 20.0)),
            },
            UiDispatchEffect::Tooltip {
                kind: UiTooltipEffectKind::Arm,
                tooltip_id: "matrix.tooltip".to_string(),
                owner: Some(node(2)),
            },
            UiDispatchEffect::DismissTransientUi {
                target: UiTransientDismissalTarget::All,
                reason: UiTransientDismissalReason::Programmatic,
            },
            UiDispatchEffect::RequestInputMethod {
                request: UiInputMethodRequest {
                    kind: UiInputMethodRequestKind::Enable,
                    owner: node(2),
                    cursor_rect: Some(UiFrame::new(10.0, 10.0, 12.0, 20.0)),
                    composition_rects: vec![UiFrame::new(10.0, 10.0, 12.0, 20.0)],
                    surrounding_text: Some(UiInputMethodSurroundingText::new("abc", 1, 1).unwrap()),
                },
            },
            UiDispatchEffect::RequestClipboard {
                request: UiClipboardRequest {
                    kind: UiClipboardRequestKind::WriteText,
                    owner: node(2),
                    text: Some("copied".to_string()),
                },
            },
            UiDispatchEffect::EmitComponentEvent {
                target: node(2),
                event: component_event.clone(),
                policy: UiComponentEmissionPolicy::Immediate,
            },
        ]),
    );

    assert!(result.rejected_effects.is_empty());
    assert_eq!(applied_indices(&result), (0..8).collect::<Vec<_>>());
    let drag = surface.input.drag_drop.as_ref().unwrap();
    assert_eq!(drag.source, node(2));
    assert_eq!(drag.target, node(2));
    assert_eq!(drag.pointer_id, pointer(8));
    assert_eq!(surface.focus.focused, Some(node(3)));
    assert!(surface.input.popup_stack.is_empty());
    assert!(surface.input.tooltip.is_none());
    assert_eq!(surface.input.input_method_owner, Some(node(2)));
    assert_eq!(
        surface.input.input_method_request.as_ref().unwrap().kind,
        UiInputMethodRequestKind::Enable
    );
    assert!(has_host_request(&result, |request| matches!(
        request,
        UiDispatchHostRequestKind::Popup { kind, popup_id, .. }
            if *kind == UiPopupEffectKind::Open && popup_id == "matrix.popup"
    )));
    assert!(has_host_request(&result, |request| matches!(
        request,
        UiDispatchHostRequestKind::Tooltip { kind, tooltip_id }
            if *kind == UiTooltipEffectKind::Arm && tooltip_id == "matrix.tooltip"
    )));
    assert!(has_host_request(&result, |request| matches!(
        request,
        UiDispatchHostRequestKind::DismissTransientUi { target, reason }
            if *target == UiTransientDismissalTarget::All
                && *reason == UiTransientDismissalReason::Programmatic
    )));
    assert!(has_host_request(&result, |request| matches!(
        request,
        UiDispatchHostRequestKind::InputMethod(request)
            if request.kind == UiInputMethodRequestKind::Enable && request.owner == node(2)
    )));
    assert!(has_host_request(&result, |request| matches!(
        request,
        UiDispatchHostRequestKind::Clipboard(request)
            if request.kind == UiClipboardRequestKind::WriteText
                && request.owner == node(2)
                && request.text.as_deref() == Some("copied")
    )));
    assert!(result.component_events.iter().any(|event| {
        event.target == node(2) && event.delivered && event.event == component_event
    }));
}

#[test]
fn dispatch_effect_matrix_rejected_effects_keep_indices_and_reasons() {
    let mut surface = effect_matrix_surface("runtime.ui.effect_matrix.rejected");
    surface.navigation.navigation_root = Some(node(99));
    let invalid_effects = vec![
        (
            UiDispatchEffect::SetFocus {
                target: node(99),
                reason: UiFocusEffectReason::Programmatic,
            },
            "focus rejected",
        ),
        (
            UiDispatchEffect::ClearFocus {
                target: node(3),
                reason: UiFocusEffectReason::Dismissal,
            },
            "focus owner mismatch",
        ),
        (
            UiDispatchEffect::CapturePointer {
                target: node(99),
                pointer_id: pointer(7),
                reason: UiPointerCaptureReason::Press,
            },
            "invalid input owner",
        ),
        (
            UiDispatchEffect::ReleasePointerCapture {
                target: node(2),
                pointer_id: pointer(7),
                reason: UiPointerCaptureReason::Cancel,
            },
            "pointer capture belongs to a different or unknown pointer",
        ),
        (
            UiDispatchEffect::LockPointer {
                target: node(99),
                policy: UiPointerLockPolicy::Confined,
            },
            "invalid input owner",
        ),
        (
            UiDispatchEffect::UnlockPointer {
                target: node(2),
                policy: UiPointerLockPolicy::Confined,
            },
            "pointer lock owner mismatch",
        ),
        (
            UiDispatchEffect::UseHighPrecisionPointer {
                target: node(2),
                enabled: true,
            },
            "high precision requires pointer capture",
        ),
        (
            UiDispatchEffect::DragDrop {
                kind: UiDragDropEffectKind::Update,
                target: node(2),
                pointer_id: pointer(7),
                session_id: Some(UiDragSessionId::new(7)),
                point: Some(UiPoint::new(20.0, 20.0)),
                payload: None,
            },
            "drag session is not active",
        ),
        (
            UiDispatchEffect::RequestNavigation {
                kind: UiNavigationEventKind::Next,
                policy: UiNavigationRequestPolicy::Direct,
            },
            "navigation route rejected",
        ),
        (
            UiDispatchEffect::Popup {
                kind: UiPopupEffectKind::Open,
                popup_id: "missing-owner-popup".to_string(),
                owner: Some(node(99)),
                anchor: None,
            },
            "invalid input owner",
        ),
        (
            UiDispatchEffect::Tooltip {
                kind: UiTooltipEffectKind::Arm,
                tooltip_id: "missing-owner-tooltip".to_string(),
                owner: Some(node(99)),
            },
            "invalid input owner",
        ),
        (
            UiDispatchEffect::RequestInputMethod {
                request: UiInputMethodRequest {
                    kind: UiInputMethodRequestKind::Reset,
                    owner: node(2),
                    cursor_rect: None,
                    composition_rects: Vec::new(),
                    surrounding_text: None,
                },
            },
            "input method owner mismatch",
        ),
        (
            UiDispatchEffect::RequestClipboard {
                request: UiClipboardRequest {
                    kind: UiClipboardRequestKind::ReadText,
                    owner: node(2),
                    text: Some("invalid".to_string()),
                },
            },
            "clipboard read request cannot carry text",
        ),
        (
            UiDispatchEffect::DirtyRedraw {
                target: node(99),
                dirty: render_dirty(),
                reason: UiRedrawRequestReason::Input,
            },
            "missing dirty target",
        ),
        (
            UiDispatchEffect::EmitComponentEvent {
                target: node(99),
                event: UiComponentEvent::Commit {
                    property: "missing".to_string(),
                    value: UiValue::Bool(true),
                },
                policy: UiComponentEmissionPolicy::Immediate,
            },
            "missing node",
        ),
    ];
    let result = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled()
            .with_effects(invalid_effects.iter().map(|(effect, _)| effect.clone())),
    );

    assert!(result.applied_effects.is_empty());
    assert!(result.host_requests.is_empty());
    assert!(result.component_events.is_empty());
    assert_eq!(result.rejected_effects.len(), invalid_effects.len());
    for (index, (expected_effect, expected_reason)) in invalid_effects.iter().enumerate() {
        let rejected = &result.rejected_effects[index];
        assert_eq!(rejected.effect_index, index);
        assert_eq!(
            effect_variant_name(&rejected.effect),
            effect_variant_name(expected_effect)
        );
        assert_eq!(&rejected.effect, expected_effect);
        assert!(
            !rejected.reason.trim().is_empty(),
            "effect {index} should keep a rejection reason"
        );
        assert!(
            rejected.reason.contains(expected_reason),
            "effect {index} reason `{}` did not contain `{expected_reason}`",
            rejected.reason
        );
    }
}

fn effect_matrix_surface(tree_id: &str) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new(tree_id));
    surface.tree.insert_root(
        UiTreeNode::new(node(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 180.0, 120.0))
            .with_state_flags(input_state()),
    );
    surface
        .tree
        .insert_child(
            node(1),
            UiTreeNode::new(node(2), UiNodePath::new("root/first"))
                .with_frame(UiFrame::new(10.0, 10.0, 70.0, 40.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state()),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            node(1),
            UiTreeNode::new(node(3), UiNodePath::new("root/second"))
                .with_frame(UiFrame::new(100.0, 10.0, 70.0, 40.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state()),
        )
        .unwrap();
    surface.rebuild();
    surface
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

fn keyboard_event() -> UiInputEvent {
    UiInputEvent::Keyboard(UiKeyboardInputEvent {
        metadata: input_metadata(),
        state: UiKeyboardInputState::Pressed,
        key_code: 65,
        scan_code: Some(30),
        physical_key: "KeyA".to_string(),
        logical_key: "KeyA".to_string(),
        text: Some("a".to_string()),
    })
}

fn input_metadata() -> UiInputEventMetadata {
    let mut metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(10), UiInputSequence::new(1));
    metadata.pointer_id = Some(pointer(7));
    metadata
}

fn render_dirty() -> UiDirtyFlags {
    UiDirtyFlags {
        render: true,
        ..UiDirtyFlags::default()
    }
}

fn node(id: u64) -> UiNodeId {
    UiNodeId::new(id)
}

fn pointer(id: u64) -> UiPointerId {
    UiPointerId::new(id)
}

fn applied_indices(result: &UiInputDispatchResult) -> Vec<usize> {
    result
        .applied_effects
        .iter()
        .map(|effect| effect.effect_index)
        .collect()
}

fn has_host_request(
    result: &UiInputDispatchResult,
    mut predicate: impl FnMut(&UiDispatchHostRequestKind) -> bool,
) -> bool {
    result
        .host_requests
        .iter()
        .any(|host_request| predicate(&host_request.request))
}

fn effect_variant_name(effect: &UiDispatchEffect) -> &'static str {
    match effect {
        UiDispatchEffect::SetFocus { .. } => "SetFocus",
        UiDispatchEffect::ClearFocus { .. } => "ClearFocus",
        UiDispatchEffect::CapturePointer { .. } => "CapturePointer",
        UiDispatchEffect::ReleasePointerCapture { .. } => "ReleasePointerCapture",
        UiDispatchEffect::LockPointer { .. } => "LockPointer",
        UiDispatchEffect::UnlockPointer { .. } => "UnlockPointer",
        UiDispatchEffect::UseHighPrecisionPointer { .. } => "UseHighPrecisionPointer",
        UiDispatchEffect::DragDrop { .. } => "DragDrop",
        UiDispatchEffect::RequestNavigation { .. } => "RequestNavigation",
        UiDispatchEffect::Popup { .. } => "Popup",
        UiDispatchEffect::Tooltip { .. } => "Tooltip",
        UiDispatchEffect::DismissTransientUi { .. } => "DismissTransientUi",
        UiDispatchEffect::RequestInputMethod { .. } => "RequestInputMethod",
        UiDispatchEffect::RequestClipboard { .. } => "RequestClipboard",
        UiDispatchEffect::RequestLinkActivation { .. } => "RequestLinkActivation",
        UiDispatchEffect::DirtyRedraw { .. } => "DirtyRedraw",
        UiDispatchEffect::EmitComponentEvent { .. } => "EmitComponentEvent",
    }
}
