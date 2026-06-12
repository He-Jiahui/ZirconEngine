use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchDisposition, UiDispatchEffect, UiDispatchPhase, UiDispatchReply,
        UiFocusEffectReason, UiInputDispatchResult, UiPointerCaptureReason,
        UiPointerDispatchEffect, UiPointerDispatchResult, UiPointerId, UiRedrawRequestReason,
    },
    event_ui::UiNodeId,
};

pub(super) fn pointer_reply(
    legacy: &UiPointerDispatchResult,
    pointer_id: UiPointerId,
) -> UiDispatchReply {
    let effects = pointer_reply_effects(legacy, pointer_id);
    let component_handler = pointer_component_handler(legacy);
    // A delivered component event is the unified input equivalent of a handled widget reply.
    let disposition = if legacy.blocked_by.is_some() {
        UiDispatchDisposition::Blocked
    } else if legacy.handled_by.is_some() || component_handler.is_some() || !effects.is_empty() {
        UiDispatchDisposition::Handled
    } else if !legacy.passthrough.is_empty() {
        UiDispatchDisposition::Passthrough
    } else {
        UiDispatchDisposition::Unhandled
    };
    UiDispatchReply {
        disposition,
        handler: legacy
            .handled_by
            .or(legacy.blocked_by)
            .or(component_handler),
        phase: pointer_reply_phase(legacy, component_handler),
        effects,
    }
}

pub(super) fn pointer_component_handler(legacy: &UiPointerDispatchResult) -> Option<UiNodeId> {
    legacy.component_events.last().map(|event| event.node_id)
}

fn pointer_reply_phase(
    legacy: &UiPointerDispatchResult,
    component_handler: Option<UiNodeId>,
) -> Option<UiDispatchPhase> {
    let handler = legacy.handled_by.or(legacy.blocked_by);
    if let Some(handler) = handler {
        return legacy
            .invocations
            .iter()
            .rev()
            .find(|invocation| invocation.node_id == handler)
            .map(|invocation| invocation.phase)
            .or(Some(UiDispatchPhase::Bubble));
    }
    if component_handler.is_some() {
        return Some(UiDispatchPhase::Target);
    }
    legacy
        .invocations
        .iter()
        .rev()
        .find(|invocation| {
            matches!(
                invocation.effect,
                UiPointerDispatchEffect::RequestDirty(_)
                    | UiPointerDispatchEffect::RequestDamage(_)
            )
        })
        .map(|invocation| invocation.phase)
        .or(Some(UiDispatchPhase::Target))
}

fn pointer_reply_effects(
    legacy: &UiPointerDispatchResult,
    pointer_id: UiPointerId,
) -> Vec<UiDispatchEffect> {
    let mut effects = Vec::new();
    if let Some(target) = legacy.captured_by {
        effects.push(UiDispatchEffect::CapturePointer {
            target,
            pointer_id,
            reason: UiPointerCaptureReason::Press,
        });
    }
    if let Some(target) = pointer_release_target(legacy) {
        effects.push(UiDispatchEffect::ReleasePointerCapture {
            target,
            pointer_id,
            reason: UiPointerCaptureReason::Cancel,
        });
    }
    if let Some(target) = legacy.focus_changed_to {
        effects.push(UiDispatchEffect::SetFocus {
            target,
            reason: UiFocusEffectReason::Input,
        });
    }
    if legacy.focus_cleared {
        if let Some(target) = legacy.route.focused {
            effects.push(UiDispatchEffect::ClearFocus {
                target,
                reason: UiFocusEffectReason::Input,
            });
        }
    }
    for invocation in &legacy.invocations {
        if let UiPointerDispatchEffect::RequestDirty(dirty) = invocation.effect {
            if dirty.any() {
                effects.push(UiDispatchEffect::DirtyRedraw {
                    target: invocation.node_id,
                    dirty,
                    reason: UiRedrawRequestReason::Input,
                });
            }
        }
    }
    if legacy.requested_dirty.any()
        && !effects
            .iter()
            .any(|effect| matches!(effect, UiDispatchEffect::DirtyRedraw { .. }))
    {
        if let Some(target) = legacy.route.target {
            effects.push(UiDispatchEffect::DirtyRedraw {
                target,
                dirty: legacy.requested_dirty,
                reason: UiRedrawRequestReason::Input,
            });
        } else {
            effects.extend(legacy.route.root_targets.iter().copied().map(|target| {
                UiDispatchEffect::DirtyRedraw {
                    target,
                    dirty: legacy.requested_dirty,
                    reason: UiRedrawRequestReason::Input,
                }
            }));
        }
    }
    effects
}

fn pointer_release_target(legacy: &UiPointerDispatchResult) -> Option<UiNodeId> {
    legacy.released_capture.or_else(|| {
        (legacy.diagnostics.capture_released && legacy.captured_by.is_none())
            .then_some(legacy.route.captured)
            .flatten()
    })
}

pub(super) fn merge_pointer_text_result(
    result: &mut UiInputDispatchResult,
    text_result: UiInputDispatchResult,
) {
    if !matches!(result.reply.disposition, UiDispatchDisposition::Blocked) {
        result.reply.disposition = text_result.reply.disposition;
        result.reply.handler = text_result.reply.handler.or(result.reply.handler);
        result.reply.phase = text_result.reply.phase.or(result.reply.phase);
    }
    let mut effect_index_map = Vec::new();
    for (local_effect_index, effect) in text_result.reply.effects.into_iter().enumerate() {
        let effect_index = result.reply.effects.len();
        result.reply.effects.push(effect);
        effect_index_map.push((local_effect_index, effect_index));
    }
    result
        .applied_effects
        .extend(text_result.applied_effects.into_iter().map(|mut applied| {
            if let Some(effect_index) =
                remap_text_effect_index(&effect_index_map, applied.effect_index)
            {
                applied.effect_index = effect_index;
            }
            applied
        }));
    result.component_events.extend(text_result.component_events);
    result.binding_reports.extend(text_result.binding_reports);
    result
        .host_requests
        .extend(text_result.host_requests.into_iter().map(|mut request| {
            if let Some(effect_index) =
                remap_text_effect_index(&effect_index_map, request.effect_index)
            {
                request.effect_index = effect_index;
            }
            request
        }));
    result.rejected_effects.extend(
        text_result
            .rejected_effects
            .into_iter()
            .map(|mut rejected| {
                if let Some(effect_index) =
                    remap_text_effect_index(&effect_index_map, rejected.effect_index)
                {
                    rejected.effect_index = effect_index;
                }
                rejected
            }),
    );
    result.drag = text_result.drag.or(result.drag);
    result.diagnostics.routed |= text_result.diagnostics.routed;
    result.diagnostics.route_target = text_result
        .diagnostics
        .route_target
        .or(result.diagnostics.route_target);
    result.diagnostics.handled_phase = text_result
        .diagnostics
        .handled_phase
        .or(result.diagnostics.handled_phase.take());
    result
        .diagnostics
        .notes
        .extend(text_result.diagnostics.notes);
}

fn remap_text_effect_index(
    effect_index_map: &[(usize, usize)],
    local_effect_index: usize,
) -> Option<usize> {
    effect_index_map
        .iter()
        .find_map(|(local, merged)| (*local == local_effect_index).then_some(*merged))
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::{
        dispatch::{
            UiDispatchAppliedEffect, UiDispatchHostRequest, UiDispatchHostRequestKind,
            UiDispatchRejectedEffect, UiInputEvent, UiInputEventMetadata, UiInputSequence,
            UiInputTimestamp, UiPointerEvent, UiPointerInputEvent, UiPopupEffectKind,
        },
        layout::UiPoint,
        surface::{UiPointerButton, UiPointerEventKind},
    };

    #[test]
    fn merge_pointer_text_result_preserves_text_effect_statuses_when_rebasing_indexes() {
        let pointer_effect = release_pointer_effect(UiNodeId::new(1), UiPointerId::new(1));
        let mut result = UiInputDispatchResult::new(
            pointer_event(),
            UiDispatchReply::handled().with_effect(pointer_effect.clone()),
        );
        result.applied_effects.push(UiDispatchAppliedEffect {
            effect_index: 0,
            effect: pointer_effect.clone(),
        });

        let text_applied = focus_effect(UiNodeId::new(2));
        let text_rejected = popup_effect("stale-popup");
        let mut text_result = UiInputDispatchResult::new(
            pointer_event(),
            UiDispatchReply::handled().with_effects([text_applied.clone(), text_rejected.clone()]),
        );
        text_result.applied_effects.push(UiDispatchAppliedEffect {
            effect_index: 0,
            effect: text_applied.clone(),
        });
        text_result.rejected_effects.push(UiDispatchRejectedEffect {
            effect_index: 1,
            effect: text_rejected.clone(),
            reason: "invalid text popup owner".to_string(),
        });
        text_result.host_requests.push(UiDispatchHostRequest {
            effect_index: 1,
            request: UiDispatchHostRequestKind::Popup {
                kind: UiPopupEffectKind::Open,
                popup_id: "stale-popup".to_string(),
                anchor: Some(UiPoint::new(10.0, 4.0)),
            },
            reason: "text popup".to_string(),
        });

        merge_pointer_text_result(&mut result, text_result);

        assert_eq!(
            result.reply.effects,
            vec![
                pointer_effect.clone(),
                text_applied.clone(),
                text_rejected.clone()
            ]
        );
        assert_eq!(
            result.applied_effects,
            vec![
                UiDispatchAppliedEffect {
                    effect_index: 0,
                    effect: pointer_effect,
                },
                UiDispatchAppliedEffect {
                    effect_index: 1,
                    effect: text_applied,
                },
            ]
        );
        assert_eq!(result.rejected_effects.len(), 1);
        assert_eq!(result.rejected_effects[0].effect_index, 2);
        assert_eq!(result.rejected_effects[0].effect, text_rejected);
        assert_eq!(
            result.rejected_effects[0].reason,
            "invalid text popup owner"
        );
        assert_eq!(result.host_requests.len(), 1);
        assert_eq!(result.host_requests[0].effect_index, 2);
    }

    fn pointer_event() -> UiInputEvent {
        UiInputEvent::Pointer(UiPointerInputEvent {
            metadata: UiInputEventMetadata::new(
                UiInputTimestamp::from_micros(1),
                UiInputSequence::new(1),
            ),
            event: UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(0.0, 0.0))
                .with_button(UiPointerButton::Secondary),
            precise_scroll: None,
        })
    }

    fn release_pointer_effect(target: UiNodeId, pointer_id: UiPointerId) -> UiDispatchEffect {
        UiDispatchEffect::ReleasePointerCapture {
            target,
            pointer_id,
            reason: UiPointerCaptureReason::Cancel,
        }
    }

    fn focus_effect(target: UiNodeId) -> UiDispatchEffect {
        UiDispatchEffect::SetFocus {
            target,
            reason: UiFocusEffectReason::Input,
        }
    }

    fn popup_effect(popup_id: &str) -> UiDispatchEffect {
        UiDispatchEffect::Popup {
            kind: UiPopupEffectKind::Open,
            popup_id: popup_id.to_string(),
            owner: Some(UiNodeId::new(2)),
            anchor: Some(UiPoint::new(10.0, 4.0)),
        }
    }
}
