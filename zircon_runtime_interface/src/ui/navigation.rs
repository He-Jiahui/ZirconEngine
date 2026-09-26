use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UiNavigationGroupId(pub String);

impl UiNavigationGroupId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

/// 显式 Tab 顺序及准入；缺省反序列化的 `tabbable` 为 false，`new` 才启用准入。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiTabIndex {
    pub order: i32,
    pub tabbable: bool,
}

impl UiTabIndex {
    pub const fn new(order: i32) -> Self {
        Self {
            order,
            tabbable: true,
        }
    }

    pub const fn disabled() -> Self {
        Self {
            order: 0,
            tabbable: false,
        }
    }
}

/// 为导航索引声明分组、模态根及排序；焦点候选资格仍由节点状态决定。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiNavigationGroup {
    pub group_id: UiNavigationGroupId,
    pub parent: Option<UiNavigationGroupId>,
    pub root: Option<UiNodeId>,
    pub modal: bool,
    pub wrap: bool,
    pub order: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiDirectionalNavigation {
    pub up: UiDirectionalNavigationTarget,
    pub down: UiDirectionalNavigationTarget,
    pub left: UiDirectionalNavigationTarget,
    pub right: UiDirectionalNavigationTarget,
}

// TODO: [CR-UINAV-0001] 确认 boundary 的运行时消费边界；当前只见序列化和测试读取，导航索引按分组与模态状态选目标；下一步补各边界值的派发契约测试。
/// 声明导航边界处的意图；具体执行需由运行时导航路径解释。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "target")]
pub enum UiNavigationBoundary {
    #[default]
    Escape,
    Wrap,
    Stop,
    Explicit(UiNodeId),
    Trap,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "target")]
pub enum UiDirectionalNavigationTarget {
    #[default]
    Auto,
    Node(UiNodeId),
    Group(UiNavigationGroupId),
    Blocked,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiNavigationContract {
    pub tab_index: Option<UiTabIndex>,
    pub group: Option<UiNavigationGroup>,
    pub directional: Option<UiDirectionalNavigation>,
    #[serde(default)]
    pub boundary: UiNavigationBoundary,
}
