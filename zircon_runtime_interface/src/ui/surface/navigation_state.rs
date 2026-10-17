use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

/// 表面导航快照：焦点变更同步导航起点与焦点可见性，事件路由在无显式焦点时读取该起点。
/// 导航根和焦点环状态的自有快照；节点标识不构成对树的借用或有效性保证。
/// 构造默认值不会寻找导航根，反序列化也不执行焦点或路由更新。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiNavigationState {
    pub focus_visible: bool,
    pub navigation_root: Option<UiNodeId>,
}
