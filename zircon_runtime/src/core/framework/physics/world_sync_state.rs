use serde::{Deserialize, Serialize};

use crate::core::framework::scene::WorldHandle;

use super::{
    PhysicsBodySyncState, PhysicsColliderSyncState, PhysicsJointSyncState, PhysicsMaterialSyncState,
};

/// 单个世界的完整物理投影；场景生成后交给插件同步，查询和调试显示读取其快照。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsWorldSyncState {
    pub world: WorldHandle,
    pub bodies: Vec<PhysicsBodySyncState>,
    pub colliders: Vec<PhysicsColliderSyncState>,
    pub joints: Vec<PhysicsJointSyncState>,
    pub materials: Vec<PhysicsMaterialSyncState>,
}

// 默认句柄只服务空载荷构造；提交同步前由调用方填写实际世界句柄。
impl Default for PhysicsWorldSyncState {
    fn default() -> Self {
        Self {
            world: WorldHandle::new(0),
            bodies: Vec::new(),
            colliders: Vec::new(),
            joints: Vec::new(),
            materials: Vec::new(),
        }
    }
}
