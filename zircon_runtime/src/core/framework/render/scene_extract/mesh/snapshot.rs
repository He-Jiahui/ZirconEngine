use crate::core::framework::scene::{EntityId, Mobility};
use crate::core::math::{Real, Transform, Vec4};
use crate::core::resource::{MaterialMarker, MeshMarker, ModelMarker, ResourceHandle};

use super::super::super::RendererCommon;
use super::{RenderMeshLodSelection, RenderMeshStaticState};

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
