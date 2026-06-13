use zircon_runtime_interface::ui::dispatch::{
    UiDispatchPhase, UiDispatchReply, UiInputDispatchResult, UiInputEvent,
    UiTypeaheadTimerInputEvent,
};

use super::super::surface::UiSurface;
use super::{route_policy::annotate_route_policy, route_steps::annotate_result_route_steps};

pub(super) fn dispatch_typeahead_timer_input(
    surface: &mut UiSurface,
    typeahead: UiTypeaheadTimerInputEvent,
) -> UiInputDispatchResult {
    let event = UiInputEvent::TypeaheadTimer(typeahead.clone());
    let supports_typeahead = surface
        .typeahead_timeout_ms_for_component_node(typeahead.target)
        .is_some();
    if !supports_typeahead {
        let mut result = UiInputDispatchResult::new(event, UiDispatchReply::unhandled());
        result
            .diagnostics
            .notes
            .push("stale_typeahead_timer_ignored".to_string());
        result.diagnostics.handled_phase = Some("typeahead_timer.stale".to_string());
        return with_typeahead_route_policy(surface, result);
    }

    let component_events = surface
        .apply_default_typeahead_expired_component_event(typeahead.target)
        .unwrap_or_default();
    let reply = if component_events.is_empty() {
        UiDispatchReply::unhandled()
    } else {
        UiDispatchReply::handled()
            .from_handler(typeahead.target)
            .in_phase(UiDispatchPhase::DefaultAction)
    };
    let mut result = UiInputDispatchResult::new(event, reply);
    result.diagnostics.routed = true;
    result.diagnostics.route_target = Some(typeahead.target);
    result.diagnostics.handled_phase = Some(if component_events.is_empty() {
        "typeahead_timer.no_binding".to_string()
    } else {
        "typeahead_timer.component_event".to_string()
    });
    result.component_events = component_events;
    with_typeahead_route_policy(surface, result)
}

fn with_typeahead_route_policy(
    surface: &UiSurface,
    mut result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let event = result.event.clone();
    annotate_route_policy(surface, &event, &mut result);
    annotate_result_route_steps(&mut result);
    result
}
