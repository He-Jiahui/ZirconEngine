//! 把通知到期输入映射为当前绑定的提交报告。
//! 输入管理器持有到期队列；本层读取当前通知身份，回执交付后由绑定消费者处理关闭语义。

use zircon_runtime_interface::ui::dispatch::{
    UiDispatchPhase, UiDispatchReply, UiInputDispatchResult, UiInputEvent, UiToastTimerInputEvent,
};

use super::super::surface::UiSurface;
use super::{route_policy::annotate_route_policy, route_steps::annotate_result_route_steps};

/// 到期后重新核对 toast ID 和目标，再投递关闭语义事件；不存在的目标返回未处理回执。
///
/// # 参数与返回
/// `surface` 提供当前通知状态与绑定；`toast` 移交拥有的通知，回执继续拥有其标识字符串。
/// # 调用时机与约束
/// 统一输入入口收到到期通知后调用；目标需仍可交互且当前标识一致。
/// 空报告或绑定查询错误都返回未处理回执；本层返回组件报告供后续消费者执行。
/// # 参考
/// 到期队列由 [`UiInputManager::tick`](crate::ui::dispatch::UiInputManager::tick) 消费。
pub(super) fn dispatch_toast_timer_input(
    surface: &mut UiSurface,
    toast: UiToastTimerInputEvent,
) -> UiInputDispatchResult {
    let component_events = surface
        .apply_default_toast_timeout_component_event(toast.target, toast.toast_id.as_str())
        .unwrap_or_default();
    let (target, event) = into_toast_timer_event(toast);
    if component_events.is_empty() {
        let mut result = UiInputDispatchResult::new(event, UiDispatchReply::unhandled());
        result
            .diagnostics
            .notes
            .push("stale_toast_timer_ignored".to_string());
        result.diagnostics.handled_phase = Some("toast_timer.stale".to_string());
        return with_toast_route_policy(surface, result);
    }

    let reply = UiDispatchReply::handled()
        .from_handler(target)
        .in_phase(UiDispatchPhase::DefaultAction);
    let mut result = UiInputDispatchResult::new(event, reply);
    result.diagnostics.routed = true;
    result.diagnostics.route_target = Some(target);
    result.diagnostics.handled_phase = Some("toast_timer.component_event".to_string());
    result.component_events = component_events;
    with_toast_route_policy(surface, result)
}

/// 保留可复制的目标，并将通知及其字符串分配整体移入事件。
fn into_toast_timer_event(
    toast: UiToastTimerInputEvent,
) -> (
    zircon_runtime_interface::ui::event_ui::UiNodeId,
    UiInputEvent,
) {
    (toast.target, UiInputEvent::ToastTimer(toast))
}

/// 报告生成后补齐默认动作策略和步骤投影，交还拥有的回执。
fn with_toast_route_policy(
    surface: &UiSurface,
    mut result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let event = result.event.clone();
    annotate_route_policy(surface, &event, &mut result);
    annotate_result_route_steps(&mut result);
    result
}

#[cfg(test)]
#[path = "toast_timer/tests/owned_event_tests.rs"]
mod owned_event_tests;
