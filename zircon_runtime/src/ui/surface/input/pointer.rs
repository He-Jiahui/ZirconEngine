use zircon_runtime_interface::ui::{
    dispatch::{
        UiComponentEventReport, UiDispatchAppliedEffect, UiInputDispatchResult, UiInputEvent,
        UiInputEventMetadata, UiPointerEvent, UiPointerInputEvent,
    },
    surface::UiPointerEventKind,
    tree::UiTreeError,
};

use super::super::surface::UiSurface;
use super::{
    pointer_reply::{merge_pointer_text_result, pointer_component_handler, pointer_reply},
    route_policy::annotate_pointer_route_trace,
    route_steps::annotate_result_route_steps,
    text_pointer::dispatch_pointer_text_edit,
};
use crate::ui::dispatch::UiPointerDispatcher;

pub(super) fn dispatch_pointer_input(
    surface: &mut UiSurface,
    pointer_dispatcher: &UiPointerDispatcher,
    pointer: UiPointerInputEvent,
) -> Result<UiInputDispatchResult, UiTreeError> {
    let metadata = pointer.metadata.clone();
    record_cursor_position_before_pointer_dispatch(surface, &metadata, &pointer.event);
    let clear_cursor_point_after_dispatch =
        should_clear_cursor_position_after_pointer_dispatch(&metadata, &pointer.event);
    let pointer_for_text = pointer.clone();
    let legacy = dispatch_pointer_event_for_metadata(
        surface,
        pointer_dispatcher,
        &metadata,
        pointer.event.clone(),
    )?;
    surface.apply_pointer_dispatch_dirty(&legacy)?;
    let event = UiInputEvent::Pointer(pointer);
    if let Some(captured_by) = legacy.captured_by {
        if let Some(pointer_id) = metadata.pointer_id {
            surface
                .input
                .set_pointer_capture_for_id(pointer_id, captured_by);
        } else {
            surface.input.captured_pointer_id = None;
        }
    }
    if legacy.diagnostics.capture_released {
        if let Some(owner) = legacy.released_capture.or(legacy.route.captured) {
            if let Some(pointer_id) = metadata.pointer_id {
                surface
                    .input
                    .clear_pointer_capture_id_for_owner(pointer_id, owner);
            } else {
                surface.input.clear_pointer_capture_for(owner);
            }
        } else {
            surface.input.clear_pointer_capture();
        }
        surface.focus.captured = surface.input.activate_any_pointer_capture();
    }
    let component_handler = pointer_component_handler(&legacy);
    let reply = pointer_reply(&legacy, metadata.pointer_id.unwrap_or_default());
    let mut applied_effects = Vec::new();
    for (effect_index, effect) in reply.effects.iter().cloned().enumerate() {
        applied_effects.push(UiDispatchAppliedEffect {
            effect_index,
            effect,
        });
    }
    let mut result = UiInputDispatchResult::new(event, reply.clone());
    result.diagnostics.routed = legacy.diagnostics.pointer_routed;
    result.diagnostics.route_target = legacy.route.target;
    result.diagnostics.blocked_by = legacy.blocked_by;
    result.diagnostics.handled_phase = if legacy.handled_by.is_some() || component_handler.is_some()
    {
        Some("pointer".to_string())
    } else {
        None
    };
    if legacy.route.kind == UiPointerEventKind::Scroll {
        result
            .diagnostics
            .notes
            .push(format!("scroll_delta={}", legacy.route.scroll_delta));
    }
    result.applied_effects = applied_effects;
    result.drag = legacy
        .component_events
        .iter()
        .filter_map(|event| event.drag)
        .last();
    result.component_events = legacy
        .component_events
        .into_iter()
        .map(|event| UiComponentEventReport {
            target: event.node_id,
            event: event.envelope.event,
            delivered: true,
            drag: event.drag,
        })
        .collect();
    result.binding_reports = legacy.binding_reports;
    if let Some(text_result) = dispatch_pointer_text_edit(surface, &pointer_for_text, &legacy.route)
    {
        merge_pointer_text_result(&mut result, text_result);
    }
    if clear_cursor_point_after_dispatch {
        surface.input.clear_last_cursor_point();
    }
    let event = result.event.clone();
    annotate_pointer_route_trace(surface, &legacy.route, &event, &mut result);
    annotate_result_route_steps(&mut result);
    Ok(result)
}

fn record_cursor_position_before_pointer_dispatch(
    surface: &mut UiSurface,
    metadata: &UiInputEventMetadata,
    event: &UiPointerEvent,
) {
    if metadata.pointer_source.is_touch_like() {
        return;
    }

    if matches!(
        event.kind,
        UiPointerEventKind::Move
            | UiPointerEventKind::Down
            | UiPointerEventKind::Up
            | UiPointerEventKind::Scroll
    ) {
        surface
            .input
            .record_pointer_position(metadata.pointer_source, event.point);
    }
}

fn should_clear_cursor_position_after_pointer_dispatch(
    metadata: &UiInputEventMetadata,
    event: &UiPointerEvent,
) -> bool {
    !metadata.pointer_source.is_touch_like() && matches!(event.kind, UiPointerEventKind::Cancel)
}

fn dispatch_pointer_event_for_metadata(
    surface: &mut UiSurface,
    pointer_dispatcher: &UiPointerDispatcher,
    metadata: &zircon_runtime_interface::ui::dispatch::UiInputEventMetadata,
    event: zircon_runtime_interface::ui::dispatch::UiPointerEvent,
) -> Result<zircon_runtime_interface::ui::dispatch::UiPointerDispatchResult, UiTreeError> {
    if let Some(owner) = metadata
        .pointer_id
        .and_then(|pointer_id| surface.input.activate_pointer_capture_for_id(pointer_id))
    {
        surface.focus.captured = Some(owner);
    }
    let previous_pointer_captures = surface.input.pointer_captures.clone();
    let event_kind = event.kind;
    let mismatched_capture = surface
        .input
        .captured_pointer_id
        .zip(metadata.pointer_id)
        .filter(|(captured, incoming)| captured != incoming);
    let bypass_captor = matches!(
        event_kind,
        UiPointerEventKind::Move | UiPointerEventKind::Up | UiPointerEventKind::Cancel
    ) && mismatched_capture.is_some();
    let previous_capture = bypass_captor.then_some(surface.focus.captured).flatten();
    let previous_pressed = bypass_captor.then_some(surface.focus.pressed).flatten();
    let previous_pointer_id = bypass_captor
        .then_some(surface.input.captured_pointer_id)
        .flatten();
    if bypass_captor {
        surface.focus.captured = None;
        surface.focus.pressed = None;
    }
    let result = surface.dispatch_pointer_event(pointer_dispatcher, event);
    let captured_by_incoming_pointer = result
        .as_ref()
        .ok()
        .and_then(|result| result.captured_by)
        .is_some();
    if !captured_by_incoming_pointer {
        if let Some(previous_capture) =
            previous_capture.filter(|_| surface.focus.captured.is_none())
        {
            surface.focus.captured = Some(previous_capture);
        }
        if let Some(previous_pointer_id) =
            previous_pointer_id.filter(|_| surface.input.captured_pointer_id.is_none())
        {
            surface.input.captured_pointer_id = Some(previous_pointer_id);
        }
    }
    if let Some(previous_pressed) = previous_pressed.filter(|_| surface.focus.pressed.is_none()) {
        surface.focus.pressed = Some(previous_pressed);
    }
    if matches!(
        event_kind,
        UiPointerEventKind::Up | UiPointerEventKind::Cancel
    ) {
        if let Some(pointer_id) = metadata.pointer_id {
            for (captured_pointer_id, capture) in previous_pointer_captures {
                if captured_pointer_id != pointer_id {
                    surface
                        .input
                        .restore_pointer_capture(captured_pointer_id, capture);
                }
            }
        }
    }
    result
}
