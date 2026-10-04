use serde::{Deserialize, Serialize};

use crate::core::framework::scene::physics::PhysicsMaterialMetadata;
use crate::core::framework::scene::EntityId;
use crate::core::math::Transform;

use super::PhysicsColliderShape;

/// 场景碰撞体的世界空间快照；实体标识把查询命中和事件关联回场景节点。
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
