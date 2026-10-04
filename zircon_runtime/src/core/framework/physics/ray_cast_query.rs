use serde::{Deserialize, Serialize};

use crate::core::framework::scene::WorldHandle;
use crate::core::math::Real;

use super::{PhysicsQueryFilter, PhysicsQueryMode};

/// 面向已同步世界的射线查询；调用方须提供有限非零方向和正距离，无效输入视为空命中。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsRayCastQuery {
    pub world: WorldHandle,
    pub origin: [Real; 3],
    pub direction: [Real; 3],
    pub max_distance: Real,
    #[serde(default)]
    pub mode: PhysicsQueryMode,
    pub filter: PhysicsQueryFilter,
}
