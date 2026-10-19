use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

/// 导航处理器每次返回一个决议：Unhandled 继续冒泡，Handled 截止处理器链，
/// Focus 由 Runtime 校验目标存在，再由 UiSurface 校验可聚焦及输入所有权并提交。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiNavigationDispatchEffect {
    Unhandled,
    Handled,
    Focus(UiNodeId),
}

impl UiNavigationDispatchEffect {
    /// 返回停止当前导航处理器链的 Handled 决议。
    pub const fn handled() -> Self {
        Self::Handled
    }

    /// 请求 Runtime/UiSurface 验证并提交目标焦点。
    pub const fn focus(node_id: UiNodeId) -> Self {
        Self::Focus(node_id)
    }
}
