use serde::{Deserialize, Serialize};

use crate::asset::{AssetReference, MeshVertex};
use crate::core::framework::render::{
    RenderMeshBounds, RenderMeshDescriptor, RenderMeshKind, RenderMeshTopology,
};

use super::VirtualGeometryAsset;

pub const VIRTUAL_GEOMETRY_VERTEX_ORDINAL_LOW_JOINT_SLOT: usize = 0;
pub const VIRTUAL_GEOMETRY_VERTEX_ORDINAL_HIGH_JOINT_SLOT: usize = 1;
const VIRTUAL_GEOMETRY_VERTEX_ORDINAL_LOW_MASK: u32 = 0xffff;
const VIRTUAL_GEOMETRY_VERTEX_ORDINAL_HIGH_SHIFT: u32 = 16;

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
    pub fn assign_virtual_geometry_vertex_ordinals(&mut self) {
        if self.virtual_geometry.is_none() {
            return;
        }

        for (ordinal, vertex) in self.vertices.iter_mut().enumerate() {
            let [low, high] = Self::encode_virtual_geometry_vertex_ordinal(
                ordinal.try_into().unwrap_or(u32::MAX),
            );
            vertex.joint_indices[VIRTUAL_GEOMETRY_VERTEX_ORDINAL_LOW_JOINT_SLOT] = low;
            vertex.joint_indices[VIRTUAL_GEOMETRY_VERTEX_ORDINAL_HIGH_JOINT_SLOT] = high;
        }
    }

    pub fn encode_virtual_geometry_vertex_ordinal(ordinal: u32) -> [u16; 2] {
        [
            (ordinal & VIRTUAL_GEOMETRY_VERTEX_ORDINAL_LOW_MASK) as u16,
            (ordinal >> VIRTUAL_GEOMETRY_VERTEX_ORDINAL_HIGH_SHIFT) as u16,
        ]
    }

    pub fn decode_virtual_geometry_vertex_ordinal(joint_indices: [u16; 4]) -> u32 {
        u32::from(joint_indices[VIRTUAL_GEOMETRY_VERTEX_ORDINAL_LOW_JOINT_SLOT])
            | (u32::from(joint_indices[VIRTUAL_GEOMETRY_VERTEX_ORDINAL_HIGH_JOINT_SLOT])
                << VIRTUAL_GEOMETRY_VERTEX_ORDINAL_HIGH_SHIFT)
    }

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
