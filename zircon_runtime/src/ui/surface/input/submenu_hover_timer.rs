use zircon_runtime_interface::ui::dispatch::{
    UiDispatchPhase, UiDispatchReply, UiInputDispatchResult, UiInputEvent,
    UiSubmenuHoverTimerInputEvent,
};

use super::super::surface::UiSurface;
use super::{route_policy::annotate_route_policy, route_steps::annotate_result_route_steps};

pub(super) fn dispatch_submenu_hover_timer_input(
    surface: &mut UiSurface,
    submenu_hover: UiSubmenuHoverTimerInputEvent,
) -> UiInputDispatchResult {
    let supports_submenu_hover = !submenu_hover.option_id.is_empty()
        && surface
            .submenu_hover_delay_ms_for_component_node(submenu_hover.target)
            .is_some();
    if !supports_submenu_hover {
        let (_, event) = into_submenu_hover_timer_event(submenu_hover);
        let mut result = UiInputDispatchResult::new(event, UiDispatchReply::unhandled());
        result
            .diagnostics
            .notes
            .push("stale_submenu_hover_timer_ignored".to_string());
        result.diagnostics.handled_phase = Some("submenu_hover_timer.stale".to_string());
        return with_submenu_hover_route_policy(surface, result);
    }

    let component_events = surface
        .apply_default_submenu_hover_ready_component_event(submenu_hover.target)
        .unwrap_or_default();
    let (target, event) = into_submenu_hover_timer_event(submenu_hover);
    let reply = if component_events.is_empty() {
        UiDispatchReply::unhandled()
    } else {
        UiDispatchReply::handled()
            .from_handler(target)
            .in_phase(UiDispatchPhase::DefaultAction)
    };
    let mut result = UiInputDispatchResult::new(event, reply);
    result.diagnostics.routed = true;
    result.diagnostics.route_target = Some(target);
    result.diagnostics.handled_phase = Some(if component_events.is_empty() {
        "submenu_hover_timer.no_binding".to_string()
    } else {
        "submenu_hover_timer.component_event".to_string()
    });
    result.component_events = component_events;
    with_submenu_hover_route_policy(surface, result)
}

fn into_submenu_hover_timer_event(
    submenu_hover: UiSubmenuHoverTimerInputEvent,
) -> (
    zircon_runtime_interface::ui::event_ui::UiNodeId,
    UiInputEvent,
) {
    (
        submenu_hover.target,
        UiInputEvent::SubmenuHoverTimer(submenu_hover),
    )
}

fn with_submenu_hover_route_policy(
    surface: &UiSurface,
    mut result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let event = result.event.clone();
    annotate_route_policy(surface, &event, &mut result);
    annotate_result_route_steps(&mut result);
    result
}

#[cfg(test)]
#[path = "submenu_hover_timer/owned_event_tests.rs"]
mod owned_event_tests;
