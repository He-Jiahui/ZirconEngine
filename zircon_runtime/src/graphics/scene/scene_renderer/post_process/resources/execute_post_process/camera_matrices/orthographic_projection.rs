use crate::core::framework::render::ViewportCameraSnapshot;
use crate::core::math::Mat4;

pub(super) fn orthographic_projection(camera: &ViewportCameraSnapshot, aspect: f32) -> Mat4 {
    let half_height = camera.ortho_size.max(0.01);
    let half_width = half_height * aspect.max(0.001);
    Mat4::orthographic_rh(
        -half_width,
        half_width,
        -half_height,
        half_height,
        camera.z_near.max(0.001),
        camera.z_far,
    )
}
