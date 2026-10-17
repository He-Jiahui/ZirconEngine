//! Surface 发布和维护的焦点状态、模态恢复凭据及布局路径快照。
//!
//! 集合与稳定路径均由值本身拥有，不借用当前树；保存节点标识不会延长节点生命期。
//! 路径构造器只整理调用方提供的数据，一致性和节点有效性由生产这些快照的 Runtime 检查。

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
    /// 接收两种已排好的路径并构造快照，不重新查树或调整顺序。
    ///
    /// # 参数与返回
    /// - `focused`：观察到的焦点标识，可为空。
    /// - `root_to_leaf`、`bubble_route`：分别为根到叶和冒泡顺序，两个数组均移交所有权。
    /// 返回拥有这些数据的快照；不校验数组互为反序、端点与焦点一致或节点仍然存活。
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

    /// 从冒泡顺序构造双向快照：保留传入数组，再克隆并反转得到根到叶顺序。
    /// 焦点标识及原数组移入返回值；不会验证冒泡顺序或节点关系，也不保留树的借用。
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
///
/// 焦点变化、已处理输入和模态恢复栈均由此值拥有；快照的标识可能晚于树更新而失效。
/// 派生默认值用于显式构造；反序列化仅对可缺省字段补值，未标记的集合仍要求输入提供。
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
