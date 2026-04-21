use serde::{Deserialize, Serialize};

use crate::core::framework::scene::EntityId;
use crate::core::math::Transform;

use super::{PhysicsColliderShape, PhysicsMaterialMetadata};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsColliderSyncState {
    pub entity: EntityId,
    pub shape: PhysicsColliderShape,
    pub sensor: bool,
    pub layer: u32,
    pub collision_group: u32,
    pub collision_mask: u32,
    pub material: Option<String>,
    pub material_override: Option<PhysicsMaterialMetadata>,
    pub transform: Transform,
}
