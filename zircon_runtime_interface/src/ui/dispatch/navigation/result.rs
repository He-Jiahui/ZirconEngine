use serde::{Deserialize, Serialize};

use crate::ui::binding::UiBindingUpdateReport;
use crate::ui::event_ui::UiNodeId;
use crate::ui::surface::UiNavigationRoute;

use super::UiNavigationDispatchInvocation;

/// 导航路由的跨层结果：dispatcher 写入处理器决议，UiSurface 可补入默认导航目标
/// 及绑定报告，统一输入层再将处理结果转为 UiDispatchReply。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiNavigationDispatchResult {
    pub route: UiNavigationRoute,
    pub invocations: Vec<UiNavigationDispatchInvocation>,
    pub handled_by: Option<UiNodeId>,
    pub focus_changed_to: Option<UiNodeId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub binding_reports: Vec<UiBindingUpdateReport>,
}

impl UiNavigationDispatchResult {
    /// 为没有处理器命中的路由建立空结果，留给 UiSurface 执行默认导航。
    pub fn new(route: UiNavigationRoute) -> Self {
        Self {
            route,
            invocations: Vec::new(),
            handled_by: None,
            focus_changed_to: None,
            binding_reports: Vec::new(),
        }
    }

    // TODO: [CR-DISPATCH-0002] 尚未证实导航生产者会生成仅含回执的报告；
    // 当前过滤仍会丢弃只含 transaction、execution_receipt 或 dirty 的输入，需明确结果契约。
    /// 将有绑定更新的报告送入导航结果，供统一输入层继续传播。
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
