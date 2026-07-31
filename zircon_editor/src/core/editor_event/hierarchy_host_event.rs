use serde::{Deserialize, Serialize};
use zircon_runtime::scene::NodeId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorHierarchyEvent {
    ReparentNodes {
        node_ids: Vec<NodeId>,
        parent: Option<NodeId>,
    },
    RenameNode {
        node_id: NodeId,
        name: String,
    },
}
