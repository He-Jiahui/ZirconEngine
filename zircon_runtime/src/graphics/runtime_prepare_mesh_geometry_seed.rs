use std::sync::Arc;

use crate::asset::{MeshSdfAsset, MeshSdfValidationError};
use crate::core::framework::render::RenderMeshBounds;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePrepareMeshSdfDeformationReason {
    ActiveMorphTargets,
    Skinning,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimePrepareMeshSdfSeed {
    Ready(Arc<[MeshSdfAsset]>),
    Missing {
        primitive_count: usize,
        payload_count: usize,
    },
    Invalid {
        primitive_index: usize,
        error: MeshSdfValidationError,
    },
    Deforming(RuntimePrepareMeshSdfDeformationReason),
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimePrepareMeshGeometrySeed {
    pub local_bounds: RenderMeshBounds,
    pub resource_revision: u64,
    pub shape_revision: u64,
    pub mesh_sdf: RuntimePrepareMeshSdfSeed,
}
