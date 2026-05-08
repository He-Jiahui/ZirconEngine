use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UiNavigationGroupId(pub String);

impl UiNavigationGroupId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiTabIndex {
    pub order: i32,
    pub tabbable: bool,
}

impl Default for UiTabIndex {
    fn default() -> Self {
        Self {
            order: 0,
            tabbable: false,
        }
    }
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
}
