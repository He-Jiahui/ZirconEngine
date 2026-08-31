use serde::{Deserialize, Serialize};
use zircon_runtime::scene::NodeId;

use crate::core::play::WorldDomain;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionHostEvent {
    SelectSceneNode {
        world_domain: WorldDomain,
        node_id: NodeId,
    },
}
