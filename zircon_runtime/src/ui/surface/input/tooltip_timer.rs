//! 将提示框布防、到期和取消通知转换为保留状态效果。
//! 输入管理器拥有计时队列；本层判定通知适用性，效果应用器负责状态修改及拒绝回执。

use zircon_runtime_interface::ui::dispatch::{
    UiDispatchEffect, UiDispatchReply, UiInputDispatchResult, UiInputEvent, UiTooltipEffectKind,
    UiTooltipTimerInputEvent, UiTooltipTimerInputEventKind,
};

use super::super::surface::UiSurface;
use super::{
    apply_dispatch_reply, route_policy::annotate_route_policy,
    route_steps::annotate_result_route_steps,
};

/// 延迟提示通知只在 ID/owner 仍匹配当前保留提示时应用，避免其他控件的旧计时器改变当前提示。
///
/// # 参数与返回
/// `surface` 是当前保留状态；`tooltip` 移交通知，返回拥有原事件及效果应用记录的回执。
/// # 调用时机与约束
/// 统一输入入口收到通知时调用。布防可建立新候选；到期与取消才要求匹配现存提示。
/// 通知的可选 owner 为空时只比较标识；指定 owner 时还需匹配所有者。
/// 匹配通过后仍可能拒绝效果，拒绝原因保留在回执中。
/// # 参考
/// 计时来源见 [`UiInputManager::tick`](crate::ui::dispatch::UiInputManager::tick)。
pub(super) fn dispatch_tooltip_timer_input(
    surface: &mut UiSurface,
    tooltip: UiTooltipTimerInputEvent,
) -> UiInputDispatchResult {
    let effect_kind = match tooltip.kind {
        UiTooltipTimerInputEventKind::Armed => UiTooltipEffectKind::Arm,
        UiTooltipTimerInputEventKind::Elapsed => UiTooltipEffectKind::Show,
        UiTooltipTimerInputEventKind::Canceled => UiTooltipEffectKind::Cancel,
    };
    if !tooltip_timer_matches_retained_state(surface, &tooltip) {
        let event = owned_tooltip_timer_event(tooltip);
        let mut result = UiInputDispatchResult::new(event, UiDispatchReply::unhandled());
        result
            .diagnostics
            .notes
            .push("stale_tooltip_timer_ignored".to_string());
        result.diagnostics.handled_phase = Some("tooltip.stale".to_string());
        return with_tooltip_route_policy(surface, result);
    }
    let event = UiInputEvent::TooltipTimer(tooltip.clone());
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

/// 失效分支直接移交通知中的字符串分配，供未处理回执保留原输入。
fn owned_tooltip_timer_event(tooltip: UiTooltipTimerInputEvent) -> UiInputEvent {
    UiInputEvent::TooltipTimer(tooltip)
}

/// 为失效或已应用效果的回执补齐策略；步骤投影保留已有记录。
fn with_tooltip_route_policy(
    surface: &UiSurface,
    mut result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let event = result.event.clone();
    annotate_route_policy(surface, &event, &mut result);
    annotate_result_route_steps(&mut result);
    result
}

/// 布防允许新建候选；其余通知的匹配规则由当前提示状态及可选所有者共同决定。
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

#[cfg(test)]
#[path = "tooltip_timer/tests/stale_owned_event.rs"]
mod stale_owned_event_tests;
