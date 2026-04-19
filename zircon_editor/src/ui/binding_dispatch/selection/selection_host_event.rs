use serde::{Deserialize, Serialize};
use zircon_runtime::scene::NodeId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionHostEvent {
    SelectSceneNode { node_id: NodeId },
}
