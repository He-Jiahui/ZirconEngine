use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ui::event_ui::{UiNodeId, UiTreeId};
use crate::ui::layout::UiSlot;

use super::UiTreeNode;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiTree {
    pub tree_id: UiTreeId,
    pub roots: Vec<UiNodeId>,
    pub nodes: BTreeMap<UiNodeId, UiTreeNode>,
    /// Parent-owned placement records for each retained parent-child edge.
    /// Older serialized trees omit this field, so deserialization defaults it empty.
    #[serde(default)]
    pub slots: Vec<UiSlot>,
}
