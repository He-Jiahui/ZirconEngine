use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchEffect, UiDispatchReply, UiDragDropEffectKind, UiDragDropInputEvent,
        UiDragDropInputEventKind, UiInputDispatchResult, UiInputEvent, UiInputRoutePolicy,
        UiPointerId,
    },
    event_ui::UiNodeId,
    layout::UiPoint,
};

use super::super::surface::UiSurface;
use super::{
    apply_dispatch_reply, route_policy::annotate_route_policy,
    route_steps::annotate_result_route_steps,
};

pub(super) fn dispatch_drag_drop_input(
    surface: &mut UiSurface,
    drag_drop: UiDragDropInputEvent,
) -> UiInputDispatchResult {
    let capture_target_before_dispatch = surface
        .input
        .drag_drop
        .as_ref()
        .map(|drag| drag.source)
        .or(surface.focus.captured);
    let event = UiInputEvent::DragDrop(drag_drop.clone());
    let pointer_id = drag_drop.metadata.pointer_id.unwrap_or_default();
    if !drag_drop_matches_retained_state(surface, &drag_drop, pointer_id) {
        let mut result = UiInputDispatchResult::new(event, UiDispatchReply::unhandled());
        result
            .diagnostics
            .notes
            .push("stale_drag_drop_event_ignored".to_string());
        result.diagnostics.handled_phase = Some("drag_drop.stale".to_string());
        return with_drag_drop_route_policy(surface, result, capture_target_before_dispatch);
    }

    let Some(target) = drag_drop_target(surface, drag_drop.point) else {
        let mut result = UiInputDispatchResult::new(event, UiDispatchReply::unhandled());
        result
            .diagnostics
            .notes
            .push("drag_drop target missing".to_string());
        return with_drag_drop_route_policy(surface, result, capture_target_before_dispatch);
    };

    let effect_kind = match drag_drop.kind {
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
        session_id: drag_drop.session_id,
        point: Some(drag_drop.point),
        payload: drag_drop.payload.clone(),
    });
    let mut result = apply_dispatch_reply(surface, event, reply);
    result.diagnostics.routed = result.rejected_effects.is_empty();
    result.diagnostics.route_target = Some(target);
    result.diagnostics.handled_phase = Some("drag_drop.effect".to_string());
    with_drag_drop_route_policy(surface, result, capture_target_before_dispatch)
}

fn with_drag_drop_route_policy(
    surface: &UiSurface,
    mut result: UiInputDispatchResult,
    capture_target_before_dispatch: Option<UiNodeId>,
) -> UiInputDispatchResult {
    let event = result.event.clone();
    annotate_route_policy(surface, &event, &mut result);
    if let UiInputEvent::DragDrop(drag_drop) = event {
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
    annotate_result_route_steps(&mut result);
    result
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
