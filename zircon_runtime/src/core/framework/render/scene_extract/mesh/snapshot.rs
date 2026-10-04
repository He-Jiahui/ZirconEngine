use crate::core::framework::scene::{EntityId, Mobility};
use crate::core::math::{Real, Transform, Vec4};
use crate::core::resource::{MaterialMarker, MeshMarker, ModelMarker, ResourceHandle};

use super::super::super::RendererCommon;
use super::{RenderMeshLodSelection, RenderMeshStaticState};

/// 场景生产者交给一帧渲染的网格实例；同一实体的多个 primitive 以稳定键区分。
/// transform_revision 与资源修订共同决定跨帧缓存是否可复用，不能只凭实体 ID 推断。
#[derive(Clone, Debug, PartialEq)]
pub struct RenderMeshSnapshot {
    pub node_id: EntityId,
    pub stable_instance_key: u64,
    pub transform_revision: u64,
    pub transform: Transform,
    pub model: ResourceHandle<ModelMarker>,
    pub mesh: Option<ResourceHandle<MeshMarker>>,
    pub material: ResourceHandle<MaterialMarker>,
    pub mesh_lod: Option<RenderMeshLodSelection>,
    pub morph_weights: Vec<Real>,
    pub tint: Vec4,
    pub mobility: Mobility,
    pub static_state: RenderMeshStaticState,
    pub common: RendererCommon,
}
