use serde::{Deserialize, Serialize};

use crate::core::framework::scene::{EntityId, WorldHandle};
use crate::core::math::Real;

use super::PhysicsTriggerEventKind;

/// 传感器配对变化事件；阶段由同一世界前后两次同步状态比较得出。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsTriggerEvent {
    pub world: WorldHandle,
    pub kind: PhysicsTriggerEventKind,
    pub trigger_entity: EntityId,
    pub other_entity: EntityId,
    pub point: [Real; 3],
}
