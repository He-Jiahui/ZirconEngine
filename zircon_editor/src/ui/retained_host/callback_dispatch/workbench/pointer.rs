use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;
use crate::ui::retained_host::host_contract::WorkbenchTooltipPointerTarget;
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    dispatch::{
        UiInputEventMetadata, UiInputSequence, UiInputTimestamp, UiPointerEvent,
        UiPointerInputEvent,
    },
};

use super::super::BuiltinWorkbenchWindowTemplateSurfaceBridge;

pub(crate) fn dispatch_componentized_workbench_pointer_event(
    runtime: &EditorHostEventController,
    bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge,
    event: UiPointerEvent,
) -> Option<Result<UiHostEventEffects, String>> {
    zircon_runtime::profile_counter!("editor", "ui.workbench.pointer.transaction_count", 1);
    let tooltip_input = UiPointerInputEvent {
        metadata: UiInputEventMetadata::new(
            UiInputTimestamp::from_micros(0),
            UiInputSequence::new(0),
        ),
        event: event.clone(),
        precise_scroll: None,
    };
    let pressed_before_route = bridge.pointer_pressed_target();
    let focused_before_route = bridge.pointer_focused_target();
    let route = match bridge.route_pointer_event(event) {
        Ok(route) => route,
        Err(error) => return Some(Err(error.to_string())),
    };
    let tooltip_target = route
        .captured
        .is_none()
        .then_some(route.target)
        .flatten()
        .map(WorkbenchTooltipPointerTarget::SurfaceNode);
    let tooltip_feedback_candidate =
        match bridge.update_workbench_icon_tooltip_candidate(tooltip_input, tooltip_target) {
            Ok(candidate) => candidate,
            Err(error) => return Some(Err(error.to_string())),
        };
    let virtual_row_scroll_candidate =
        match bridge.refresh_component_property_rows_after_scroll(&route) {
            Ok(candidate) => candidate,
            Err(error) => return Some(Err(error.to_string())),
        };
    let hover_feedback_candidate = match bridge.update_pointer_hover_feedback(&route) {
        Ok(candidate) => candidate,
        Err(error) => return Some(Err(error.to_string())),
    };
    let press_feedback_candidate =
        match bridge.update_pointer_press_feedback(&route, pressed_before_route) {
            Ok(candidate) => candidate,
            Err(error) => return Some(Err(error.to_string())),
        };
    let range_feedback_candidate = match bridge.update_pointer_range_feedback(&route) {
        Ok(candidate) => candidate,
        Err(error) => return Some(Err(error.to_string())),
    };
    let text_input_feedback_candidate = match bridge.update_text_input_pointer_feedback(&route) {
        Ok(candidate) => candidate,
        Err(error) => return Some(Err(error.to_string())),
    };
    let focus_feedback_candidate =
        match bridge.update_pointer_focus_feedback(&route, focused_before_route) {
            Ok(candidate) => candidate,
            Err(error) => return Some(Err(error.to_string())),
        };
    let pointer_feedback_candidate = tooltip_feedback_candidate
        || hover_feedback_candidate
        || press_feedback_candidate
        || range_feedback_candidate
        || text_input_feedback_candidate
        || focus_feedback_candidate
        || virtual_row_scroll_candidate;
    let cleared_search_control = match bridge.clear_search_field_from_pointer_route(&route) {
        Ok(control_id) => control_id,
        Err(error) => return Some(Err(error.to_string())),
    };
    if let Some(control_id) = cleared_search_control {
        return match super::control::dispatch_componentized_workbench_control(
            runtime,
            bridge,
            &control_id,
            UiEventKind::Change,
        ) {
            Some(Ok(mut effects)) => {
                effects.request_paint_only();
                Some(Ok(effects))
            }
            Some(Err(error)) => {
                if let Err(refresh_error) = bridge.refresh_pointer_feedback(true) {
                    return Some(Err(format!(
                        "{error}; pointer feedback refresh also failed: {refresh_error}"
                    )));
                }
                Some(Err(error))
            }
            None => {
                if let Err(error) = bridge.refresh_pointer_feedback(true) {
                    return Some(Err(error.to_string()));
                }
                let mut effects = UiHostEventEffects::default();
                effects.request_paint_only();
                Some(Ok(effects))
            }
        };
    }
    if let Some((control_id, event_kind)) = bridge.activation_route_for_pointer_route(&route) {
        // Control-state dispatch publishes its mutation, so it can commit pending feedback too.
        let coalesces_pending_feedback = pointer_feedback_candidate
            && bridge.surface().pending_invalidation_changed_node_count() > 0;
        let dispatched = super::control::dispatch_componentized_workbench_control(
            runtime,
            bridge,
            &control_id,
            event_kind,
        );
        if let Some(result) = dispatched {
            if result.is_ok() && coalesces_pending_feedback {
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.workbench.pointer.activation_coalesced_refresh_count",
                    1
                );
            }
            if result.is_err() {
                if let Err(refresh_error) =
                    bridge.refresh_pointer_feedback(pointer_feedback_candidate)
                {
                    return Some(Err(format!(
                        "pointer activation failed and feedback refresh also failed: {refresh_error}"
                    )));
                }
            }
            return Some(result);
        }
    }

    let pointer_feedback_dirty = match bridge.refresh_pointer_feedback(pointer_feedback_candidate) {
        Ok(dirty) => dirty,
        Err(error) => return Some(Err(error.to_string())),
    };
    if pointer_feedback_dirty {
        let mut effects = UiHostEventEffects::default();
        effects.request_paint_only();
        return Some(Ok(effects));
    }
    None
}
