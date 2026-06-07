use zircon_runtime_interface::ui::dispatch::{
    UiDispatchEffect, UiDispatchReply, UiInputDispatchResult, UiInputEvent, UiTooltipEffectKind,
    UiTooltipTimerInputEvent, UiTooltipTimerInputEventKind,
};

use super::super::surface::UiSurface;
use super::{
    apply_dispatch_reply, route_policy::annotate_route_policy,
    route_steps::annotate_result_route_steps,
};

pub(super) fn dispatch_tooltip_timer_input(
    surface: &mut UiSurface,
    tooltip: UiTooltipTimerInputEvent,
) -> UiInputDispatchResult {
    let effect_kind = match tooltip.kind {
        UiTooltipTimerInputEventKind::Armed => UiTooltipEffectKind::Arm,
        UiTooltipTimerInputEventKind::Elapsed => UiTooltipEffectKind::Show,
        UiTooltipTimerInputEventKind::Canceled => UiTooltipEffectKind::Cancel,
    };
    let event = UiInputEvent::TooltipTimer(tooltip.clone());
    if !tooltip_timer_matches_retained_state(surface, &tooltip) {
        let mut result = UiInputDispatchResult::new(event, UiDispatchReply::unhandled());
        result
            .diagnostics
            .notes
            .push("stale_tooltip_timer_ignored".to_string());
        result.diagnostics.handled_phase = Some("tooltip.stale".to_string());
        return with_tooltip_route_policy(surface, result);
    }
    let reply = UiDispatchReply::handled().with_effect(UiDispatchEffect::Tooltip {
        kind: effect_kind,
        tooltip_id: tooltip.tooltip_id,
        owner: tooltip.owner,
    });
    let mut result = apply_dispatch_reply(surface, event, reply);
    result.diagnostics.routed = result.rejected_effects.is_empty();
    result.diagnostics.handled_phase = Some("tooltip.effect".to_string());
    with_tooltip_route_policy(surface, result)
}

fn with_tooltip_route_policy(
    surface: &UiSurface,
    mut result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let event = result.event.clone();
    annotate_route_policy(surface, &event, &mut result);
    annotate_result_route_steps(&mut result);
    result
}

fn tooltip_timer_matches_retained_state(
    surface: &UiSurface,
    tooltip: &UiTooltipTimerInputEvent,
) -> bool {
    match tooltip.kind {
        UiTooltipTimerInputEventKind::Armed => true,
        UiTooltipTimerInputEventKind::Elapsed | UiTooltipTimerInputEventKind::Canceled => surface
            .input
            .tooltip_matches(tooltip.tooltip_id.as_str(), tooltip.owner),
    }
}
