use crate::core::framework::scene::EntityId;
use serde::{Deserialize, Serialize};

use super::{RenderPhase, RenderPhaseQueueOrderingKey, RenderPhaseSortKey};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
