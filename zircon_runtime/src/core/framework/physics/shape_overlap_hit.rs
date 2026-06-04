use serde::{Deserialize, Serialize};

use crate::core::framework::scene::EntityId;
use crate::core::math::Transform;

use super::PhysicsColliderShape;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsShapeOverlapHit {
    pub entity: EntityId,
    pub shape: PhysicsColliderShape,
    pub transform: Transform,
    pub sensor: bool,
    pub layer: u32,
    pub collision_group: u32,
}
