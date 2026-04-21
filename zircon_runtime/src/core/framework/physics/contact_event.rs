use serde::{Deserialize, Serialize};

use crate::core::framework::scene::{EntityId, WorldHandle};
use crate::core::math::Real;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsContactEvent {
    pub world: WorldHandle,
    pub entity: EntityId,
    pub other_entity: EntityId,
    pub point: [Real; 3],
    pub normal: [Real; 3],
}
