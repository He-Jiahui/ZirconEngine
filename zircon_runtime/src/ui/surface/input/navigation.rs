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
    let legacy = surface.dispatch_navigation_event(dispatcher, navigation.kind)?;
    let event = UiInputEvent::Navigation(navigation);
    let mut reply = if legacy.handled_by.is_some() || legacy.focus_changed_to.is_some() {
        UiDispatchReply::handled()
    } else {
        UiDispatchReply::unhandled()
    };
    if let Some(target) = legacy.focus_changed_to {
        reply = reply.with_effect(UiDispatchEffect::SetFocus {
            target,
            reason: zircon_runtime_interface::ui::dispatch::UiFocusEffectReason::Navigation,
        });
    }
    if legacy.handled_by.is_some() || legacy.focus_changed_to.is_some() {
        reply = reply.in_phase(UiDispatchPhase::Target);
        if let Some(handler) = legacy
            .handled_by
            .or(legacy.route.target)
            .or(legacy.focus_changed_to)
        {
            reply = reply.from_handler(handler);
        }
    }
    let mut result = UiInputDispatchResult::new(event, reply.clone());
    result.diagnostics.routed = legacy.route.target.is_some() || legacy.route.fallback_to_root;
    result.diagnostics.route_target = legacy.route.target.or(legacy.focus_changed_to);
    result.diagnostics.handled_phase = legacy.handled_by.map(|_| "navigation".to_string());
    if let Some(focused) = legacy.focus_changed_to.or(legacy.route.target) {
        record_owner_focused_input(
            surface,
            UiFocusedInputKind::Navigation,
            focused,
            legacy.handled_by.or(legacy.focus_changed_to),
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
    result.binding_reports = legacy.binding_reports;
    let event = result.event.clone();
    annotate_navigation_route_trace(surface, &legacy.route, &event, &mut result);
    annotate_result_route_steps(&mut result);
    Ok(result)
}
