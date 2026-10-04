use serde::{Deserialize, Serialize};

use crate::core::framework::scene::WorldHandle;
use crate::core::math::Transform;

use super::{PhysicsColliderShape, PhysicsQueryFilter, PhysicsQueryMode};

/// 检查世界空间形状与已同步碰撞体的重叠；与射线查询共用筛选条件。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsShapeOverlapQuery {
    pub world: WorldHandle,
    pub shape: PhysicsColliderShape,
    pub transform: Transform,
    #[serde(default)]
    pub mode: PhysicsQueryMode,
    pub filter: PhysicsQueryFilter,
}
