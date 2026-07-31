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
    rich_link::dispatch_pointer_rich_link_activation,
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
    let routed_result = dispatch_pointer_event_for_metadata(
        surface,
        pointer_dispatcher,
        &metadata,
        pointer.event.clone(),
    )?;
    surface.apply_pointer_dispatch_dirty(&routed_result)?;
    let event = UiInputEvent::Pointer(pointer);
    if let Some(captured_by) = routed_result.captured_by {
        if let Some(pointer_id) = metadata.pointer_id {
            surface
                .input
                .set_pointer_capture_for_id(pointer_id, captured_by);
        }
    }
    if routed_result.diagnostics.capture_released {
        if let Some(owner) = routed_result
            .released_capture
            .or(routed_result.route.captured)
        {
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
    let component_handler = pointer_component_handler(&routed_result);
    let reply = pointer_reply(&routed_result, metadata.pointer_id.unwrap_or_default());
    let mut applied_effects = Vec::new();
    for (effect_index, effect) in reply.effects.iter().cloned().enumerate() {
        applied_effects.push(UiDispatchAppliedEffect {
            effect_index,
            effect,
        });
    }
    let mut result = UiInputDispatchResult::new(event, reply.clone());
    result.diagnostics.routed = routed_result.diagnostics.pointer_routed;
    result.diagnostics.route_target = routed_result.route.target;
    result.diagnostics.blocked_by = routed_result.blocked_by;
    result.diagnostics.handled_phase =
        if routed_result.handled_by.is_some() || component_handler.is_some() {
            Some("pointer".to_string())
        } else {
            None
        };
    if routed_result.route.kind == UiPointerEventKind::Scroll {
        result
            .diagnostics
            .notes
            .push(format!("scroll_delta={}", routed_result.route.scroll_delta));
    }
    result.applied_effects = applied_effects;
    result.drag = routed_result
        .component_events
        .iter()
        .filter_map(|event| event.drag)
        .last();
    result.component_events = routed_result
        .component_events
        .into_iter()
        .map(|event| UiComponentEventReport {
            target: event.node_id,
            event: event.envelope.event,
            delivered: true,
            drag: event.drag,
            template_action: event.template_action,
        })
        .collect();
    result.binding_reports = routed_result.binding_reports;
    if let Some(text_result) =
        dispatch_pointer_text_edit(surface, &pointer_for_text, &routed_result.route)
    {
        merge_pointer_text_result(&mut result, text_result);
    }
    dispatch_pointer_rich_link_activation(
        surface,
        &pointer_for_text,
        &routed_result.route,
        &mut result,
    );
    if clear_cursor_point_after_dispatch {
        surface.input.clear_last_cursor_point();
    }
    let event = result.event.clone();
    annotate_pointer_route_trace(surface, &routed_result.route, &event, &mut result);
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
    let incoming_capture = metadata
        .pointer_id
        .and_then(|pointer_id| surface.input.activate_pointer_capture_for_id(pointer_id));
    if let Some(owner) = incoming_capture {
        surface.focus.captured = Some(owner);
    }
    let event_kind = event.kind;
    let previous_pointer_captures = matches!(
        event_kind,
        UiPointerEventKind::Up | UiPointerEventKind::Cancel
    )
    .then(|| surface.input.pointer_captures.clone());
    let bypass_captor = matches!(
        event_kind,
        UiPointerEventKind::Move | UiPointerEventKind::Up | UiPointerEventKind::Cancel
    ) && metadata.pointer_id.is_some()
        && incoming_capture.is_none()
        && surface.input.active_pointer_capture().is_some();
    let previous_capture = bypass_captor.then_some(surface.focus.captured).flatten();
    let previous_pressed = bypass_captor.then_some(surface.focus.pressed).flatten();
    if bypass_captor {
        surface.focus.captured = None;
        surface.focus.pressed = None;
    }
    let result = surface.dispatch_pointer_event_with_modifiers(
        pointer_dispatcher,
        event,
        metadata.modifiers,
    );
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
    }
    if let Some(previous_pressed) = previous_pressed.filter(|_| surface.focus.pressed.is_none()) {
        surface.focus.pressed = Some(previous_pressed);
    }
    if matches!(
        event_kind,
        UiPointerEventKind::Up | UiPointerEventKind::Cancel
    ) {
        if let (Some(pointer_id), Some(previous_pointer_captures)) =
            (metadata.pointer_id, previous_pointer_captures)
        {
            for (retained_pointer_id, capture) in previous_pointer_captures {
                if retained_pointer_id != pointer_id {
                    surface
                        .input
                        .restore_pointer_capture(retained_pointer_id, capture);
                }
            }
        }
    }
    result
}
