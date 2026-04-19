use crate::core::framework::render::ViewportCameraSnapshot;
use crate::core::math::Vec3;

pub(super) fn reflection_probe_radius(
    _camera: &ViewportCameraSnapshot,
    probe_radius: f32,
    probe_position: Vec3,
    camera_position: Vec3,
) -> f32 {
    let distance = (camera_position - probe_position).length().max(1.0);
    (probe_radius.max(0.05) / distance).clamp(0.04, 0.6)
}
