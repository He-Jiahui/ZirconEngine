use serde::{Deserialize, Serialize};

use crate::core::framework::scene::WorldHandle;
use crate::core::math::{Real, Transform};

use super::{PhysicsColliderShape, PhysicsQueryFilter, PhysicsQueryMode};

/// 以世界空间起始姿态扫掠形状；退化方向、距离或形状在查询服务中返回空命中。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsShapeCastQuery {
    pub world: WorldHandle,
    pub shape: PhysicsColliderShape,
    pub origin_transform: Transform,
    pub direction: [Real; 3],
    pub max_distance: Real,
    #[serde(default)]
    pub mode: PhysicsQueryMode,
    pub filter: PhysicsQueryFilter,
}
