//! 将一次指针派发的记录投影为统一输入回复，并合并随后产生的文本结果。
//!
//! 路由和状态变更由调用方及专属 owner 完成；本模块组装拥有型效果与回执记录。
//! 借入的指针结果只在投影期间使用，文本结果按值消费；返回记录不借用两者。
//!
//! 主要入口为 [`pointer_reply`] 和 [`merge_pointer_text_result`]。

use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchDisposition, UiDispatchEffect, UiDispatchPhase, UiDispatchReply,
        UiFocusEffectReason, UiInputDispatchResult, UiPointerCaptureReason,
        UiPointerDispatchEffect, UiPointerDispatchResult, UiPointerId, UiRedrawRequestReason,
    },
    event_ui::UiNodeId,
};

/// 把已完成的一次指针派发投影为统一回复。
///
/// # 参数与返回值
/// 借入派发记录，`pointer_id` 用于捕获及释放效果的身份；返回值拥有新建的效果列表。
/// 返回记录选择阻断或处理节点，必要时以最后一个组件事件节点补足处理者。
///
/// # 约束与时机
/// 本函数读取已产生的状态与事件，不再次调用处理器或应用效果。
/// 阶段可采用兜底值，不能仅据此证明相应阶段实际执行过处理器。
///
/// # 参考
/// [`UiDispatchReply`]、[`UiPointerDispatchResult`]。
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

/// 估计固定效果加逐项或兜底重绘所需的条目容量。
///
/// # 参数与返回值
/// 三个计数分别来自固定状态变化、调用记录数和兜底目标数。
/// 逐项已产生重绘时不会再追加兜底重绘，因而后两项取较大值后与固定数做饱和相加。
///
/// # 约束
/// 这里只估计条目数；不设置内存预算，也不保证后续容量申请成功。
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

/// 顺序投影非空 dirty 请求，并记录最后匹配的处理与重绘阶段。
///
/// # 参数与返回值
/// 借入派发记录，按节点与可选处理者匹配；把新效果追加到传入向量，不清空原内容。
/// 返回值分别保留最后一次节点匹配和最后一次 dirty/damage 调用的阶段。
///
/// # 约束
/// damage 请求在此只参与阶段选择；此处不会为它另建 damage 效果。
/// 记录阶段不要求 dirty 非空；只有非空 dirty 才追加重绘效果。
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
/// 消费文本侧结果，将其效果与回执按原有顺序附加到指针结果。
///
/// # 参数与所有权
/// `result` 原地接收累计结果；`text_result` 按值消费，内部向量迁入累计结果。
///
/// # 约束
/// 仅落在文本效果序列内的本地索引会平移；越界索引保持原值，由调用方保证回执身份有效。
/// 本函数不校验两份结果是否属于同一输入，也不重新应用效果或执行宿主请求。
/// 已有阻断结论只保护回复头部的选择，后续记录仍会附加。
///
/// # 参考
/// [`UiInputDispatchResult`]。
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
