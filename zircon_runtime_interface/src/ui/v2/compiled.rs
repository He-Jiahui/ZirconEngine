use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{UiV2ComponentGraph, UiV2NodeArena, UiV2NodeHandle};

/// 同批编译的节点 arena、源 ID 索引和拓扑投影共享一套句柄，不可与其他编译结果拼接。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiV2CompiledDocument {
    pub asset_id: String,
    pub arena: UiV2NodeArena,
    pub node_handles: BTreeMap<String, UiV2NodeHandle>,
    pub component_graph: UiV2ComponentGraph,
}
