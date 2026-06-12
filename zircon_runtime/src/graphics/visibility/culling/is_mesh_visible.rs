use crate::core::framework::render::{ProjectionMode, ViewportCameraSnapshot};
use crate::core::math::view_matrix;

use super::{orthographic_visible::orthographic_visible, perspective_visible::perspective_visible};
use crate::graphics::visibility::VisibilityBounds;

pub(crate) fn is_bounds_visible(bounds: VisibilityBounds, camera: &ViewportCameraSnapshot) -> bool {
    let world_center = bounds.center;
    let world_radius = bounds.radius;
    let view_position = view_matrix(camera.transform).transform_point3(world_center);
    let depth = -view_position.z;
    let near = camera.z_near.max(0.001);
    let far = camera.z_far.max(near);

    if depth + world_radius < near || depth - world_radius > far {
        return false;
    }

    match camera.projection_mode {
        ProjectionMode::Perspective => {
            perspective_visible(view_position, depth, world_radius, camera)
        }
        ProjectionMode::Orthographic => orthographic_visible(view_position, world_radius, camera),
    }
}
