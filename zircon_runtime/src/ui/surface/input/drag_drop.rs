use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchEffect, UiDispatchReply, UiDragDropEffectKind, UiDragDropInputEvent,
        UiDragDropInputEventKind, UiInputDiagnosticsMode, UiInputDispatchResult, UiInputEvent,
        UiInputRoutePolicy, UiPointerId,
    },
    event_ui::UiNodeId,
    layout::UiPoint,
};

use super::super::surface::UiSurface;
use super::{
    effect::apply_dispatch_reply_core, route_policy::annotate_route_policy,
    route_steps::annotate_result_route_steps,
};

pub(super) fn dispatch_drag_drop_input(
    surface: &mut UiSurface,
    drag_drop: UiDragDropInputEvent,
    diagnostics_mode: UiInputDiagnosticsMode,
) -> UiInputDispatchResult {
    let capture_target_before_dispatch = surface
        .input
        .drag_drop
        .as_ref()
        .map(|drag| drag.source)
        .or(surface.focus.captured);
    let pointer_id = drag_drop.metadata.pointer_id.unwrap_or_default();
    if !drag_drop_matches_retained_state(surface, &drag_drop, pointer_id) {
        let event = owned_drag_drop_input_event(drag_drop);
        let mut result = UiInputDispatchResult::new(event, UiDispatchReply::unhandled());
        if diagnostics_mode.captures_full_trace() {
            result
                .diagnostics
                .notes
                .push("stale_drag_drop_event_ignored".to_string());
            result.diagnostics.handled_phase = Some("drag_drop.stale".to_string());
        }
        return with_drag_drop_route_policy(
            surface,
            result,
            capture_target_before_dispatch,
            diagnostics_mode,
        );
    }

    let kind = drag_drop.kind;
    let session_id = drag_drop.session_id;
    let point = drag_drop.point;
    let drag_payload = drag_drop.payload.clone();
    let target = drag_drop_target(surface, point);
    let event = UiInputEvent::DragDrop(drag_drop);
    let Some(target) = target else {
        let mut result = UiInputDispatchResult::new(event, UiDispatchReply::unhandled());
        if diagnostics_mode.captures_full_trace() {
            result
                .diagnostics
                .notes
                .push("drag_drop target missing".to_string());
        }
        return with_drag_drop_route_policy(
            surface,
            result,
            capture_target_before_dispatch,
            diagnostics_mode,
        );
    };

    let effect_kind = match kind {
        UiDragDropInputEventKind::Begin => UiDragDropEffectKind::Begin,
        UiDragDropInputEventKind::Enter | UiDragDropInputEventKind::Over => {
            UiDragDropEffectKind::Update
        }
        UiDragDropInputEventKind::Leave => UiDragDropEffectKind::Reject,
        UiDragDropInputEventKind::Drop => UiDragDropEffectKind::Accept,
        UiDragDropInputEventKind::End => UiDragDropEffectKind::Complete,
    };
    let reply = UiDispatchReply::handled().with_effect(UiDispatchEffect::DragDrop {
        kind: effect_kind,
        target,
        pointer_id,
        session_id,
        point: Some(point),
        payload: drag_payload,
    });
    let mut result = apply_dispatch_reply_core(surface, event, reply, diagnostics_mode);
    result.diagnostics.routed = result.rejected_effects.is_empty();
    result.diagnostics.route_target = Some(target);
    if diagnostics_mode.captures_full_trace() {
        result.diagnostics.handled_phase = Some("drag_drop.effect".to_string());
    }
    with_drag_drop_route_policy(
        surface,
        result,
        capture_target_before_dispatch,
        diagnostics_mode,
    )
}

fn owned_drag_drop_input_event(drag_drop: UiDragDropInputEvent) -> UiInputEvent {
    UiInputEvent::DragDrop(drag_drop)
}

fn with_drag_drop_route_policy(
    surface: &UiSurface,
    mut result: UiInputDispatchResult,
    capture_target_before_dispatch: Option<UiNodeId>,
    diagnostics_mode: UiInputDiagnosticsMode,
) -> UiInputDispatchResult {
    if !diagnostics_mode.captures_full_trace() {
        return result;
    }

    let event = take_owned_drag_drop_input_event(&mut result.event);
    annotate_route_policy(surface, &event, &mut result);
    if let UiInputEvent::DragDrop(drag_drop) = &event {
        if matches!(drag_drop.kind, UiDragDropInputEventKind::End) {
            if let Some(capture_target) = capture_target_before_dispatch {
                result.diagnostics.route_trace.capture_target = Some(capture_target);
                if result.diagnostics.route_policy == UiInputRoutePolicy::PointerCapture {
                    result.diagnostics.route_trace.direct_target = Some(capture_target);
                }
                result.diagnostics.route_steps.clear();
            }
        }
    }
    result.event = event;
    annotate_result_route_steps(&mut result);
    result
}

fn take_owned_drag_drop_input_event(event: &mut UiInputEvent) -> UiInputEvent {
    std::mem::replace(
        event,
        UiInputEvent::DragDrop(UiDragDropInputEvent {
            metadata: Default::default(),
            kind: UiDragDropInputEventKind::Begin,
            session_id: None,
            point: UiPoint::new(0.0, 0.0),
            payload: None,
        }),
    )
}

fn drag_drop_matches_retained_state(
    surface: &UiSurface,
    drag_drop: &UiDragDropInputEvent,
    pointer_id: UiPointerId,
) -> bool {
    match drag_drop.kind {
        UiDragDropInputEventKind::Begin => true,
        UiDragDropInputEventKind::Enter
        | UiDragDropInputEventKind::Over
        | UiDragDropInputEventKind::Leave
        | UiDragDropInputEventKind::Drop
        | UiDragDropInputEventKind::End => surface
            .input
            .drag_drop_matches(pointer_id, drag_drop.session_id),
    }
}

fn drag_drop_target(surface: &UiSurface, point: UiPoint) -> Option<UiNodeId> {
    surface
        .hit_test(point)
        .top_hit
        .or_else(|| surface.input.drag_drop.as_ref().map(|drag| drag.target))
        .or(surface.focus.captured)
        .or(surface.focus.focused)
}

#[cfg(test)]
#[path = "drag_drop/stale_owned_event_tests.rs"]
mod stale_owned_event_tests;
