use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchAppliedEffect, UiDispatchEffect, UiDispatchPhase, UiDispatchReply,
        UiInputDispatchResult, UiInputEvent, UiPointerCaptureReason, UiPointerInputEvent,
    },
    event_ui::UiNodeId,
    focus::UiFocusedInputKind,
    surface::{UiPointerActivationPhase, UiPointerEventKind, UiPointerRoute, UiTextEditAction},
};

use crate::ui::text::{apply_text_edit_action, hit_test_text_layout};
use crate::ui::tree::UiRuntimeTreeRoutingExt;

use super::super::surface::UiSurface;
use super::{
    dispatch::{apply_editable_text_state, TextComponentEventKind},
    is_valid_input_owner,
    text_state::editable_text_state_for_node,
};

pub(super) fn dispatch_pointer_text_edit(
    surface: &mut UiSurface,
    pointer: &UiPointerInputEvent,
    route: &UiPointerRoute,
) -> Option<UiInputDispatchResult> {
    let target = text_pointer_target(surface, pointer, route)?;
    if !is_valid_input_owner(surface, target) {
        return None;
    }
    if matches!(route.kind, UiPointerEventKind::Up) {
        return dispatch_pointer_text_release(surface, pointer, route, target);
    }
    let source_offset = text_pointer_source_offset(surface, target, route)?;
    let editable = editable_text_state_for_node(surface, target)?;
    let extend_selection =
        pointer.metadata.modifiers.shift || matches!(route.kind, UiPointerEventKind::Move);
    let next = apply_text_edit_action(
        editable,
        UiTextEditAction::MoveCaret {
            offset: source_offset,
            extend_selection,
        },
    );

    if matches!(route.kind, UiPointerEventKind::Down) {
        surface.capture_pointer(target).ok()?;
        surface.input.captured_pointer_id = pointer.metadata.pointer_id;
    }
    let drag = match route.kind {
        UiPointerEventKind::Down => surface.input.begin_pointer_drag(target, route.point),
        UiPointerEventKind::Move => surface.input.update_pointer_drag(target, route.point),
        _ => return None,
    };

    let mut result = apply_editable_text_state(
        surface,
        UiInputEvent::Pointer(pointer.clone()),
        target,
        next,
        pointer_text_phase(route.kind),
        TextComponentEventKind::Change,
    );
    result
        .diagnostics
        .notes
        .push(format!("text_pointer_offset={source_offset}"));
    result.drag = Some(drag);
    result.diagnostics.notes.push(format!(
        "text_pointer_drag={:?}:{:.3}",
        drag.phase, drag.distance
    ));
    if matches!(route.kind, UiPointerEventKind::Down) {
        push_text_pointer_capture_effect(&mut result, target, pointer);
    }
    Some(result)
}

fn dispatch_pointer_text_release(
    surface: &mut UiSurface,
    pointer: &UiPointerInputEvent,
    route: &UiPointerRoute,
    target: UiNodeId,
) -> Option<UiInputDispatchResult> {
    if !surface.input.pointer_drags.contains_key(&target) {
        return None;
    }
    let drag = surface.input.end_pointer_drag(target, route.point);
    let mut result = UiInputDispatchResult::new(
        UiInputEvent::Pointer(pointer.clone()),
        UiDispatchReply::handled()
            .from_handler(target)
            .in_phase(UiDispatchPhase::DefaultAction),
    );
    result.diagnostics.routed = true;
    result.diagnostics.route_target = Some(target);
    result.diagnostics.handled_phase = Some("pointer.text_release".to_string());
    result.drag = Some(drag);
    result.diagnostics.notes.push(format!(
        "text_pointer_drag={:?}:{:.3}",
        drag.phase, drag.distance
    ));
    let focused_route = surface.tree.bubble_route(target).unwrap_or_default();
    surface.record_focused_input(
        UiFocusedInputKind::Pointer,
        target,
        focused_route,
        Some(target),
        true,
    );
    Some(result)
}

fn text_pointer_target(
    surface: &UiSurface,
    pointer: &UiPointerInputEvent,
    route: &UiPointerRoute,
) -> Option<UiNodeId> {
    match route.kind {
        UiPointerEventKind::Down => (route.activation_phase
            == UiPointerActivationPhase::PrimaryPress)
            .then_some(route.target)
            .flatten(),
        UiPointerEventKind::Move => route
            .captured
            .filter(|target| text_pointer_capture_matches(surface, pointer, *target)),
        UiPointerEventKind::Up => route.captured,
        _ => None,
    }
}

fn text_pointer_capture_matches(
    surface: &UiSurface,
    pointer: &UiPointerInputEvent,
    target: UiNodeId,
) -> bool {
    if surface.focus.captured != Some(target) {
        return false;
    }
    match surface.input.captured_pointer_id {
        Some(pointer_id) => Some(pointer_id) == pointer.metadata.pointer_id,
        None => true,
    }
}

fn text_pointer_source_offset(
    surface: &UiSurface,
    target: UiNodeId,
    route: &UiPointerRoute,
) -> Option<usize> {
    let layout = surface
        .render_extract
        .list
        .commands
        .iter()
        .find_map(|command| {
            (command.node_id == target)
                .then(|| command.text_layout.as_ref())
                .flatten()
        })?;
    Some(hit_test_text_layout(layout, route.point).source_offset)
}

fn push_text_pointer_capture_effect(
    result: &mut UiInputDispatchResult,
    target: UiNodeId,
    pointer: &UiPointerInputEvent,
) {
    let effect = UiDispatchEffect::CapturePointer {
        target,
        pointer_id: pointer.metadata.pointer_id.unwrap_or_default(),
        reason: UiPointerCaptureReason::Press,
    };
    let effect_index = result.reply.effects.len();
    result.reply.handler = Some(target);
    result.reply.phase = Some(UiDispatchPhase::DefaultAction);
    result.reply.effects.push(effect.clone());
    result.applied_effects.push(UiDispatchAppliedEffect {
        effect_index,
        effect,
    });
}

fn pointer_text_phase(kind: UiPointerEventKind) -> &'static str {
    match kind {
        UiPointerEventKind::Move => "pointer.text_drag",
        UiPointerEventKind::Up => "pointer.text_release",
        _ => "pointer.text_press",
    }
}
