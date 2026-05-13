use serde::{Deserialize, Serialize};

use super::UiV2NodeHandle;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiV2ComponentGraph {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<UiV2NodeHandle>,
    #[serde(default)]
    pub nodes: Vec<UiV2ComponentGraphNode>,
}

impl UiV2ComponentGraph {
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiV2ComponentGraphNode {
    pub handle: UiV2NodeHandle,
    pub source_id: String,
    pub component: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<UiV2NodeHandle>,
    #[serde(default)]
    pub children: Vec<UiV2NodeHandle>,
}
