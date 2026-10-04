use crate::core::math::{Vec3, Vec4};

use super::{append_ring, RING_VERTEX_CAPACITY};

#[test]
fn ring_capacity_matches_non_degenerate_output() {
    let mut vertices = Vec::new();
    append_ring(&mut vertices, Vec3::ZERO, Vec3::Z, 1.0, Vec4::ONE);

    assert_eq!(vertices.len(), RING_VERTEX_CAPACITY);
}
