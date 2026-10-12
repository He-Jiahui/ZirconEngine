use crate::core::framework::render::{OverlayBillboardIcon, ViewportIconId};
use crate::core::math::{Vec3, Vec4};

use super::{append_icon_fallback_lines, icon_fallback_vertex_capacity};

#[test]
fn scene_gizmo_line_capacity_matches_icon_fallbacks() {
    let icons = [
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
    ];

    for icon in icons {
        let mut vertices = Vec::new();
        append_icon_fallback_lines(&mut vertices, &icon, Vec3::X, Vec3::Y);

        assert_eq!(vertices.len(), icon_fallback_vertex_capacity(&icon));
    }
}
