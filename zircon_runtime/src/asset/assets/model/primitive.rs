use serde::{Deserialize, Serialize};

use crate::asset::{AssetReference, MeshVertex};
use crate::core::framework::render::{
    RenderMeshBounds, RenderMeshDescriptor, RenderMeshKind, RenderMeshTopology,
};

use super::VirtualGeometryAsset;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelPrimitiveAsset {
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,
    /// Optional assetized mesh subasset that mirrors this primitive payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mesh: Option<AssetReference>,
    #[serde(default)]
    pub virtual_geometry: Option<VirtualGeometryAsset>,
}

impl ModelPrimitiveAsset {
    pub fn render_mesh_descriptor(&self) -> RenderMeshDescriptor {
        let bounds =
            RenderMeshBounds::from_positions(self.vertices.iter().map(|vertex| vertex.position));
        let is_planar = self.vertices.iter().all(|vertex| vertex.position[2] == 0.0);
        RenderMeshDescriptor {
            topology: RenderMeshTopology::TriangleList,
            bounds,
            primitive_kind: if is_planar {
                RenderMeshKind::Planar2d
            } else {
                RenderMeshKind::Spatial3d
            },
            suitable_for_2d: is_planar,
            suitable_for_3d: true,
            vertex_count: self.vertices.len(),
            index_count: self.indices.len(),
            primitive_count: self.indices.len() / 3,
            has_virtual_geometry_payload: self.virtual_geometry.is_some(),
        }
    }
}
