use zircon_math::{Transform, Vec3};
use zircon_scene::{ProjectionMode, Scene, ViewOrientation};

use super::SceneViewportController;

const FRAME_DISTANCE: f32 = 6.0;
const FRAME_ORTHO_SIZE: f32 = 2.5;

impl SceneViewportController {
    pub(in crate::editing::viewport::controller) fn frame_selection(
        &mut self,
        scene: &Scene,
    ) -> bool {
        let Some(target) = Self::selected_world_position(scene) else {
            return false;
        };

        let mut camera = self.current_camera(scene);
        let offset = camera.transform.translation - target;
        let direction = if offset.length_squared() > f32::EPSILON {
            offset.normalize_or_zero()
        } else {
            Vec3::new(0.6, 0.45, 1.0).normalize_or_zero()
        };
        let distance = offset.length().max(FRAME_DISTANCE);
        camera.transform = Transform::looking_at(target + direction * distance, target, Vec3::Y);
        if camera.projection_mode == ProjectionMode::Orthographic {
            camera.ortho_size = FRAME_ORTHO_SIZE.max(distance * 0.35);
        }
        camera.apply_viewport_size(self.state.viewport.size);

        self.state.camera = Some(camera);
        self.state.orbit_target = target;
        self.state.settings.view_orientation = ViewOrientation::User;
        true
    }
}
