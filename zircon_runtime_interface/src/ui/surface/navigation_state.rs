use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

/// 表面导航快照：焦点变更同步导航起点与焦点可见性，事件路由在无显式焦点时读取该起点。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiNavigationState {
    pub focus_visible: bool,
    pub navigation_root: Option<UiNodeId>,
}
