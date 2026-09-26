use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

use super::UiNavigationDispatchEffect;

/// 记录第一个接管导航路由的处理器决议，供派发结果追溯；
/// Unhandled 不留记录，焦点变更仍由 UiSurface 在派发后提交。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiNavigationDispatchInvocation {
    pub node_id: UiNodeId,
    pub effect: UiNavigationDispatchEffect,
}
