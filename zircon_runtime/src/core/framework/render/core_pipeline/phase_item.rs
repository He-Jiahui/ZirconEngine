use crate::core::framework::scene::EntityId;
use serde::{Deserialize, Serialize};

use super::{RenderPhase, RenderPhaseSortKey};

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
