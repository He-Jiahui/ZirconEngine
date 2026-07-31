use std::collections::HashSet;

use crate::core::math::Vec3;

use super::gpu_mesh_vertex::GpuMeshVertex;

pub(super) fn build_wire_segments(vertices: &[GpuMeshVertex], indices: &[u32]) -> Vec<[Vec3; 2]> {
    let mut unique_edges = HashSet::new();
    let mut segments = Vec::new();

    for triangle in indices.chunks_exact(3) {
        for (a, b) in [
            (triangle[0], triangle[1]),
            (triangle[1], triangle[2]),
            (triangle[2], triangle[0]),
        ] {
            let (lo, hi) = if a < b { (a, b) } else { (b, a) };
            if !unique_edges.insert((lo, hi)) {
                continue;
            }
            let start = vertices
                .get(lo as usize)
                .map(|vertex| Vec3::from_array(vertex.position))
                .unwrap_or(Vec3::ZERO);
            let end = vertices
                .get(hi as usize)
                .map(|vertex| Vec3::from_array(vertex.position))
                .unwrap_or(Vec3::ZERO);
            segments.push([start, end]);
        }
    }

    segments
}
