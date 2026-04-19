use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::event_ui::{UiNodeId, UiTreeId};

use super::UiTreeNode;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiTree {
    pub tree_id: UiTreeId,
    pub roots: Vec<UiNodeId>,
    pub nodes: BTreeMap<UiNodeId, UiTreeNode>,
    pub(crate) next_paint_order: u64,
}
