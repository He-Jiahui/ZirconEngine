use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchDisposition, UiDispatchEffect, UiDispatchPhase, UiDispatchReply,
        UiFocusEffectReason, UiInputDispatchResult, UiPointerCaptureReason,
        UiPointerDispatchEffect, UiPointerDispatchResult, UiPointerId, UiRedrawRequestReason,
    },
    event_ui::UiNodeId,
};

pub(super) fn pointer_reply(
    routed_result: &UiPointerDispatchResult,
    pointer_id: UiPointerId,
) -> UiDispatchReply {
    let component_handler = pointer_component_handler(routed_result);
    let handler = routed_result.handled_by.or(routed_result.blocked_by);
    let (effects, invocation_phases) = pointer_reply_effects(routed_result, pointer_id, handler);
    // A delivered component event is the unified input equivalent of a handled widget reply.
    let disposition = if routed_result.blocked_by.is_some() {
        UiDispatchDisposition::Blocked
    } else if routed_result.handled_by.is_some()
        || component_handler.is_some()
        || !effects.is_empty()
    {
        UiDispatchDisposition::Handled
    } else if !routed_result.passthrough.is_empty() {
        UiDispatchDisposition::Passthrough
    } else {
        UiDispatchDisposition::Unhandled
    };
    UiDispatchReply {
        disposition,
        handler: handler.or(component_handler),
        phase: if handler.is_some() {
            invocation_phases
                .handler_phase
                .or(Some(UiDispatchPhase::Bubble))
        } else if component_handler.is_some() {
            Some(UiDispatchPhase::Target)
        } else {
            invocation_phases
                .redraw_phase
                .or(Some(UiDispatchPhase::Target))
        },
        effects,
    }
}

pub(super) fn pointer_component_handler(
    routed_result: &UiPointerDispatchResult,
) -> Option<UiNodeId> {
    routed_result
        .component_events
        .last()
        .map(|event| event.node_id)
}

fn pointer_reply_effects(
    routed_result: &UiPointerDispatchResult,
    pointer_id: UiPointerId,
    handler: Option<UiNodeId>,
) -> (Vec<UiDispatchEffect>, PointerInvocationPhases) {
    let mut effects = Vec::new();
    if let Some(target) = routed_result.captured_by {
        effects.push(UiDispatchEffect::CapturePointer {
            target,
            pointer_id,
            reason: UiPointerCaptureReason::Press,
        });
    }
    if let Some(target) = pointer_release_target(routed_result) {
        effects.push(UiDispatchEffect::ReleasePointerCapture {
            target,
            pointer_id,
            reason: UiPointerCaptureReason::Cancel,
        });
    }
    if let Some(target) = routed_result.focus_changed_to {
        effects.push(UiDispatchEffect::SetFocus {
            target,
            reason: UiFocusEffectReason::Input,
        });
    }
    if routed_result.focus_cleared {
        if let Some(target) = routed_result.route.focused {
            effects.push(UiDispatchEffect::ClearFocus {
                target,
                reason: UiFocusEffectReason::Input,
            });
        }
    }
    let invocation_phases = scan_pointer_invocations(routed_result, handler, &mut effects);
    if routed_result.requested_dirty.any()
        && !effects
            .iter()
            .any(|effect| matches!(effect, UiDispatchEffect::DirtyRedraw { .. }))
    {
        if let Some(target) = routed_result.route.target {
            effects.push(UiDispatchEffect::DirtyRedraw {
                target,
                dirty: routed_result.requested_dirty,
                reason: UiRedrawRequestReason::Input,
            });
        } else {
            effects.extend(
                routed_result
                    .route
                    .root_targets
                    .iter()
                    .copied()
                    .map(|target| UiDispatchEffect::DirtyRedraw {
                        target,
                        dirty: routed_result.requested_dirty,
                        reason: UiRedrawRequestReason::Input,
                    }),
            );
        }
    }
    (effects, invocation_phases)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct PointerInvocationPhases {
    handler_phase: Option<UiDispatchPhase>,
    redraw_phase: Option<UiDispatchPhase>,
}

fn scan_pointer_invocations(
    routed_result: &UiPointerDispatchResult,
    handler: Option<UiNodeId>,
    effects: &mut Vec<UiDispatchEffect>,
) -> PointerInvocationPhases {
    let mut phases = PointerInvocationPhases::default();
    for invocation in &routed_result.invocations {
        if Some(invocation.node_id) == handler {
            phases.handler_phase = Some(invocation.phase);
        }
        if matches!(
            invocation.effect,
            UiPointerDispatchEffect::RequestDirty(_) | UiPointerDispatchEffect::RequestDamage(_)
        ) {
            phases.redraw_phase = Some(invocation.phase);
        }
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
    phases
}

fn pointer_release_target(routed_result: &UiPointerDispatchResult) -> Option<UiNodeId> {
    routed_result.released_capture.or_else(|| {
        (routed_result.diagnostics.capture_released && routed_result.captured_by.is_none())
            .then_some(routed_result.route.captured)
            .flatten()
    })
}

#[cfg(test)]
#[path = "pointer_reply/single_pass_tests.rs"]
mod single_pass_tests;

pub(super) fn merge_pointer_text_result(
    result: &mut UiInputDispatchResult,
    text_result: UiInputDispatchResult,
) {
    if !matches!(result.reply.disposition, UiDispatchDisposition::Blocked) {
        result.reply.disposition = text_result.reply.disposition;
        result.reply.handler = text_result.reply.handler.or(result.reply.handler);
        result.reply.phase = text_result.reply.phase.or(result.reply.phase);
    }
    let effect_index_offset = result.reply.effects.len();
    let text_effect_count = text_result.reply.effects.len();
    for (local_effect_index, effect) in text_result.reply.effects.into_iter().enumerate() {
        debug_assert_eq!(
            result.reply.effects.len(),
            effect_index_offset + local_effect_index
        );
        result.reply.effects.push(effect);
    }
    result
        .applied_effects
        .extend(text_result.applied_effects.into_iter().map(|mut applied| {
            if applied.effect_index < text_effect_count {
                applied.effect_index += effect_index_offset;
            }
            applied
        }));
    result.component_events.extend(text_result.component_events);
    result.binding_reports.extend(text_result.binding_reports);
    result
        .host_requests
        .extend(text_result.host_requests.into_iter().map(|mut request| {
            if request.effect_index < text_effect_count {
                request.effect_index += effect_index_offset;
            }
            request
        }));
    result.rejected_effects.extend(
        text_result
            .rejected_effects
            .into_iter()
            .map(|mut rejected| {
                if rejected.effect_index < text_effect_count {
                    rejected.effect_index += effect_index_offset;
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
