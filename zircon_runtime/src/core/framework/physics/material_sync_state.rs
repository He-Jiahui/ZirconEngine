use serde::{Deserialize, Serialize};

use crate::core::framework::scene::physics::PhysicsMaterialMetadata;
use crate::core::framework::scene::EntityId;

// TODO: [CR-PHYSICS-FRAMEWORK-0002] 确认独立材质列表的消费契约；目前仅构建和净化该列表，后端从碰撞体字段读取材质；下一步核对是否保留该载荷。
/// 与碰撞体同实体保存材质资源定位符和覆盖值；同步阶段会移除失去对应碰撞体的记录。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsMaterialSyncState {
    pub entity: EntityId,
    pub locator: Option<String>,
    pub material: PhysicsMaterialMetadata,
}
