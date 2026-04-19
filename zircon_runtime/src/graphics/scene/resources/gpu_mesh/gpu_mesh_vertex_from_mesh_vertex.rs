use crate::asset::pipeline::types::MeshVertex;

use super::gpu_mesh_vertex::GpuMeshVertex;

impl From<MeshVertex> for GpuMeshVertex {
    fn from(value: MeshVertex) -> Self {
        Self {
            position: value.position,
            normal: value.normal,
            uv: value.uv,
        }
    }
}
