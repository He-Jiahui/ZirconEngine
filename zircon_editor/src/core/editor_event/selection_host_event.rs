use serde::{Deserialize, Serialize};
use zircon_runtime::scene::NodeId;

use crate::core::play::WorldDomain;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 宿主选择意图显式携带世界域，使编辑世界与 Play 世界的同名节点不会被混用。
pub enum SelectionHostEvent {
    SelectSceneNode {
        world_domain: WorldDomain,
        node_id: NodeId,
    },
}
