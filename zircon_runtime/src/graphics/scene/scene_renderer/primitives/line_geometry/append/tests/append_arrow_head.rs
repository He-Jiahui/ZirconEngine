use crate::core::math::{Vec3, Vec4};

use super::{append_arrow_head, ARROW_HEAD_VERTEX_CAPACITY};

#[test]
fn arrow_head_capacity_matches_non_degenerate_output() {
    let mut vertices = Vec::new();
    append_arrow_head(&mut vertices, Vec3::ZERO, Vec3::X, Vec4::ONE);

    assert_eq!(vertices.len(), ARROW_HEAD_VERTEX_CAPACITY);
}
