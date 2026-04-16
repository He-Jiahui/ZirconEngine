use zircon_math::Vec3;
use zircon_scene::{OverlayBillboardIcon, ViewportIconId};

use crate::scene::scene_renderer::primitives::LineVertex;

use super::super::icons::icon_world_size;
use super::super::line_geometry::append_cross;

pub(crate) fn append_icon_fallback_lines(
    vertices: &mut Vec<LineVertex>,
    icon: &OverlayBillboardIcon,
    right: Vec3,
    up: Vec3,
) {
    let size = icon_world_size(icon);
    match icon.id {
        ViewportIconId::Camera => {
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
        ViewportIconId::DirectionalLight => {
            append_cross(vertices, icon.position, size, icon.tint, right, up);
            let diagonal = (right + up).normalize_or_zero();
            vertices.push(LineVertex::new(
                icon.position - diagonal * size * 0.7,
                icon.tint,
            ));
            vertices.push(LineVertex::new(
                icon.position + diagonal * size * 0.7,
                icon.tint,
            ));
            let diagonal = (right - up).normalize_or_zero();
            vertices.push(LineVertex::new(
                icon.position - diagonal * size * 0.7,
                icon.tint,
            ));
            vertices.push(LineVertex::new(
                icon.position + diagonal * size * 0.7,
                icon.tint,
            ));
        }
    }
}
