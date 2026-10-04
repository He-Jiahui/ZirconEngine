use std::sync::Arc;

use crate::asset::ModelAsset;
use crate::core::framework::render::RenderMeshBounds;

use super::super::resource_streamer::model_geometry_resolution::ModelMeshDependencyState;
use super::super::GpuModelResource;
use super::PreparedGeometryDeformation;
use crate::graphics::RuntimePrepareMeshSdfSeed;

/// source_revision 跟踪模型资产；revision 是结合 mesh 依赖版本与状态的几何指纹，依赖变化时整组几何须重新发布。
pub(in crate::graphics::scene::resources) struct PreparedModel {
    pub(in crate::graphics::scene::resources) revision: u64,
    pub(in crate::graphics::scene::resources) source_revision: u64,
    pub(in crate::graphics::scene::resources) mesh_dependency_states: Vec<ModelMeshDependencyState>,
    pub(in crate::graphics::scene::resources) local_bounds: RenderMeshBounds,
    pub(in crate::graphics::scene::resources) deformation: PreparedGeometryDeformation,
    pub(in crate::graphics::scene::resources) mesh_sdf: RuntimePrepareMeshSdfSeed,
    pub(in crate::graphics::scene::resources) asset: Arc<ModelAsset>,
    pub(in crate::graphics::scene::resources) resource: Arc<GpuModelResource>,
}
