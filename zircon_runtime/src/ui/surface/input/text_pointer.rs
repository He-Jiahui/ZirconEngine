use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchAppliedEffect, UiDispatchEffect, UiDispatchHostRequest,
        UiDispatchHostRequestKind, UiDispatchPhase, UiDispatchReply, UiInputDispatchResult,
        UiInputEvent, UiPointerCaptureReason, UiPointerInputEvent, UiPopupEffectKind,
    },
    event_ui::UiNodeId,
    focus::UiFocusedInputKind,
    surface::{UiPointerActivationPhase, UiPointerEventKind, UiPointerRoute, UiTextEditAction},
};

use crate::ui::text::{apply_text_edit_action, hit_test_text_layout, word_range_at};
use crate::ui::tree::UiRuntimeTreeRoutingExt;

use super::super::surface::UiSurface;
use super::{
    editable_text::{apply_editable_text_state, TextComponentEventKind},
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
    let primary_press = route.activation_phase == UiPointerActivationPhase::PrimaryPress;
    let secondary_press = route.activation_phase == UiPointerActivationPhase::SecondaryPress;
    let is_double_click =
        primary_press && pointer.event.click_count >= 2 && !pointer.metadata.modifiers.shift;
    let mut phase = pointer_text_phase(route.kind);
    let mut selection_note = None;
    let next = if is_double_click {
        let (anchor, focus) =
            word_range_at(&editable.text, source_offset).unwrap_or((source_offset, source_offset));
        apply_text_edit_action(editable, UiTextEditAction::SetSelection { anchor, focus })
    } else if secondary_press {
        phase = "pointer.text_secondary_press";
        let reset_selection = secondary_press_should_reset_selection(&editable, source_offset);
        selection_note = Some(if reset_selection {
            "text_pointer_secondary_selection_reset"
        } else {
            "text_pointer_secondary_selection_preserved"
        });
        if reset_selection {
            apply_text_edit_action(
                editable,
                UiTextEditAction::MoveCaret {
                    offset: source_offset,
                    extend_selection: false,
                },
            )
        } else {
            editable
        }
    } else {
        let extend_selection =
            pointer.metadata.modifiers.shift || matches!(route.kind, UiPointerEventKind::Move);
        apply_text_edit_action(
            editable,
            UiTextEditAction::MoveCaret {
                offset: source_offset,
                extend_selection,
            },
        )
    };

    if matches!(route.kind, UiPointerEventKind::Down) {
        surface.capture_pointer(target).ok()?;
        surface.input.captured_pointer_id = pointer.metadata.pointer_id;
    }
    let drag = match route.kind {
        UiPointerEventKind::Down if primary_press => {
            Some(surface.input.begin_pointer_drag(target, route.point))
        }
        UiPointerEventKind::Down if secondary_press => None,
        UiPointerEventKind::Move => Some(surface.input.update_pointer_drag(target, route.point)),
        _ => return None,
    };

    let mut result = apply_editable_text_state(
        surface,
        UiInputEvent::Pointer(pointer.clone()),
        target,
        next,
        phase,
        TextComponentEventKind::Change,
    );
    result
        .diagnostics
        .notes
        .push(format!("text_pointer_offset={source_offset}"));
    if is_double_click {
        result
            .diagnostics
            .notes
            .push("text_pointer_word_selection".to_string());
    }
    if let Some(selection_note) = selection_note {
        result.diagnostics.notes.push(selection_note.to_string());
    }
    if let Some(drag) = drag {
        result.drag = Some(drag);
        result.diagnostics.notes.push(format!(
            "text_pointer_drag={:?}:{:.3}",
            drag.phase, drag.distance
        ));
    }
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
    if route.activation_phase == UiPointerActivationPhase::SecondaryRelease {
        return Some(dispatch_pointer_text_secondary_release(
            surface, pointer, route, target,
        ));
    }
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

fn dispatch_pointer_text_secondary_release(
    surface: &mut UiSurface,
    pointer: &UiPointerInputEvent,
    route: &UiPointerRoute,
    target: UiNodeId,
) -> UiInputDispatchResult {
    let mut result = UiInputDispatchResult::new(
        UiInputEvent::Pointer(pointer.clone()),
        UiDispatchReply::handled()
            .from_handler(target)
            .in_phase(UiDispatchPhase::DefaultAction),
    );
    result.diagnostics.routed = true;
    result.diagnostics.route_target = Some(target);
    result.diagnostics.handled_phase = Some("pointer.text_secondary_release".to_string());

    let focused_route = surface.tree.bubble_route(target).unwrap_or_default();
    surface.record_focused_input(
        UiFocusedInputKind::Pointer,
        target,
        focused_route,
        Some(target),
        true,
    );

    if route.stacked.contains(&target) {
        push_text_pointer_context_popup_effect(&mut result, surface, target, route);
        result
            .diagnostics
            .notes
            .push("text_pointer_secondary_context_menu".to_string());
    } else {
        result
            .diagnostics
            .notes
            .push("text_pointer_secondary_release_outside".to_string());
    }

    result
}

fn text_pointer_target(
    surface: &UiSurface,
    pointer: &UiPointerInputEvent,
    route: &UiPointerRoute,
) -> Option<UiNodeId> {
    match route.kind {
        UiPointerEventKind::Down => (route.activation_phase
            == UiPointerActivationPhase::PrimaryPress
            || route.activation_phase == UiPointerActivationPhase::SecondaryPress)
            .then_some(route.target)
            .flatten(),
        UiPointerEventKind::Move => route.captured.filter(|target| {
            text_pointer_capture_matches(surface, pointer, *target)
                && surface.input.pointer_drags.contains_key(target)
        }),
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

fn push_text_pointer_context_popup_effect(
    result: &mut UiInputDispatchResult,
    surface: &mut UiSurface,
    target: UiNodeId,
    route: &UiPointerRoute,
) {
    let popup_id = text_context_popup_id(target);
    let effect = UiDispatchEffect::Popup {
        kind: UiPopupEffectKind::Open,
        popup_id: popup_id.clone(),
        owner: Some(target),
        anchor: Some(route.point),
    };
    let effect_index = result.reply.effects.len();

    surface
        .input
        .open_popup(popup_id.clone(), Some(target), Some(route.point));
    result.reply.effects.push(effect.clone());
    result.applied_effects.push(UiDispatchAppliedEffect {
        effect_index,
        effect,
    });
    result.host_requests.push(UiDispatchHostRequest {
        effect_index,
        request: UiDispatchHostRequestKind::Popup {
            kind: UiPopupEffectKind::Open,
            popup_id,
            anchor: Some(route.point),
        },
        reason: format!("effect applied for {target:?}"),
    });
}

fn text_context_popup_id(target: UiNodeId) -> String {
    format!("text_input.context_menu.{}", target.0)
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

fn secondary_press_should_reset_selection(
    editable: &zircon_runtime_interface::ui::surface::UiEditableTextState,
    source_offset: usize,
) -> bool {
    editable.selection.as_ref().is_some_and(|selection| {
        let range = selection.range();
        range.start != range.end && !(range.start <= source_offset && source_offset <= range.end)
    })
}

fn pointer_text_phase(kind: UiPointerEventKind) -> &'static str {
    match kind {
        UiPointerEventKind::Move => "pointer.text_drag",
        UiPointerEventKind::Up => "pointer.text_release",
        _ => "pointer.text_press",
    }
}
