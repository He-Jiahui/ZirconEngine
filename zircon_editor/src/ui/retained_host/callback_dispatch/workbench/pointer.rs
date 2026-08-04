use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;
use zircon_runtime_interface::ui::{binding::UiEventKind, dispatch::UiPointerEvent};

use super::super::BuiltinWorkbenchWindowTemplateSurfaceBridge;

pub(crate) fn dispatch_componentized_workbench_pointer_event(
    runtime: &EditorHostEventController,
    bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge,
    event: UiPointerEvent,
) -> Option<Result<UiHostEventEffects, String>> {
    let pressed_before_route = bridge.pointer_pressed_target();
    let focused_before_route = bridge.pointer_focused_target();
    let route = match bridge.route_pointer_event(event) {
        Ok(route) => route,
        Err(error) => return Some(Err(error.to_string())),
    };
    let hover_feedback_dirty = match bridge.refresh_pointer_hover_feedback(&route) {
        Ok(dirty) => dirty,
        Err(error) => return Some(Err(error.to_string())),
    };
    let press_feedback_dirty =
        match bridge.refresh_pointer_press_feedback(&route, pressed_before_route) {
            Ok(dirty) => dirty,
            Err(error) => return Some(Err(error.to_string())),
        };
    let range_feedback_dirty = match bridge.refresh_pointer_range_feedback(&route) {
        Ok(dirty) => dirty,
        Err(error) => return Some(Err(error.to_string())),
    };
    let text_input_feedback_dirty = match bridge.refresh_text_input_pointer_feedback(&route) {
        Ok(dirty) => dirty,
        Err(error) => return Some(Err(error.to_string())),
    };
    let focus_feedback_dirty =
        match bridge.refresh_pointer_focus_feedback(&route, focused_before_route) {
            Ok(dirty) => dirty,
            Err(error) => return Some(Err(error.to_string())),
        };
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
            Some(Err(error)) => Some(Err(error)),
            None => {
                let mut effects = UiHostEventEffects::default();
                effects.request_paint_only();
                Some(Ok(effects))
            }
        };
    }
    let Some((control_id, event_kind)) = bridge.activation_route_for_pointer_route(&route) else {
        if hover_feedback_dirty
            || press_feedback_dirty
            || range_feedback_dirty
            || text_input_feedback_dirty
            || focus_feedback_dirty
        {
            let mut effects = UiHostEventEffects::default();
            effects.request_paint_only();
            return Some(Ok(effects));
        }
        return None;
    };

    super::control::dispatch_componentized_workbench_control(
        runtime,
        bridge,
        &control_id,
        event_kind,
    )
}
