use zircon_math::{Transform, Vec2, Vec3};
use zircon_runtime::scene::Scene;

use super::runtime_camera_controller::RuntimeCameraController;

impl RuntimeCameraController {
    pub(super) fn apply_orbit(&self, scene: &mut Scene, previous: Vec2, current: Vec2) {
        let delta = (current - previous) * 0.01;
        let Some(camera) = scene.find_node(scene.active_camera()).cloned() else {
            return;
        };
        let offset = camera.transform.translation - self.orbit_target;
        let distance = offset.length().max(0.001);
        let mut yaw = offset.x.atan2(offset.z);
        let horizontal = (offset.x * offset.x + offset.z * offset.z)
            .sqrt()
            .max(0.001);
        let mut pitch = offset.y.atan2(horizontal);

        yaw -= delta.x;
        pitch = (pitch + delta.y).clamp(-1.4, 1.4);

        let next_offset = Vec3::new(
            distance * pitch.cos() * yaw.sin(),
            distance * pitch.sin(),
            distance * pitch.cos() * yaw.cos(),
        );
        let transform =
            Transform::looking_at(self.orbit_target + next_offset, self.orbit_target, Vec3::Y);
        let _ = scene.update_transform(camera.id, transform);
    }
}
