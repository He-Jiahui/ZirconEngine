use std::collections::HashSet;

use crate::core::math::Vec3;

use super::gpu_mesh_vertex::GpuMeshVertex;

/// 由上传用三角索引提取去重边，供线框调试渲染复用设备网格的几何身份。
/// 此结果只用于可视化边线，不能代替网格拓扑校验。
pub(super) fn build_wire_segments(vertices: &[GpuMeshVertex], indices: &[u32]) -> Vec<[Vec3; 2]> {
    let mut unique_edges = HashSet::with_capacity(indices.len());
    let mut segments = Vec::with_capacity(indices.len());

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

#[cfg(test)]
#[path = "wire_segments/tests/capacity_tests.rs"]
mod capacity_tests;
