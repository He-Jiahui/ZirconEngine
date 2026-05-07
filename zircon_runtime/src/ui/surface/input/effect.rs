use zircon_runtime_interface::ui::{
    dispatch::{
        UiComponentEmissionPolicy, UiComponentEventReport, UiDispatchAppliedEffect,
        UiDispatchEffect, UiDispatchHostRequest, UiDispatchHostRequestKind,
        UiDispatchRejectedEffect, UiDispatchReply, UiDispatchReplyStep, UiDragDropEffectKind,
        UiInputDispatchResult, UiInputEvent, UiInputMethodRequest, UiInputMethodRequestKind,
    },
    event_ui::UiNodeId,
    tree::UiDirtyFlags,
};

use crate::ui::tree::UiRuntimeTreeFocusExt;

use super::super::surface::UiSurface;
use super::require_valid_input_owner;

pub(crate) fn apply_dispatch_reply(
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

    for (effect_index, effect) in reply.effects.iter().cloned().enumerate() {
        match apply_effect(surface, &effect) {
            Ok(applied) => {
                if result.diagnostics.route_target.is_none() {
                    result.diagnostics.route_target = effect_target(&effect);
                }
                result.applied_effects.push(UiDispatchAppliedEffect {
                    effect_index,
                    effect: effect.clone(),
                });
                if let Some(host_request) = host_request_for_effect(effect_index, &effect, applied)
                {
                    result.host_requests.push(host_request);
                }
                if let UiDispatchEffect::EmitComponentEvent { target, event, .. } = effect {
                    result.component_events.push(UiComponentEventReport {
                        target,
                        event,
                        delivered: true,
                    });
                }
            }
            Err(reason) => {
                result.rejected_effects.push(UiDispatchRejectedEffect {
                    effect_index,
                    effect,
                    reason,
                });
            }
        }
    }

    result
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
    let mut result = apply_dispatch_reply(surface, event, merge.reply);
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
    result
}

fn apply_effect(
    surface: &mut UiSurface,
    effect: &UiDispatchEffect,
) -> Result<Option<UiNodeId>, String> {
    match effect {
        UiDispatchEffect::SetFocus { target, .. } => {
            surface
                .focus_node(*target)
                .map_err(|error| format!("focus rejected: {error}"))?;
            Ok(Some(*target))
        }
        UiDispatchEffect::ClearFocus { target, .. } => {
            if surface.focus.focused != Some(*target) {
                return Err("focus owner mismatch".to_string());
            }
            surface.clear_focus();
            if surface.input.input_method_owner == Some(*target) {
                surface.input.clear_input_method();
            }
            Ok(Some(*target))
        }
        UiDispatchEffect::CapturePointer {
            target, pointer_id, ..
        } => {
            require_valid_input_owner(surface, *target)?;
            if let Some(previous) = surface.focus.captured.filter(|owner| owner != target) {
                surface.input.clear_high_precision_for(previous);
            }
            surface.focus.captured = Some(*target);
            surface.input.captured_pointer_id = Some(*pointer_id);
            Ok(Some(*target))
        }
        UiDispatchEffect::ReleasePointerCapture {
            target, pointer_id, ..
        } => {
            if surface.focus.captured == Some(*target)
                && surface.input.captured_pointer_id == Some(*pointer_id)
            {
                surface.focus.captured = None;
                surface.input.clear_pointer_capture_for(*target);
                Ok(None)
            } else {
                Err("pointer capture belongs to a different or unknown pointer".to_string())
            }
        }
        UiDispatchEffect::LockPointer { target, policy } => {
            require_valid_input_owner(surface, *target)?;
            surface.input.pointer_lock_owner = Some(*target);
            surface.input.pointer_lock_policy = Some(*policy);
            Ok(Some(*target))
        }
        UiDispatchEffect::UnlockPointer { target, .. } => {
            if surface.input.pointer_lock_owner == Some(*target) {
                surface.input.pointer_lock_owner = None;
                surface.input.pointer_lock_policy = None;
                Ok(Some(*target))
            } else {
                Err("pointer lock owner mismatch".to_string())
            }
        }
        UiDispatchEffect::UseHighPrecisionPointer { target, enabled } => {
            require_valid_input_owner(surface, *target)?;
            if *enabled {
                if surface.focus.captured != Some(*target)
                    || surface.input.captured_pointer_id.is_none()
                {
                    return Err("high precision requires pointer capture".to_string());
                }
                surface.input.high_precision_owner = Some(*target);
            } else if surface.input.high_precision_owner == Some(*target) {
                surface.input.high_precision_owner = None;
            } else {
                return Err("high precision owner mismatch".to_string());
            }
            Ok(Some(*target))
        }
        UiDispatchEffect::DragDrop {
            target,
            kind,
            pointer_id,
            session_id,
            point,
            payload,
        } => {
            require_node(surface, *target)?;
            match kind {
                UiDragDropEffectKind::Begin => {
                    require_valid_input_owner(surface, *target)?;
                    surface.input.begin_drag_drop(
                        *target,
                        *target,
                        *pointer_id,
                        *session_id,
                        *point,
                        payload.clone(),
                    )?;
                    surface.focus.captured = Some(*target);
                    surface.input.captured_pointer_id = Some(*pointer_id);
                    Ok(Some(*target))
                }
                UiDragDropEffectKind::Update => {
                    surface.input.update_drag_drop(
                        *target,
                        *pointer_id,
                        *session_id,
                        *point,
                        payload.clone(),
                    )?;
                    Ok(Some(*target))
                }
                UiDragDropEffectKind::Accept => {
                    surface
                        .input
                        .accept_drag_drop(*target, *pointer_id, *session_id)?;
                    Ok(Some(*target))
                }
                UiDragDropEffectKind::Reject => {
                    surface
                        .input
                        .reject_drag_drop(*target, *pointer_id, *session_id)?;
                    Ok(Some(*target))
                }
                UiDragDropEffectKind::Complete | UiDragDropEffectKind::Cancel => {
                    if let Some(source) = surface.input.end_drag_drop(*pointer_id, *session_id)? {
                        if surface.focus.captured == Some(source) {
                            surface.focus.captured = None;
                        }
                        surface.input.clear_pointer_capture_for(source);
                    }
                    Ok(Some(*target))
                }
            }
        }
        UiDispatchEffect::RequestNavigation { kind, .. } => {
            let route = surface
                .route_navigation_event(*kind)
                .map_err(|error| format!("navigation route rejected: {error}"))?;
            let target = surface
                .tree
                .next_focusable_target(route.target, *kind)
                .map_err(|error| format!("navigation target rejected: {error}"))?;
            if let Some(target) = target {
                surface
                    .focus_node(target)
                    .map_err(|error| format!("navigation focus rejected: {error}"))?;
                Ok(Some(target))
            } else {
                Ok(route.target)
            }
        }
        UiDispatchEffect::Popup {
            kind,
            popup_id,
            anchor,
        } => {
            match kind {
                zircon_runtime_interface::ui::dispatch::UiPopupEffectKind::Open => {
                    surface.input.open_popup(popup_id.clone(), *anchor);
                }
                zircon_runtime_interface::ui::dispatch::UiPopupEffectKind::Close => {
                    surface.input.close_popup(popup_id.as_str());
                }
                zircon_runtime_interface::ui::dispatch::UiPopupEffectKind::Toggle => {
                    surface.input.toggle_popup(popup_id.clone(), *anchor);
                }
            }
            Ok(None)
        }
        UiDispatchEffect::Tooltip { kind, tooltip_id } => {
            match kind {
                zircon_runtime_interface::ui::dispatch::UiTooltipEffectKind::Arm => {
                    surface.input.arm_tooltip(tooltip_id.clone());
                }
                zircon_runtime_interface::ui::dispatch::UiTooltipEffectKind::Show => {
                    surface.input.show_tooltip(tooltip_id.clone());
                }
                zircon_runtime_interface::ui::dispatch::UiTooltipEffectKind::Hide
                | zircon_runtime_interface::ui::dispatch::UiTooltipEffectKind::Cancel => {
                    surface.input.clear_tooltip(tooltip_id.as_str());
                }
            }
            Ok(None)
        }
        UiDispatchEffect::RequestInputMethod { request } => {
            apply_input_method_request(surface, request)
        }
        UiDispatchEffect::DirtyRedraw { target, dirty, .. } => {
            let node = surface
                .tree
                .nodes
                .get_mut(target)
                .ok_or_else(|| format!("missing dirty target {target:?}"))?;
            merge_dirty(&mut node.dirty, *dirty);
            node.state_flags.dirty |= dirty.any();
            Ok(Some(*target))
        }
        UiDispatchEffect::EmitComponentEvent { target, policy, .. } => {
            require_node(surface, *target)?;
            match policy {
                UiComponentEmissionPolicy::Immediate
                | UiComponentEmissionPolicy::Queue
                | UiComponentEmissionPolicy::Coalesce => Ok(Some(*target)),
            }
        }
    }
}

fn apply_input_method_request(
    surface: &mut UiSurface,
    request: &UiInputMethodRequest,
) -> Result<Option<UiNodeId>, String> {
    require_valid_input_owner(surface, request.owner)?;
    match request.kind {
        UiInputMethodRequestKind::Enable => {
            surface.input.input_method_owner = Some(request.owner);
            surface.input.input_method_request = Some(request.clone());
        }
        UiInputMethodRequestKind::Reset | UiInputMethodRequestKind::UpdateCursor => {
            if surface.input.input_method_owner == Some(request.owner) {
                surface.input.input_method_request = Some(request.clone());
            } else {
                return Err("input method owner mismatch".to_string());
            }
        }
        UiInputMethodRequestKind::Disable => {
            if surface.input.input_method_owner == Some(request.owner) {
                surface.input.clear_input_method();
            } else {
                return Err("input method owner mismatch".to_string());
            }
        }
    }
    Ok(Some(request.owner))
}

fn host_request_for_effect(
    effect_index: usize,
    effect: &UiDispatchEffect,
    target: Option<UiNodeId>,
) -> Option<UiDispatchHostRequest> {
    let request = match effect {
        UiDispatchEffect::LockPointer { target, policy } => {
            UiDispatchHostRequestKind::PointerLock {
                target: *target,
                policy: *policy,
            }
        }
        UiDispatchEffect::UnlockPointer { policy, .. } => {
            UiDispatchHostRequestKind::PointerUnlock { policy: *policy }
        }
        UiDispatchEffect::UseHighPrecisionPointer { target, enabled } => {
            UiDispatchHostRequestKind::HighPrecisionPointer {
                target: *target,
                enabled: *enabled,
            }
        }
        UiDispatchEffect::Popup {
            kind,
            popup_id,
            anchor,
        } => UiDispatchHostRequestKind::Popup {
            kind: *kind,
            popup_id: popup_id.clone(),
            anchor: *anchor,
        },
        UiDispatchEffect::Tooltip { kind, tooltip_id } => UiDispatchHostRequestKind::Tooltip {
            kind: *kind,
            tooltip_id: tooltip_id.clone(),
        },
        UiDispatchEffect::RequestInputMethod { request } => {
            UiDispatchHostRequestKind::InputMethod(request.clone())
        }
        _ => return None,
    };
    Some(UiDispatchHostRequest {
        effect_index,
        request,
        reason: target
            .map(|node_id| format!("effect applied for {node_id:?}"))
            .unwrap_or_else(|| "effect applied".to_string()),
    })
}

fn effect_target(effect: &UiDispatchEffect) -> Option<UiNodeId> {
    match effect {
        UiDispatchEffect::SetFocus { target, .. }
        | UiDispatchEffect::ClearFocus { target, .. }
        | UiDispatchEffect::CapturePointer { target, .. }
        | UiDispatchEffect::ReleasePointerCapture { target, .. }
        | UiDispatchEffect::LockPointer { target, .. }
        | UiDispatchEffect::UnlockPointer { target, .. }
        | UiDispatchEffect::UseHighPrecisionPointer { target, .. }
        | UiDispatchEffect::DragDrop { target, .. }
        | UiDispatchEffect::DirtyRedraw { target, .. }
        | UiDispatchEffect::EmitComponentEvent { target, .. } => Some(*target),
        UiDispatchEffect::RequestInputMethod { request } => Some(request.owner),
        UiDispatchEffect::RequestNavigation { .. }
        | UiDispatchEffect::Popup { .. }
        | UiDispatchEffect::Tooltip { .. } => None,
    }
}

fn require_node(surface: &UiSurface, node_id: UiNodeId) -> Result<(), String> {
    surface
        .tree
        .nodes
        .contains_key(&node_id)
        .then_some(())
        .ok_or_else(|| format!("missing node {node_id:?}"))
}

fn merge_dirty(target: &mut UiDirtyFlags, dirty: UiDirtyFlags) {
    target.layout |= dirty.layout;
    target.hit_test |= dirty.hit_test;
    target.render |= dirty.render;
    target.style |= dirty.style;
    target.text |= dirty.text;
    target.input |= dirty.input;
    target.visible_range |= dirty.visible_range;
}
