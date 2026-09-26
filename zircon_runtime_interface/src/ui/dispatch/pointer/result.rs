use serde::{Deserialize, Serialize};

use crate::ui::binding::UiBindingUpdateReport;
use crate::ui::event_ui::UiNodeId;
use crate::ui::layout::UiFrame;
use crate::ui::surface::UiPointerRoute;
use crate::ui::tree::UiDirtyFlags;

use super::{UiPointerComponentEvent, UiPointerDispatchInvocation};

/// 汇总路由及 UiSurface 后续动作的观测值；pointer_routed 只表示存在目标或根候选，
/// 并不表示某个处理器已经接管事件。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiPointerDispatchDiagnostics {
    pub pointer_routed: bool,
    // TODO: [CR-DISPATCH-0006] Hover 无命中且进入、离开列表都为空时也会置位；
    // 需确认该字段表示无悬停变化，还是严格表示命中同一目标。
    pub ignored_same_target_hover: bool,
    pub hover_entered: usize,
    pub hover_left: usize,
    pub focus_changed: bool,
    pub capture_started: bool,
    pub capture_released: bool,
    pub click_target_resolved: bool,
    pub default_click_rejected: bool,
    pub component_event_count: usize,
    pub scroll_defaulted: bool,
}

/// 指针路由的跨层结果：dispatcher 收集处理器效果，UiSurface 应用捕获、焦点及
/// 组件默认行为，统一输入适配器再将处理决议投影为 UiDispatchReply。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPointerDispatchResult {
    pub route: UiPointerRoute,
    pub invocations: Vec<UiPointerDispatchInvocation>,
    pub handled_by: Option<UiNodeId>,
    pub blocked_by: Option<UiNodeId>,
    pub passthrough: Vec<UiNodeId>,
    pub captured_by: Option<UiNodeId>,
    #[serde(default)]
    pub released_capture: Option<UiNodeId>,
    #[serde(default)]
    pub focus_changed_to: Option<UiNodeId>,
    #[serde(default)]
    pub focus_cleared: bool,
    #[serde(default)]
    pub requested_dirty: UiDirtyFlags,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requested_damage: Vec<UiFrame>,
    #[serde(default)]
    pub diagnostics: UiPointerDispatchDiagnostics,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub component_events: Vec<UiPointerComponentEvent>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub binding_reports: Vec<UiBindingUpdateReport>,
}

impl UiPointerDispatchResult {
    /// 从已计算的命中路由建立基础诊断；UiSurface 随后补写焦点、捕获和组件事件统计。
    pub fn new(route: UiPointerRoute) -> Self {
        let diagnostics = UiPointerDispatchDiagnostics {
            pointer_routed: route.target.is_some() || !route.root_targets.is_empty(),
            ignored_same_target_hover: route.activation_phase
                == crate::ui::surface::UiPointerActivationPhase::Hover
                && route.entered.is_empty()
                && route.left.is_empty(),
            hover_entered: route.entered.len(),
            hover_left: route.left.len(),
            click_target_resolved: route.click_target.is_some(),
            ..UiPointerDispatchDiagnostics::default()
        };
        Self {
            route,
            invocations: Vec::new(),
            handled_by: None,
            blocked_by: None,
            passthrough: Vec::new(),
            captured_by: None,
            released_capture: None,
            focus_changed_to: None,
            focus_cleared: false,
            requested_dirty: UiDirtyFlags::default(),
            requested_damage: Vec::new(),
            diagnostics,
            component_events: Vec::new(),
            binding_reports: Vec::new(),
        }
    }

    // BUG: [CR-DISPATCH-0003] 全部 Optional 目标缺值时可生成零更新但带事务及执行回执的报告；
    // 当前判空忽略这些回执，导致该次绑定执行记录丢失。
    /// 将组件事件触发的绑定报告送往统一输入结果，避免无意义的空更新占用结果。
    pub fn record_binding_report(&mut self, report: UiBindingUpdateReport) {
        if !report.updates.is_empty()
            || report.applied_count > 0
            || report.unchanged_count > 0
            || report.rejected_count > 0
        {
            self.binding_reports.push(report);
        }
    }
}
