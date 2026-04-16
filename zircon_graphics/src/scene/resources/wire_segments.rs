use std::collections::HashSet;

use zircon_math::Vec3;

pub(super) fn build_wire_segments(positions: &[Vec3], indices: &[u32]) -> Vec<[Vec3; 2]> {
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
            let start = positions.get(lo as usize).copied().unwrap_or(Vec3::ZERO);
            let end = positions.get(hi as usize).copied().unwrap_or(Vec3::ZERO);
            segments.push([start, end]);
        }
    }

    segments
}
