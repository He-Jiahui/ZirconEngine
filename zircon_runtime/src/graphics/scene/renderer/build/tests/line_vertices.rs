use crate::core::framework::render::{
    OverlayBillboardIcon, OverlayLineSegment, OverlayWireShape, SceneGizmoKind,
    SceneGizmoOverlayExtract, ViewportIconId,
};
use crate::core::math::{Vec3, Vec4};

use super::scene_gizmo_line_vertex_capacity;

#[test]
fn scene_gizmo_line_capacity_counts_only_missing_icon_fallbacks() {
    let gizmos = [SceneGizmoOverlayExtract::new(
        1,
        SceneGizmoKind::Camera,
        false,
        vec![OverlayLineSegment {
            start: Vec3::ZERO,
            end: Vec3::X,
            color: Vec4::ONE,
        }],
        vec![OverlayWireShape::Arrow {
            origin: Vec3::ZERO,
            direction: Vec3::X,
            length: 1.0,
            color: Vec4::ONE,
        }],
        vec![
            OverlayBillboardIcon {
                id: ViewportIconId::Camera,
                position: Vec3::ZERO,
                tint: Vec4::ONE,
                size: 1.0,
            },
            OverlayBillboardIcon {
                id: ViewportIconId::DirectionalLight,
                position: Vec3::ONE,
                tint: Vec4::ONE,
                size: 1.0,
            },
        ],
        Vec::new(),
    )];

    assert_eq!(scene_gizmo_line_vertex_capacity(&gizmos, &|_| false), 28);
    assert_eq!(
        scene_gizmo_line_vertex_capacity(&gizmos, &|id| id == ViewportIconId::Camera),
        16
    );
    assert_eq!(scene_gizmo_line_vertex_capacity(&gizmos, &|_| true), 8);
}
