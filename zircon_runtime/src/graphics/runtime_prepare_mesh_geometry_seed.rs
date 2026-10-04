//! 网格准备向运行时 provider 传递几何身份及 SDF 可用性，变形网格保留拒绝原因以防误用静态数据。
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

/// 供准备收集器判断网格静态几何可用性的只读种子。
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimePrepareMeshGeometrySeed {
    pub local_bounds: RenderMeshBounds,
    pub resource_revision: u64,
    pub shape_revision: u64,
    pub mesh_sdf: RuntimePrepareMeshSdfSeed,
}
