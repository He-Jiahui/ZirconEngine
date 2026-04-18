use zircon_math::{Transform, Vec2};
use zircon_runtime::scene::Scene;

use super::runtime_camera_controller::RuntimeCameraController;

impl RuntimeCameraController {
    pub(super) fn apply_pan(&mut self, scene: &mut Scene, previous: Vec2, current: Vec2) {
        let delta = current - previous;
        let Some(camera) = scene.find_node(scene.active_camera()).cloned() else {
            return;
        };
        let distance = (camera.transform.translation - self.orbit_target)
            .length()
            .max(0.5);
        let world_per_pixel = distance * 0.0015;
        let translation = (-camera.transform.right() * delta.x + camera.transform.up() * delta.y)
            * world_per_pixel;
        let transform = Transform {
            translation: camera.transform.translation + translation,
            ..camera.transform
        };
        self.orbit_target += translation;
        let _ = scene.update_transform(camera.id, transform);
    }
}
