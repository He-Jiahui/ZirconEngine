use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use toml::Value;

use crate::ui::template::UiBindingRef;

use super::UiV2StyleDeclarationBlock;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UiV2NodeHandle(pub u32);

impl UiV2NodeHandle {
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiV2NodeArena {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<UiV2NodeHandle>,
    #[serde(default)]
    pub nodes: Vec<UiV2ArenaNode>,
}

impl UiV2NodeArena {
    pub fn node(&self, handle: UiV2NodeHandle) -> Option<&UiV2ArenaNode> {
        self.nodes.get(handle.index())
    }

    pub fn node_mut(&mut self, handle: UiV2NodeHandle) -> Option<&mut UiV2ArenaNode> {
        self.nodes.get_mut(handle.index())
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiV2ArenaNode {
    pub source_id: String,
    pub component: String,
    #[serde(default)]
    pub control_id: Option<String>,
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(default)]
    pub props: BTreeMap<String, Value>,
    #[serde(default)]
    pub state: BTreeMap<String, Value>,
    #[serde(default)]
    pub layout: Option<BTreeMap<String, Value>>,
    #[serde(default)]
    pub style: UiV2StyleDeclarationBlock,
    #[serde(default)]
    pub slots: BTreeMap<String, Value>,
    #[serde(default)]
    pub events: Vec<UiBindingRef>,
    #[serde(default)]
    pub children: Vec<UiV2ArenaChild>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiV2ArenaChild {
    pub child: UiV2NodeHandle,
    #[serde(default)]
    pub slot: BTreeMap<String, Value>,
}
