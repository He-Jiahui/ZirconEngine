use zircon_math::{view_matrix, Vec3};
use zircon_scene::{ProjectionMode, ViewportCameraSnapshot};

use super::super::super::culling::{
    orthographic_visible::orthographic_visible, perspective_visible::perspective_visible,
};

pub(super) fn sphere_visible(center: Vec3, radius: f32, camera: &ViewportCameraSnapshot) -> bool {
    let view_position = view_matrix(camera.transform).transform_point3(center);
    let depth = -view_position.z;
    let near = camera.z_near.max(0.001);
    let far = camera.z_far.max(near);
    let radius = radius.max(0.0);

    if depth + radius < near || depth - radius > far {
        return false;
    }

    match camera.projection_mode {
        ProjectionMode::Perspective => perspective_visible(view_position, depth, radius, camera),
        ProjectionMode::Orthographic => orthographic_visible(view_position, radius, camera),
    }
}
