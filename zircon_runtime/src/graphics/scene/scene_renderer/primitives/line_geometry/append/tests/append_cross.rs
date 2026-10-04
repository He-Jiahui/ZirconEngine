use crate::core::math::{Vec3, Vec4};

use super::{append_cross, CROSS_VERTEX_CAPACITY};

#[test]
fn cross_capacity_matches_output() {
    let mut vertices = Vec::new();
    append_cross(&mut vertices, Vec3::ZERO, 1.0, Vec4::ONE, Vec3::X, Vec3::Y);

    assert_eq!(vertices.len(), CROSS_VERTEX_CAPACITY);
}
