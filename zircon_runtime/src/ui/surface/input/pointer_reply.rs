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

// 回复效果按捕获/释放、焦点变化、逐 invocation 的 dirty 请求和兜底重绘顺序写入；同一遍扫描同时记录 handler 与 redraw phase，供后续 route trace 解释实际终止阶段。
fn pointer_reply_effects(
    routed_result: &UiPointerDispatchResult,
    pointer_id: UiPointerId,
    handler: Option<UiNodeId>,
) -> (Vec<UiDispatchEffect>, PointerInvocationPhases) {
    let release_target = pointer_release_target(routed_result);
    let fixed_effect_count = usize::from(routed_result.captured_by.is_some())
        + usize::from(release_target.is_some())
        + usize::from(routed_result.focus_changed_to.is_some())
        + usize::from(routed_result.focus_cleared && routed_result.route.focused.is_some());
    let fallback_effect_count = routed_result
        .route
        .target
        .map_or(routed_result.route.root_targets.len(), |_| 1);
    let effect_capacity = pointer_reply_effect_capacity_hint(
        fixed_effect_count,
        routed_result.invocations.len(),
        fallback_effect_count,
    );
    let mut effects = Vec::with_capacity(effect_capacity);
    if let Some(target) = routed_result.captured_by {
        effects.push(UiDispatchEffect::CapturePointer {
            target,
            pointer_id,
            reason: UiPointerCaptureReason::Press,
        });
    }
    if let Some(target) = release_target {
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

fn pointer_reply_effect_capacity_hint(
    fixed_effect_count: usize,
    invocation_count: usize,
    fallback_effect_count: usize,
) -> usize {
    fixed_effect_count.saturating_add(invocation_count.max(fallback_effect_count))
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
#[path = "pointer_reply/tests/single_pass_tests.rs"]
mod single_pass_tests;

#[cfg(test)]
#[path = "pointer_reply/tests/capacity_tests.rs"]
mod capacity_tests;

// 指针路由先产生基础回复，文本编辑随后追加自己的 effects 与事件；保留已有 Blocked 状态，并把文本侧 effect 索引整体平移，保证 applied/rejected/host request 仍指向合并后的序列。
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
    result.reply.effects.reserve(text_effect_count);
    result
        .applied_effects
        .reserve(text_result.applied_effects.len());
    result
        .component_events
        .reserve(text_result.component_events.len());
    result
        .widget_events
        .reserve(text_result.widget_events.len());
    result
        .binding_reports
        .reserve(text_result.binding_reports.len());
    result
        .host_requests
        .reserve(text_result.host_requests.len());
    result
        .rejected_effects
        .reserve(text_result.rejected_effects.len());
    result
        .diagnostics
        .notes
        .reserve(text_result.diagnostics.notes.len());
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
    result.widget_events.extend(text_result.widget_events);
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
#[path = "tests/pointer_reply.rs"]
mod tests;
