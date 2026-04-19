use zircon_framework::render::OverlayBillboardIcon;
use zircon_math::Vec3;

use crate::scene::scene_renderer::primitives::IconVertex;

use super::super::size::icon_world_size;

pub(crate) fn build_icon_quad_vertices(
    icon: &OverlayBillboardIcon,
    right: Vec3,
    up: Vec3,
) -> [IconVertex; 6] {
    let half = icon_world_size(icon) * 0.5;
    let top_left = icon.position - right * half + up * half;
    let top_right = icon.position + right * half + up * half;
    let bottom_left = icon.position - right * half - up * half;
    let bottom_right = icon.position + right * half - up * half;
    [
        IconVertex::new(top_left, [0.0, 0.0], icon.tint),
        IconVertex::new(bottom_left, [0.0, 1.0], icon.tint),
        IconVertex::new(top_right, [1.0, 0.0], icon.tint),
        IconVertex::new(top_right, [1.0, 0.0], icon.tint),
        IconVertex::new(bottom_left, [0.0, 1.0], icon.tint),
        IconVertex::new(bottom_right, [1.0, 1.0], icon.tint),
    ]
}
