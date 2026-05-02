use crate::scene::viewport::ViewportCameraSnapshot;
use zircon_runtime_interface::math::{UVec2, Vec2, Vec3};

use crate::scene::viewport::projection::project_point;

pub(in crate::scene::viewport::handles) fn projected_axis_delta(
    start_cursor: Vec2,
    current_cursor: Vec2,
    origin: Vec3,
    axis: Vec3,
    camera: &ViewportCameraSnapshot,
    viewport: UVec2,
) -> Option<f32> {
    let start = project_point(origin, camera, viewport)?;
    let end = project_point(origin + axis.normalize_or_zero(), camera, viewport)?;
    let direction = (end - start).normalize_or_zero();
    if direction.length_squared() <= f32::EPSILON {
        return None;
    }
    Some((current_cursor - start_cursor).dot(direction))
}

pub(in crate::scene::viewport::handles) fn maybe_snap(value: f32, enabled: bool, step: f32) -> f32 {
    if enabled && step > f32::EPSILON {
        (value / step).round() * step
    } else {
        value
    }
}
