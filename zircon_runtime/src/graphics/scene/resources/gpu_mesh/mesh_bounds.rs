use crate::core::math::Vec3;

use super::gpu_mesh_vertex::GpuMeshVertex;

pub(super) fn mesh_bounds(vertices: &[GpuMeshVertex]) -> (Vec3, Vec3) {
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for vertex in vertices {
        let position = Vec3::from_array(vertex.position);
        min = min.min(position);
        max = max.max(position);
    }
    if !min.is_finite() || !max.is_finite() {
        (Vec3::ZERO, Vec3::ZERO)
    } else {
        (min, max)
    }
}
