use serde::{Deserialize, Serialize};

use crate::core::framework::scene::EntityId;
use crate::core::math::Real;

use super::PhysicsJointType;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsJointSyncState {
    pub entity: EntityId,
    pub kind: PhysicsJointType,
    pub connected_entity: Option<EntityId>,
    pub anchor: [Real; 3],
    pub axis: [Real; 3],
    pub limits: Option<[Real; 2]>,
    pub collide_connected: bool,
}
