use crate::core::framework::scene::EntityId;

use super::{visibility_batch_key::VisibilityBatchKey, visibility_bounds::VisibilityBounds};

/// 空间索引中的一份渲染实例快照，以稳定实例键对齐历史与本帧更新计划。
#[derive(Clone, Debug, PartialEq)]
pub struct VisibilityBvhInstance {
    pub entity: EntityId,
    pub stable_instance_key: u64,
    pub key: VisibilityBatchKey,
    pub bounds: VisibilityBounds,
}
