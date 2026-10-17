//! 将子菜单悬停到期通知转换为当前目标的就绪报告。
//! 输入管理器拥有计时状态；本层核对目标可用性，并把选项标识随事件保留在回执中。

use zircon_runtime_interface::ui::dispatch::{
    UiDispatchPhase, UiDispatchReply, UiInputDispatchResult, UiInputEvent,
    UiSubmenuHoverTimerInputEvent,
};

use super::super::surface::UiSurface;
use super::{route_policy::annotate_route_policy, route_steps::annotate_result_route_steps};

/// 消费输入管理器已到期的菜单悬停通知，再核对目标仍支持菜单交互。
/// 本入口只发出语义组件事件，菜单模型负责实际打开对应子菜单。
///
/// # 参数与返回
/// `surface` 提供当前目标和绑定；`submenu_hover` 移交拥有的通知，回执继续拥有其中的选项标识。
/// # 调用时机与约束
/// 统一输入入口收到悬停到期通知时调用；空选项或失效目标返回未处理回执。
/// 就绪报告按当前目标生成，绑定查询错误折叠为空报告。
/// # 参考
/// 到期队列由 [`UiInputManager::tick`](crate::ui::dispatch::UiInputManager::tick) 消费。
pub(super) fn dispatch_submenu_hover_timer_input(
    surface: &mut UiSurface,
    submenu_hover: UiSubmenuHoverTimerInputEvent,
) -> UiInputDispatchResult {
    let supports_submenu_hover = !submenu_hover.option_id.is_empty()
        && surface
            .submenu_hover_delay_ms_for_component_node(submenu_hover.target)
            .is_some();
    if !supports_submenu_hover {
        let (_, event) = into_submenu_hover_timer_event(submenu_hover);
        let mut result = UiInputDispatchResult::new(event, UiDispatchReply::unhandled());
        result
            .diagnostics
            .notes
            .push("stale_submenu_hover_timer_ignored".to_string());
        result.diagnostics.handled_phase = Some("submenu_hover_timer.stale".to_string());
        return with_submenu_hover_route_policy(surface, result);
    }

    let component_events = surface
        .apply_default_submenu_hover_ready_component_event(submenu_hover.target)
        .unwrap_or_default();
    let (target, event) = into_submenu_hover_timer_event(submenu_hover);
    let reply = if component_events.is_empty() {
        UiDispatchReply::unhandled()
    } else {
        UiDispatchReply::handled()
            .from_handler(target)
            .in_phase(UiDispatchPhase::DefaultAction)
    };
    let mut result = UiInputDispatchResult::new(event, reply);
    result.diagnostics.routed = true;
    result.diagnostics.route_target = Some(target);
    result.diagnostics.handled_phase = Some(if component_events.is_empty() {
        "submenu_hover_timer.no_binding".to_string()
    } else {
        "submenu_hover_timer.component_event".to_string()
    });
    result.component_events = component_events;
    with_submenu_hover_route_policy(surface, result)
}

/// 先取出可复制的目标，再将整份通知移入事件，使选项字符串的原分配随事件转移。
fn into_submenu_hover_timer_event(
    submenu_hover: UiSubmenuHoverTimerInputEvent,
) -> (
    zircon_runtime_interface::ui::event_ui::UiNodeId,
    UiInputEvent,
) {
    (
        submenu_hover.target,
        UiInputEvent::SubmenuHoverTimer(submenu_hover),
    )
}

/// 按当前表面补齐默认动作策略和步骤投影，保留已有组件报告。
fn with_submenu_hover_route_policy(
    surface: &UiSurface,
    mut result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let event = result.event.clone();
    annotate_route_policy(surface, &event, &mut result);
    annotate_result_route_steps(&mut result);
    result
}

#[cfg(test)]
#[path = "submenu_hover_timer/tests/owned_event.rs"]
mod owned_event_tests;
