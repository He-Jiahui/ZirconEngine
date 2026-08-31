use zircon_runtime_interface::ui::dispatch::{
    UiDispatchEffect, UiDispatchReply, UiInputDispatchResult, UiInputEvent, UiPopupEffectKind,
    UiPopupInputEvent, UiPopupInputEventKind,
};

use super::super::surface::UiSurface;
use super::{
    apply_dispatch_reply, route_policy::annotate_route_policy,
    route_steps::annotate_result_route_steps,
};

pub(super) fn dispatch_popup_input(
    surface: &mut UiSurface,
    popup: UiPopupInputEvent,
) -> UiInputDispatchResult {
    let effect_kind = match popup.kind {
        UiPopupInputEventKind::OpenRequested => UiPopupEffectKind::Open,
        UiPopupInputEventKind::CloseRequested | UiPopupInputEventKind::Dismissed => {
            UiPopupEffectKind::Close
        }
    };
    if !popup_matches_retained_state(surface, &popup) {
        let event = owned_popup_input_event(popup);
        let mut result = UiInputDispatchResult::new(event, UiDispatchReply::unhandled());
        result
            .diagnostics
            .notes
            .push("stale_popup_event_ignored".to_string());
        result.diagnostics.handled_phase = Some("popup.stale".to_string());
        return with_popup_route_policy(surface, result);
    }

    let event = UiInputEvent::Popup(popup.clone());
    let reply = UiDispatchReply::handled().with_effect(UiDispatchEffect::Popup {
        kind: effect_kind,
        popup_id: popup.popup_id,
        owner: popup.owner,
        anchor: popup.anchor,
    });
    let mut result = apply_dispatch_reply(surface, event, reply);
    result.diagnostics.routed = result.rejected_effects.is_empty();
    result.diagnostics.handled_phase = Some("popup.effect".to_string());
    with_popup_route_policy(surface, result)
}

fn owned_popup_input_event(popup: UiPopupInputEvent) -> UiInputEvent {
    UiInputEvent::Popup(popup)
}

fn with_popup_route_policy(
    surface: &UiSurface,
    mut result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let event = result.event.clone();
    annotate_route_policy(surface, &event, &mut result);
    annotate_result_route_steps(&mut result);
    result
}

fn popup_matches_retained_state(surface: &UiSurface, popup: &UiPopupInputEvent) -> bool {
    match popup.kind {
        UiPopupInputEventKind::OpenRequested => true,
        UiPopupInputEventKind::CloseRequested | UiPopupInputEventKind::Dismissed => surface
            .input
            .popup_matches(popup.popup_id.as_str(), popup.owner),
    }
}

#[cfg(test)]
#[path = "popup/stale_owned_event_tests.rs"]
mod stale_owned_event_tests;
