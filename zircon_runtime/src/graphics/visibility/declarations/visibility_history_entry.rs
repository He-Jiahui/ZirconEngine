use crate::core::framework::scene::EntityId;

use super::{visibility_batch_key::VisibilityBatchKey, visibility_bounds::VisibilityBounds};

/// 上帧实例的身份、批次键与包围数据，用于决定本帧 BVH 的增删改。
#[derive(Clone, Debug, PartialEq)]
pub struct VisibilityHistoryEntry {
    pub entity: EntityId,
    pub stable_instance_key: u64,
    pub key: VisibilityBatchKey,
    pub bounds: VisibilityBounds,
}
