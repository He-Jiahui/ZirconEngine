use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchAppliedEffect, UiDispatchEffect, UiDispatchPhase, UiDispatchReply,
        UiInputDispatchResult, UiInputEvent, UiNavigationInputEvent,
    },
    focus::UiFocusedInputKind,
    tree::UiTreeError,
};

use super::super::surface::UiSurface;
use super::{
    owner_route::record_owner_focused_input, route_policy::annotate_navigation_route_trace,
    route_steps::annotate_result_route_steps,
};
use crate::ui::dispatch::UiNavigationDispatcher;

pub(super) fn dispatch_navigation_input(
    surface: &mut UiSurface,
    dispatcher: &UiNavigationDispatcher,
    navigation: UiNavigationInputEvent,
) -> Result<UiInputDispatchResult, UiTreeError> {
    let routed_reply = surface.dispatch_navigation_event(dispatcher, navigation.kind)?;
    let event = UiInputEvent::Navigation(navigation);
    let mut reply = if routed_reply.handled_by.is_some() || routed_reply.focus_changed_to.is_some()
    {
        UiDispatchReply::handled()
    } else {
        UiDispatchReply::unhandled()
    };
    if let Some(target) = routed_reply.focus_changed_to {
        reply = reply.with_effect(UiDispatchEffect::SetFocus {
            target,
            reason: zircon_runtime_interface::ui::dispatch::UiFocusEffectReason::Navigation,
        });
    }
    if routed_reply.handled_by.is_some() || routed_reply.focus_changed_to.is_some() {
        reply = reply.in_phase(UiDispatchPhase::Target);
        if let Some(handler) = routed_reply
            .handled_by
            .or(routed_reply.route.target)
            .or(routed_reply.focus_changed_to)
        {
            reply = reply.from_handler(handler);
        }
    }
    let mut result = UiInputDispatchResult::new(event, reply.clone());
    result.diagnostics.routed =
        routed_reply.route.target.is_some() || routed_reply.route.fallback_to_root;
    result.diagnostics.route_target = routed_reply.route.target.or(routed_reply.focus_changed_to);
    result.diagnostics.handled_phase = routed_reply.handled_by.map(|_| "navigation".to_string());
    if let Some(focused) = routed_reply.focus_changed_to.or(routed_reply.route.target) {
        record_owner_focused_input(
            surface,
            UiFocusedInputKind::Navigation,
            focused,
            routed_reply.handled_by.or(routed_reply.focus_changed_to),
            result.reply.disposition
                != zircon_runtime_interface::ui::dispatch::UiDispatchDisposition::Unhandled,
        );
    }
    for (effect_index, effect) in reply.effects.into_iter().enumerate() {
        result.applied_effects.push(UiDispatchAppliedEffect {
            effect_index,
            effect,
        });
    }
    result.binding_reports = routed_reply.binding_reports;
    let event = result.event.clone();
    annotate_navigation_route_trace(surface, &routed_reply.route, &event, &mut result);
    annotate_result_route_steps(&mut result);
    Ok(result)
}
