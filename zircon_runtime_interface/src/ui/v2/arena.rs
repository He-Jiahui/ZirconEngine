use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use toml::Value;

use crate::ui::layout::UiPixelSnappingPolicy;
use crate::ui::template::UiBindingRef;
use crate::ui::widget::UiWidgetContract;

use super::{UiV2Repeat, UiV2StyleDeclarationBlock};

/// One authored component invocation site in a node's instance ancestry.
/// The values point back to the source document and source node key; generated
/// arena ids and control ids are deliberately kept separate.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiTemplateNodeInstancePathStep {
    pub source_path: String,
    pub source_node_id: String,
}

/// 一次编译所得节点 arena 内的索引；不能把句柄当作跨文档或跨次编译的节点身份。
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_node_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance_path: Option<Vec<UiTemplateNodeInstancePathStep>>,
    pub component: String,
    #[serde(default)]
    pub control_id: Option<String>,
    #[serde(default)]
    pub pixel_snapping: UiPixelSnappingPolicy,
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(default)]
    pub props: BTreeMap<String, Value>,
    #[serde(default)]
    pub state: BTreeMap<String, Value>,
    #[serde(default)]
    pub layout: Option<BTreeMap<String, Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat: Option<UiV2Repeat>,
    #[serde(default)]
    pub style: UiV2StyleDeclarationBlock,
    #[serde(default)]
    pub slots: BTreeMap<String, Value>,
    #[serde(default)]
    pub events: Vec<UiBindingRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub widget: Option<UiWidgetContract>,
    #[serde(default)]
    pub children: Vec<UiV2ArenaChild>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiV2ArenaChild {
    pub child: UiV2NodeHandle,
    #[serde(default)]
    pub slot: BTreeMap<String, Value>,
}
