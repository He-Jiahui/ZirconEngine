use zircon_runtime::core::math::Transform;
use zircon_runtime::scene::Scene;

use super::runtime_camera_controller::RuntimeCameraController;

impl RuntimeCameraController {
    pub(super) fn apply_zoom(&self, scene: &mut Scene, delta: f32) {
        let Some(camera) = scene.find_node(scene.active_camera()).cloned() else {
            return;
        };
        let direction = camera.transform.forward();
        let distance = (camera.transform.translation - self.orbit_target)
            .length()
            .max(0.25);
        let step = (distance * 0.15 * delta.signum()).min(distance - 0.25);
        let transform = Transform {
            translation: camera.transform.translation + direction * step,
            ..camera.transform
        };
        let _ = scene.update_transform(camera.id, transform);
    }
}
