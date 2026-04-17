use zircon_math::Mat4;
use zircon_scene::ViewportCameraSnapshot;

pub(super) fn perspective_projection(camera: &ViewportCameraSnapshot, aspect: f32) -> Mat4 {
    Mat4::perspective_rh(
        camera.fov_y_radians,
        aspect.max(0.001),
        camera.z_near.max(0.001),
        camera.z_far,
    )
}
