use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{UiV2ComponentGraph, UiV2NodeArena, UiV2NodeHandle};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiV2CompiledDocument {
    pub asset_id: String,
    pub arena: UiV2NodeArena,
    pub node_handles: BTreeMap<String, UiV2NodeHandle>,
    pub component_graph: UiV2ComponentGraph,
}
