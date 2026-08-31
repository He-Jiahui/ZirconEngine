use std::sync::Arc;

use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::{UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface},
};
use zircon_runtime_interface::ui::dispatch::{
    UiAnalogInputEvent, UiDispatchDisposition, UiDispatchEffect, UiDispatchPhase, UiDispatchReply,
    UiDispatchReplyStep, UiDragDropEffectKind, UiDragDropInputEvent, UiDragDropInputEventKind,
    UiDragSessionId, UiFocusEffectReason, UiInputEvent, UiInputEventMetadata, UiInputMethodRequest,
    UiInputMethodRequestKind, UiInputSequence, UiInputTimestamp, UiKeyboardInputEvent,
    UiKeyboardInputState, UiNavigationInputEvent, UiNavigationRequestPolicy,
    UiPointerCaptureReason, UiPointerEvent, UiPointerId, UiPointerInputEvent, UiPointerLockPolicy,
    UiPopupInputEvent, UiPopupInputEventKind, UiTooltipTimerInputEvent,
    UiTooltipTimerInputEventKind,
};
use zircon_runtime_interface::ui::{
    component::{UiDragPayload, UiDragPayloadKind, UiValue},
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    focus::UiFocusChangeReason,
    layout::{UiFrame, UiPoint},
    surface::{UiNavigationEventKind, UiPointerButton, UiPointerEventKind},
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode, UiVisibility},
};

mod drag_drop;
mod high_precision_dispatch;
mod input_method;
mod owner_validation;
mod popup_tooltip;
mod route_trace;
mod transaction;

#[test]
fn input_hot_paths_avoid_eager_capture_trace_and_effect_index_allocations() {
    let pointer = include_str!("../surface/input/pointer.rs");
    let policy = include_str!("../surface/input/route_policy.rs");
    let reply = include_str!("../surface/input/pointer_reply.rs");
    let keyboard_navigation = include_str!("../surface/input/keyboard_navigation.rs");
    let keyboard_action = include_str!("../surface/input/keyboard_action.rs");
    let text_constraints = include_str!("../surface/input/text_constraints.rs");
    let ime_context = include_str!("../surface/input/editable_text/ime_context.rs");
    let pointer_capture = include_str!("../surface/input/state/pointer_capture.rs");

    assert!(
        pointer.contains("let previous_pointer_captures = matches!("),
        "pointer capture maps should only be copied for terminal pointer events"
    );
    assert!(
        !policy.contains("annotate_route_policy(surface, event, result);"),
        "specialized route traces should not first build a generic trace that is overwritten"
    );
    assert!(
        !reply.contains("effect_index_map") && !reply.contains("remap_text_effect_index"),
        "contiguous appended effects should rebase indexes with a constant offset"
    );
    assert!(
        !keyboard_navigation.contains("normalized_key_name"),
        "directional key matching should not allocate a normalized key String"
    );
    assert!(
        !keyboard_action.contains("normalized_key_name"),
        "semantic keyboard actions should not allocate a normalized key String"
    );
    assert!(
        !text_constraints.contains("normalize_constraint_token")
            && !text_constraints.contains("fn string_attribute"),
        "text input constraints should borrow and compare static metadata without normalization allocations"
    );
    assert!(
        !ime_context.contains("layout: layout.clone()")
            && !ime_context.contains("style: command.style.clone()"),
        "IME cursor lookup should borrow the current render layout and style"
    );
    assert!(
        !pointer_capture.contains("collect::<Vec<_>>()"),
        "clearing captures for an owner should retain in place without a temporary id list"
    );
}

fn capture_pointer_for_test(surface: &mut UiSurface, pointer_id: UiPointerId, owner: UiNodeId) {
    surface.focus.captured = Some(owner);
    surface.input.set_pointer_capture_for_id(pointer_id, owner);
}

fn assert_pointer_capture(surface: &UiSurface, pointer_id: UiPointerId, owner: UiNodeId) {
    assert_eq!(surface.input.pointer_capture_owner(pointer_id), Some(owner));
}

fn assert_no_pointer_capture(surface: &UiSurface) {
    assert_eq!(surface.input.active_pointer_capture(), None);
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
        payload: payload.map(Arc::new),
    })
}

fn pointer_event(kind: UiPointerEventKind, point: UiPoint) -> UiInputEvent {
    UiInputEvent::Pointer(UiPointerInputEvent {
        metadata: input_metadata(),
        event: UiPointerEvent::new(kind, point).with_button(UiPointerButton::Primary),
        precise_scroll: None,
    })
}

fn popup_input_event(
    kind: UiPopupInputEventKind,
    popup_id: &str,
    anchor: Option<UiPoint>,
) -> UiInputEvent {
    popup_input_event_for_owner(kind, popup_id, None, anchor)
}

fn popup_input_event_for_owner(
    kind: UiPopupInputEventKind,
    popup_id: &str,
    owner: Option<UiNodeId>,
    anchor: Option<UiPoint>,
) -> UiInputEvent {
    UiInputEvent::Popup(UiPopupInputEvent {
        metadata: input_metadata(),
        kind,
        popup_id: popup_id.to_string(),
        owner,
        anchor,
    })
}

fn tooltip_input_event(kind: UiTooltipTimerInputEventKind, tooltip_id: &str) -> UiInputEvent {
    tooltip_input_event_for_owner(kind, tooltip_id, None)
}

fn tooltip_input_event_for_owner(
    kind: UiTooltipTimerInputEventKind,
    tooltip_id: &str,
    owner: Option<UiNodeId>,
) -> UiInputEvent {
    UiInputEvent::TooltipTimer(UiTooltipTimerInputEvent {
        metadata: input_metadata(),
        kind,
        tooltip_id: tooltip_id.to_string(),
        owner,
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
        payload: payload.map(Arc::new),
    }
}

fn input_method_request(kind: UiInputMethodRequestKind, owner: UiNodeId) -> UiInputMethodRequest {
    UiInputMethodRequest {
        kind,
        owner,
        cursor_rect: None,
        composition_rects: Vec::new(),
        surrounding_text: None,
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
