use std::sync::Arc;

use crate::asset::MeshAsset;
use crate::core::framework::render::RenderMeshBounds;

use super::super::GpuMeshResource;
use super::PreparedGeometryDeformation;
use crate::graphics::RuntimePrepareMeshSdfSeed;

/// 将同一网格资产修订的 CPU 几何、变形种子、保守边界和 GPU 缓冲保持在一个缓存项中；
/// 绘制提取只在修订仍有效时借用这组数据。
pub(in crate::graphics::scene::resources) struct PreparedMesh {
    pub(in crate::graphics::scene::resources) revision: u64,
    pub(in crate::graphics::scene::resources) local_bounds: RenderMeshBounds,
    pub(in crate::graphics::scene::resources) deformation: PreparedGeometryDeformation,
    pub(in crate::graphics::scene::resources) mesh_sdf: RuntimePrepareMeshSdfSeed,
    pub(in crate::graphics::scene::resources) asset: Arc<MeshAsset>,
    pub(in crate::graphics::scene::resources) resource: Arc<GpuMeshResource>,
}
