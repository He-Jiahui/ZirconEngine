use serde::{Deserialize, Serialize};

use crate::core::framework::scene::{EntityId, WorldHandle};
use crate::core::math::Real;

/// 物理更新阶段产生的接触载荷；固定更新阶段按世界和实体标识发布给场景。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsContactEvent {
    pub world: WorldHandle,
    pub entity: EntityId,
    pub other_entity: EntityId,
    pub point: [Real; 3],
    pub normal: [Real; 3],
}
