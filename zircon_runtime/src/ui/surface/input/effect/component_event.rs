use zircon_runtime_interface::ui::{
    dispatch::{UiComponentEmissionPolicy, UiComponentEventReport, UiDispatchEffect},
    event_ui::UiNodeId,
};

use super::super::super::surface::UiSurface;
use super::super::{UiSurfaceInputEffectError, UiSurfaceInputEffectResult};
use super::node::require_node;

/// effect 阶段验证组件目标，实际事件作为派发结果交回宿主消费。
pub(super) fn apply_component_event_effect(
    surface: &UiSurface,
    effect: &UiDispatchEffect,
) -> UiSurfaceInputEffectResult<Option<UiNodeId>> {
    match effect {
        UiDispatchEffect::EmitComponentEvent { target, policy, .. } => {
            require_node(surface, *target)?;
            // TODO: [CR-UI-SURFACE-0003] 确认 Queue/Coalesce 是否应影响事件投递；
            // 三种策略目前都返回同一结果，报告也始终 delivered=true；缺少非 Immediate 调用与顺序测试。
            match policy {
                UiComponentEmissionPolicy::Immediate
                | UiComponentEmissionPolicy::Queue
                | UiComponentEmissionPolicy::Coalesce => Ok(Some(*target)),
            }
        }
        _ => Err(UiSurfaceInputEffectError::UnexpectedEffect {
            expected: "component event",
        }),
    }
}

pub(super) fn component_event_report_for_effect(
    effect: &UiDispatchEffect,
) -> Option<UiComponentEventReport> {
    let UiDispatchEffect::EmitComponentEvent { target, event, .. } = effect else {
        return None;
    };
    Some(UiComponentEventReport {
        target: *target,
        event: event.clone(),
        delivered: true,
        drag: None,
        template_action: None,
    })
}
