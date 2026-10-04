use crate::core::framework::scene::EntityId;
use serde::{Deserialize, Serialize};

use super::{RenderPhase, RenderPhaseQueueOrderingKey, RenderPhaseSortKey};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// 队列项回指同次提取中的网格或精灵数组；索引不能跨帧或跨快照复用。
pub enum RenderPhaseMeshSource {
    MeshIndex(usize),
    SpriteIndex(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RenderPhaseItem {
    pub entity: EntityId,
    pub phase: RenderPhase,
    pub sort_key: RenderPhaseSortKey,
    pub mesh_source: RenderPhaseMeshSource,
}

impl RenderPhaseItem {
    pub const fn ordering_key(&self) -> RenderPhaseQueueOrderingKey {
        RenderPhaseQueueOrderingKey::new(self.phase, self.sort_key, self.entity)
    }
}
