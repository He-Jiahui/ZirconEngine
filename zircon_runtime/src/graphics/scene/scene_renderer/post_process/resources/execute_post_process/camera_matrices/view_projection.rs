use crate::core::framework::render::{ProjectionMode, ViewportCameraSnapshot};
use crate::core::math::{view_matrix, Mat4, UVec2};

use super::orthographic_projection::orthographic_projection;
use super::perspective_projection::perspective_projection;

pub(in super::super) fn view_projection(
    camera: &ViewportCameraSnapshot,
    viewport_size: UVec2,
) -> (Mat4, Mat4) {
    let aspect = viewport_size.x.max(1) as f32 / viewport_size.y.max(1) as f32;
    let projection = match camera.projection_mode {
        ProjectionMode::Perspective => perspective_projection(camera, aspect),
        ProjectionMode::Orthographic => orthographic_projection(camera, aspect),
    };
    let view = view_matrix(camera.transform);
    (view, projection)
}
