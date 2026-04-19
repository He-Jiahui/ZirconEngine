use zircon_framework::render::ViewportCameraSnapshot;
use zircon_math::{Real, Vec3};

pub(crate) fn orthographic_visible(
    view_position: Vec3,
    radius: Real,
    camera: &ViewportCameraSnapshot,
) -> bool {
    let half_height = camera.ortho_size.max(0.01);
    let half_width = half_height * camera.aspect_ratio.max(0.001);
    view_position.x.abs() <= half_width + radius && view_position.y.abs() <= half_height + radius
}
