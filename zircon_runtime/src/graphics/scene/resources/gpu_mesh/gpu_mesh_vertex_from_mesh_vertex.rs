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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::math::{Vec2, Vec3};

    #[test]
    fn gpu_mesh_vertex_conversion_preserves_skinning_channels() {
        let source = MeshVertex {
            position: Vec3::new(1.0, 2.0, 3.0).to_array(),
            normal: Vec3::new(0.0, 1.0, 0.0).to_array(),
            uv: Vec2::new(0.25, 0.75).to_array(),
            joint_indices: [2, 4, 6, 8],
            joint_weights: [0.4, 0.3, 0.2, 0.1],
        };

        let gpu = GpuMeshVertex::from(source);

        assert_eq!(gpu.position, [1.0, 2.0, 3.0]);
        assert_eq!(gpu.normal, [0.0, 1.0, 0.0]);
        assert_eq!(gpu.uv, [0.25, 0.75]);
        assert_eq!(gpu.joint_indices, [2, 4, 6, 8]);
        assert_eq!(gpu.joint_weights, [0.4, 0.3, 0.2, 0.1]);
    }
}
