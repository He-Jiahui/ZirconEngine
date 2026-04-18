use zircon_math::view_matrix;
use zircon_scene::{ProjectionMode, RenderVirtualGeometryCluster, ViewportCameraSnapshot};

use super::super::super::super::culling::{
    orthographic_visible::orthographic_visible, perspective_visible::perspective_visible,
};

pub(in crate::visibility::planning::build_virtual_geometry_plan) fn cluster_visible(
    cluster: &RenderVirtualGeometryCluster,
    camera: &ViewportCameraSnapshot,
) -> bool {
    let view_position = view_matrix(camera.transform).transform_point3(cluster.bounds_center);
    let depth = -view_position.z;
    let near = camera.z_near.max(0.001);
    let far = camera.z_far.max(near);
    let radius = cluster.bounds_radius.max(0.0);

    if depth + radius < near || depth - radius > far {
        return false;
    }

    match camera.projection_mode {
        ProjectionMode::Perspective => perspective_visible(view_position, depth, radius, camera),
        ProjectionMode::Orthographic => orthographic_visible(view_position, radius, camera),
    }
}
