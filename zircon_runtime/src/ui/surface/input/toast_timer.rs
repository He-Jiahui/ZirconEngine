use zircon_runtime_interface::ui::dispatch::{
    UiDispatchPhase, UiDispatchReply, UiInputDispatchResult, UiInputEvent, UiToastTimerInputEvent,
};

use super::super::surface::UiSurface;
use super::{route_policy::annotate_route_policy, route_steps::annotate_result_route_steps};

pub(super) fn dispatch_toast_timer_input(
    surface: &mut UiSurface,
    toast: UiToastTimerInputEvent,
) -> UiInputDispatchResult {
    let event = UiInputEvent::ToastTimer(toast.clone());
    let component_events = surface
        .apply_default_toast_timeout_component_event(toast.target, toast.toast_id.as_str())
        .unwrap_or_default();
    if component_events.is_empty() {
        let mut result = UiInputDispatchResult::new(event, UiDispatchReply::unhandled());
        result
            .diagnostics
            .notes
            .push("stale_toast_timer_ignored".to_string());
        result.diagnostics.handled_phase = Some("toast_timer.stale".to_string());
        return with_toast_route_policy(surface, result);
    }

    let reply = UiDispatchReply::handled()
        .from_handler(toast.target)
        .in_phase(UiDispatchPhase::DefaultAction);
    let mut result = UiInputDispatchResult::new(event, reply);
    result.diagnostics.routed = true;
    result.diagnostics.route_target = Some(toast.target);
    result.diagnostics.handled_phase = Some("toast_timer.component_event".to_string());
    result.component_events = component_events;
    with_toast_route_policy(surface, result)
}

fn with_toast_route_policy(
    surface: &UiSurface,
    mut result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let event = result.event.clone();
    annotate_route_policy(surface, &event, &mut result);
    annotate_result_route_steps(&mut result);
    result
}
