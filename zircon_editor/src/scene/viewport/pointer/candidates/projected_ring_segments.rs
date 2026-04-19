use crate::scene::viewport::ViewportCameraSnapshot;
use zircon_runtime::core::math::{UVec2, Vec2, Vec3};

use crate::scene::viewport::projection::projected_point;

pub(in crate::scene::viewport::pointer) fn projected_ring_segments(
    center: Vec3,
    normal: Vec3,
    radius: f32,
    camera: &ViewportCameraSnapshot,
    viewport: UVec2,
) -> Vec<(Vec2, Vec2)> {
    let normal = normal.normalize_or_zero();
    if normal.length_squared() <= f32::EPSILON {
        return Vec::new();
    }
    let tangent = if normal.cross(Vec3::Y).length_squared() > f32::EPSILON {
        normal.cross(Vec3::Y).normalize_or_zero()
    } else {
        normal.cross(Vec3::X).normalize_or_zero()
    };
    let bitangent = normal.cross(tangent).normalize_or_zero();
    const SEGMENTS: usize = 48;
    let mut segments = Vec::new();
    let mut previous = center + tangent * radius;
    for step in 1..=SEGMENTS {
        let angle = std::f32::consts::TAU * step as f32 / SEGMENTS as f32;
        let next = center + (tangent * angle.cos() + bitangent * angle.sin()) * radius;
        let (Some(a), Some(b)) = (
            projected_point(previous, camera, viewport),
            projected_point(next, camera, viewport),
        ) else {
            previous = next;
            continue;
        };
        segments.push((a.position, b.position));
        previous = next;
    }
    segments
}
