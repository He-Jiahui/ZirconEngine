use crate::core::framework::render::ViewportCameraSnapshot;
use crate::core::math::{Real, Vec3};

pub(crate) fn perspective_visible(
    view_position: Vec3,
    depth: Real,
    radius: Real,
    camera: &ViewportCameraSnapshot,
) -> bool {
    let clamped_depth = depth.max(camera.z_near.max(0.001));
    let half_height = clamped_depth * (camera.fov_y_radians * 0.5).tan();
    let half_width = half_height * camera.aspect_ratio.max(0.001);
    view_position.x.abs() <= half_width + radius && view_position.y.abs() <= half_height + radius
}
