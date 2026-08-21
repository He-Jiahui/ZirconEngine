mod component_event;
mod drag_drop;
mod focus_pointer;
mod host_request;
mod ime_lifecycle;
mod link;
mod navigation;
mod node;
mod popup_tooltip;
mod redraw;
mod target;
mod text_services;
mod transaction;

use component_event::{apply_component_event_effect, component_event_report_for_effect};
use drag_drop::apply_drag_drop_effect;
use focus_pointer::apply_focus_pointer_effect;
use host_request::host_request_for_effect;
use ime_lifecycle::append_focus_input_method_lifecycle;
use link::apply_link_activation_effect;
use navigation::apply_navigation_effect;
use popup_tooltip::apply_popup_tooltip_effect;
use redraw::apply_redraw_effect;
use target::effect_target;
use text_services::apply_text_service_effect;
use transaction::UiInputTransaction;
use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchAppliedEffect, UiDispatchEffect, UiDispatchHostRequest,
        UiDispatchHostRequestKind, UiDispatchRejectedEffect, UiDispatchReply, UiDispatchReplyStep,
        UiInputDispatchResult, UiInputEvent,
    },
    event_ui::UiNodeId,
};

use super::super::surface::UiSurface;
use super::UiSurfaceInputEffectResult;
use super::{route_policy::annotate_route_policy, route_steps::annotate_result_route_steps};

pub(crate) fn apply_dispatch_reply(
    surface: &mut UiSurface,
    event: UiInputEvent,
    reply: UiDispatchReply,
) -> UiInputDispatchResult {
    let mut result = apply_dispatch_reply_core(surface, event, reply);
    let event = result.event.clone();
    annotate_route_policy(surface, &event, &mut result);
    annotate_result_route_steps(&mut result);
    result
}

fn apply_dispatch_reply_core(
    surface: &mut UiSurface,
    event: UiInputEvent,
    reply: UiDispatchReply,
) -> UiInputDispatchResult {
    let mut result = UiInputDispatchResult::new(event, reply.clone());
    result.diagnostics.routed = true;
    result.diagnostics.route_target = reply.handler;
    result.diagnostics.handled_phase =
        reply
            .phase
            .map(|phase| phase.as_str().to_string())
            .or_else(|| match reply.disposition {
                zircon_runtime_interface::ui::dispatch::UiDispatchDisposition::Unhandled => None,
                zircon_runtime_interface::ui::dispatch::UiDispatchDisposition::Handled => {
                    Some("reply".to_string())
                }
                zircon_runtime_interface::ui::dispatch::UiDispatchDisposition::Blocked => {
                    Some("blocked".to_string())
                }
                zircon_runtime_interface::ui::dispatch::UiDispatchDisposition::Passthrough => {
                    Some("passthrough".to_string())
                }
            });

    let transaction = UiInputTransaction::prepare(surface, &reply.effects);
    for (effect_index, effect) in reply.effects.iter().cloned().enumerate() {
        let rejected_before = result.rejected_effects.len();
        apply_dispatch_effect_at_index(surface, &mut result, effect_index, effect);
        if result.rejected_effects.len() != rejected_before && transaction.is_atomic() {
            let reason = result
                .rejected_effects
                .last()
                .map(|rejected| rejected.reason.clone())
                .unwrap_or_else(|| "effect rejected".to_string());
            transaction.abort(surface, &mut result, effect_index, reason);
            return result;
        }
    }
    transaction.commit(&mut result);

    result
}

pub(in crate::ui::surface::input) fn append_dispatch_effect_to_result(
    surface: &mut UiSurface,
    result: &mut UiInputDispatchResult,
    effect: UiDispatchEffect,
) {
    let effect_index = result.reply.effects.len();
    result.reply.effects.push(effect.clone());
    apply_dispatch_effect_at_index(surface, result, effect_index, effect);
}

fn apply_dispatch_effect_at_index(
    surface: &mut UiSurface,
    result: &mut UiInputDispatchResult,
    effect_index: usize,
    effect: UiDispatchEffect,
) {
    let high_precision_release_target = high_precision_release_target_for_effect(surface, &effect);
    match apply_effect(surface, &effect) {
        Ok(applied) => {
            let high_precision_released = high_precision_release_target
                .filter(|target| surface.input.high_precision_owner != Some(*target));
            if result.diagnostics.route_target.is_none() {
                result.diagnostics.route_target = applied.or_else(|| effect_target(&effect));
            }
            result.applied_effects.push(UiDispatchAppliedEffect {
                effect_index,
                effect: effect.clone(),
            });
            if let Some(host_request) = host_request_for_effect(effect_index, &effect, applied) {
                result.host_requests.push(host_request);
            }
            if let Some(target) = high_precision_released {
                result
                    .host_requests
                    .push(high_precision_release_host_request(effect_index, target));
            }
            if let Some(report) = component_event_report_for_effect(&effect) {
                result.component_events.push(report);
            }
            append_focus_input_method_lifecycle(surface, result, effect_index);
        }
        Err(error) => {
            result.rejected_effects.push(UiDispatchRejectedEffect {
                effect_index,
                effect,
                reason: error.to_string(),
            });
        }
    }
}

fn high_precision_release_target_for_effect(
    surface: &UiSurface,
    effect: &UiDispatchEffect,
) -> Option<UiNodeId> {
    let UiDispatchEffect::ReleasePointerCapture { target, .. } = effect else {
        return None;
    };
    (surface.input.high_precision_owner == Some(*target)).then_some(*target)
}

fn high_precision_release_host_request(
    effect_index: usize,
    target: UiNodeId,
) -> UiDispatchHostRequest {
    UiDispatchHostRequest {
        effect_index,
        request: UiDispatchHostRequestKind::HighPrecisionPointer {
            target,
            enabled: false,
        },
        reason: format!("release pointer capture disabled high precision for {target:?}"),
    }
}

pub(crate) fn apply_dispatch_reply_steps(
    surface: &mut UiSurface,
    event: UiInputEvent,
    steps: impl IntoIterator<Item = UiDispatchReplyStep>,
) -> UiInputDispatchResult {
    let merge = UiDispatchReply::merge_route(steps);
    let stopped = merge.stopped;
    let stopped_at = merge.stopped_at;
    let stopped_phase = merge.stopped_phase;
    let step_count = merge.step_count;
    let route_steps = merge.steps;
    let mut result = apply_dispatch_reply_core(surface, event, merge.reply);
    result.diagnostics.route_steps = route_steps;
    result
        .diagnostics
        .notes
        .push(format!("dispatch_steps={step_count}"));
    if stopped {
        result
            .diagnostics
            .notes
            .push("propagation_stopped".to_string());
    }
    if let Some(target) = stopped_at {
        result.diagnostics.route_target = Some(target);
    }
    if let Some(phase) = stopped_phase {
        result.diagnostics.handled_phase = Some(phase.as_str().to_string());
    }
    let event = result.event.clone();
    annotate_route_policy(surface, &event, &mut result);
    result
}

fn apply_effect(
    surface: &mut UiSurface,
    effect: &UiDispatchEffect,
) -> UiSurfaceInputEffectResult<Option<UiNodeId>> {
    match effect {
        UiDispatchEffect::SetFocus { .. }
        | UiDispatchEffect::ClearFocus { .. }
        | UiDispatchEffect::CapturePointer { .. }
        | UiDispatchEffect::ReleasePointerCapture { .. }
        | UiDispatchEffect::LockPointer { .. }
        | UiDispatchEffect::UnlockPointer { .. }
        | UiDispatchEffect::UseHighPrecisionPointer { .. } => {
            apply_focus_pointer_effect(surface, effect)
        }
        UiDispatchEffect::DragDrop { .. } => apply_drag_drop_effect(surface, effect),
        UiDispatchEffect::RequestNavigation { .. } => apply_navigation_effect(surface, effect),
        UiDispatchEffect::Popup { .. }
        | UiDispatchEffect::Tooltip { .. }
        | UiDispatchEffect::DismissTransientUi { .. } => {
            apply_popup_tooltip_effect(surface, effect)
        }
        UiDispatchEffect::RequestInputMethod { .. } | UiDispatchEffect::RequestClipboard { .. } => {
            apply_text_service_effect(surface, effect)
        }
        UiDispatchEffect::RequestLinkActivation { .. } => {
            apply_link_activation_effect(surface, effect)
        }
        UiDispatchEffect::DirtyRedraw { .. } => apply_redraw_effect(surface, effect),
        UiDispatchEffect::EmitComponentEvent { .. } => {
            apply_component_event_effect(surface, effect)
        }
    }
}
