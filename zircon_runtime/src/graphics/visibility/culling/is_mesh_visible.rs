use crate::core::framework::render::{ProjectionMode, RenderMeshSnapshot, ViewportCameraSnapshot};
use crate::core::math::view_matrix;

use super::{
    mesh_bounds::mesh_bounds, orthographic_visible::orthographic_visible,
    perspective_visible::perspective_visible,
};

pub(crate) fn is_mesh_visible(mesh: &RenderMeshSnapshot, camera: &ViewportCameraSnapshot) -> bool {
    let bounds = mesh_bounds(mesh);
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
