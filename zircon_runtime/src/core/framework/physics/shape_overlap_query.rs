use serde::{Deserialize, Serialize};

use crate::core::framework::scene::WorldHandle;
use crate::core::math::Transform;

use super::{PhysicsColliderShape, PhysicsQueryFilter, PhysicsQueryMode};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsShapeOverlapQuery {
    pub world: WorldHandle,
    pub shape: PhysicsColliderShape,
    pub transform: Transform,
    #[serde(default)]
    pub mode: PhysicsQueryMode,
    pub filter: PhysicsQueryFilter,
}
