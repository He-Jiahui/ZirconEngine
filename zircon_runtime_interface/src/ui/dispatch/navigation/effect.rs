use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

// BUG: [CR-DISPATCH-0001] UiSurface 的默认 Next/方向导航只检查 focus_changed_to；
// 已返回 Handled 但未指定焦点时，默认导航仍可能移动焦点并覆盖 handled_by。
/// 导航处理器每次返回一个决议：Unhandled 继续冒泡，Handled 截止处理器链，
/// Focus 由 Runtime 校验目标存在，再由 UiSurface 校验可聚焦及输入所有权并提交。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiNavigationDispatchEffect {
    Unhandled,
    Handled,
    Focus(UiNodeId),
}

impl UiNavigationDispatchEffect {
    pub const fn handled() -> Self {
        Self::Handled
    }

    pub const fn focus(node_id: UiNodeId) -> Self {
        Self::Focus(node_id)
    }
}
