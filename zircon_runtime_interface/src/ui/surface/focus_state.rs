use serde::{Deserialize, Serialize};

use crate::ui::event_ui::{UiNodeId, UiNodePath};
use crate::ui::focus::{UiFocusChangeEvent, UiFocusVisible, UiFocusedInput};

/// 从布局快照投影出的焦点路径，分别保存根到叶和叶到根顺序，供帧观察者核对路由。
/// 布局尚未包含焦点节点时，focused 可以存在而两个路径为空；消费方须处理该状态。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiFocusPath {
    pub focused: Option<UiNodeId>,
    pub root_to_leaf: Vec<UiNodeId>,
    pub bubble_route: Vec<UiNodeId>,
}

impl UiFocusPath {
    pub fn with_route(
        focused: Option<UiNodeId>,
        root_to_leaf: Vec<UiNodeId>,
        bubble_route: Vec<UiNodeId>,
    ) -> Self {
        Self {
            focused,
            root_to_leaf,
            bubble_route,
        }
    }

    pub fn from_bubble_route(focused: Option<UiNodeId>, bubble_route: Vec<UiNodeId>) -> Self {
        let mut root_to_leaf = bubble_route.clone();
        root_to_leaf.reverse();
        Self::with_route(focused, root_to_leaf, bubble_route)
    }
}

/// 模态焦点恢复凭据。重建 UI 后若保存了稳定路径，Runtime 按路径重新查找目标；
/// 只有缺少路径时才尝试旧节点 ID。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiModalFocusRestoreState {
    pub modal: UiNodeId,
    #[serde(default)]
    pub modal_path: Option<UiNodePath>,
    pub restore: Option<UiNodeId>,
    #[serde(default)]
    pub restore_path: Option<UiNodePath>,
}

/// Surface 持有的焦点与指针交互状态；Runtime 输入路由维护它，并按帧发布给观察者。
/// captured、pressed 与 hovered 属于同一输入状态，不代表当前键盘焦点。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiFocusState {
    pub focused: Option<UiNodeId>,
    #[serde(default)]
    pub previous: Option<UiNodeId>,
    #[serde(default)]
    pub pending_autofocus: Option<UiNodeId>,
    #[serde(default)]
    pub focus_visible: UiFocusVisible,
    #[serde(default)]
    pub changes: Vec<UiFocusChangeEvent>,
    #[serde(default)]
    pub focused_inputs: Vec<UiFocusedInput>,
    #[serde(default)]
    pub modal_restore_stack: Vec<UiModalFocusRestoreState>,
    pub captured: Option<UiNodeId>,
    #[serde(default)]
    pub pressed: Option<UiNodeId>,
    pub hovered: Vec<UiNodeId>,
}
