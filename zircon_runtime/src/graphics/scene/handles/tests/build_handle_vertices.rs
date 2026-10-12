use crate::core::framework::render::{HandleElementExtract, OverlayAxis};
use crate::core::math::{Vec3, Vec4};

use super::handle_element_vertex_capacity;

#[test]
fn handle_capacity_matches_non_degenerate_element_topology() {
    let elements = [
        HandleElementExtract::AxisLine {
            axis: OverlayAxis::X,
            start: Vec3::ZERO,
            end: Vec3::X,
            color: Vec4::ONE,
            pick_radius: 1.0,
        },
        HandleElementExtract::AxisRing {
            axis: OverlayAxis::Y,
            center: Vec3::ZERO,
            normal: Vec3::Y,
            radius: 1.0,
            color: Vec4::ONE,
            pick_radius: 1.0,
        },
        HandleElementExtract::AxisScale {
            axis: OverlayAxis::Z,
            start: Vec3::ZERO,
            end: Vec3::Z,
            color: Vec4::ONE,
            pick_radius: 1.0,
            handle_size: 1.0,
        },
        HandleElementExtract::CenterAnchor {
            position: Vec3::ZERO,
            size: 1.0,
            color: Vec4::ONE,
        },
    ];

    let capacities = elements.map(|element| handle_element_vertex_capacity(&element));

    assert_eq!(capacities, [6, 96, 6, 4]);
}
