use crate::asset::pipeline::types::MeshVertex;

use super::gpu_mesh_vertex::GpuMeshVertex;

impl From<MeshVertex> for GpuMeshVertex {
    fn from(value: MeshVertex) -> Self {
        Self {
            position: value.position,
            normal: value.normal,
            uv: value.uv,
            joint_indices: value.joint_indices,
            joint_weights: value.joint_weights,
            tangent: value.tangent,
            color: value.color,
            uv1: value.uv1,
        }
    }
}

#[cfg(test)]
#[path = "tests/gpu_mesh_vertex_from_mesh_vertex.rs"]
mod tests;
