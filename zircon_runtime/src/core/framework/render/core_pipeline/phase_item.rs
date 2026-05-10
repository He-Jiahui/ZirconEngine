use crate::core::framework::scene::EntityId;

use super::{RenderPhase, RenderPhaseSortKey};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderPhaseMeshSource {
    MeshIndex(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderPhaseItem {
    pub entity: EntityId,
    pub phase: RenderPhase,
    pub sort_key: RenderPhaseSortKey,
    pub mesh_source: RenderPhaseMeshSource,
}
