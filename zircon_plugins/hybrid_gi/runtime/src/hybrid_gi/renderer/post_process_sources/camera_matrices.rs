use zircon_runtime::core::framework::render::{ProjectionMode, ViewportCameraSnapshot};
use zircon_runtime::core::math::{view_matrix, Mat4, UVec2};

pub(super) fn view_projection(
    camera: &ViewportCameraSnapshot,
    viewport_size: UVec2,
) -> (Mat4, Mat4) {
    let aspect = viewport_size.x.max(1) as f32 / viewport_size.y.max(1) as f32;
    let projection = match camera.projection_mode {
        ProjectionMode::Perspective => Mat4::perspective_rh_gl(
            camera.fov_y_radians,
            aspect,
            camera.z_near.max(0.001),
            camera.z_far.max(camera.z_near + 0.001),
        ),
        ProjectionMode::Orthographic => {
            let half_height = camera.ortho_size.max(0.001) * 0.5;
            let half_width = half_height * aspect;
            Mat4::orthographic_rh_gl(
                -half_width,
                half_width,
                -half_height,
                half_height,
                camera.z_near,
                camera.z_far.max(camera.z_near + 0.001),
            )
        }
    };
    let view = view_matrix(camera.transform);
    (view, projection)
}
