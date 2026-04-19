use crate::core::framework::render::OverlayBillboardIcon;
use crate::core::math::Vec3;

use crate::graphics::scene::scene_renderer::primitives::LineVertex;

pub(in crate::graphics::scene::scene_renderer::primitives::scene_gizmo::append) fn append_camera_icon_fallback_lines(
    vertices: &mut Vec<LineVertex>,
    icon: &OverlayBillboardIcon,
    right: Vec3,
    up: Vec3,
    size: f32,
) {
    let half = size * 0.5;
    let left = icon.position - right * half;
    let right_pt = icon.position + right * half;
    let top = icon.position + up * half;
    let bottom = icon.position - up * half;
    vertices.push(LineVertex::new(left, icon.tint));
    vertices.push(LineVertex::new(right_pt, icon.tint));
    vertices.push(LineVertex::new(top, icon.tint));
    vertices.push(LineVertex::new(bottom, icon.tint));
    vertices.push(LineVertex::new(left, icon.tint));
    vertices.push(LineVertex::new(top, icon.tint));
    vertices.push(LineVertex::new(top, icon.tint));
    vertices.push(LineVertex::new(right_pt, icon.tint));
    vertices.push(LineVertex::new(right_pt, icon.tint));
    vertices.push(LineVertex::new(bottom, icon.tint));
    vertices.push(LineVertex::new(bottom, icon.tint));
    vertices.push(LineVertex::new(left, icon.tint));
}
