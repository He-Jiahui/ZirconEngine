use zircon_math::{Transform, Vec2, Vec3};
use zircon_scene::{ProjectionMode, ViewOrientation};

use super::{constants::MIN_CAMERA_DISTANCE, SceneViewportController};

const ORBIT_SENSITIVITY: f32 = 0.01;
const PAN_PERSPECTIVE_FACTOR: f32 = 0.0015;
const ORTHO_SCROLL_FACTOR: f32 = 0.1;
const MIN_ORTHO_SIZE: f32 = 0.25;

impl SceneViewportController {
    pub(in crate::scene::viewport::controller) fn apply_orbit(
        &mut self,
        previous: Vec2,
        current: Vec2,
    ) -> bool {
        let Some(camera) = self.state.camera.as_mut() else {
            return false;
        };

        let delta = (current - previous) * ORBIT_SENSITIVITY;
        let offset = camera.transform.translation - self.state.orbit_target;
        let distance = offset.length().max(MIN_CAMERA_DISTANCE);
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
        camera.transform = Transform::looking_at(
            self.state.orbit_target + next_offset,
            self.state.orbit_target,
            Vec3::Y,
        );
        self.state.settings.view_orientation = ViewOrientation::User;
        true
    }

    pub(in crate::scene::viewport::controller) fn apply_pan(
        &mut self,
        previous: Vec2,
        current: Vec2,
    ) -> bool {
        let Some(camera) = self.state.camera.as_mut() else {
            return false;
        };

        let delta = current - previous;
        let translation = match camera.projection_mode {
            ProjectionMode::Perspective => {
                let distance = (camera.transform.translation - self.state.orbit_target)
                    .length()
                    .max(0.5);
                let world_per_pixel = distance * PAN_PERSPECTIVE_FACTOR;
                (-camera.transform.right() * delta.x + camera.transform.up() * delta.y)
                    * world_per_pixel
            }
            ProjectionMode::Orthographic => {
                let world_per_pixel = camera.ortho_size.max(MIN_ORTHO_SIZE) * 2.0
                    / self.state.viewport.size.y.max(1) as f32;
                (-camera.transform.right() * delta.x + camera.transform.up() * delta.y)
                    * world_per_pixel
            }
        };

        camera.transform.translation += translation;
        self.state.orbit_target += translation;
        self.state.settings.view_orientation = ViewOrientation::User;
        true
    }

    pub(in crate::scene::viewport::controller) fn apply_zoom(&mut self, delta: f32) -> bool {
        let Some(camera) = self.state.camera.as_mut() else {
            return false;
        };

        match camera.projection_mode {
            ProjectionMode::Perspective => {
                let direction = camera.transform.forward();
                let distance = (camera.transform.translation - self.state.orbit_target)
                    .length()
                    .max(MIN_CAMERA_DISTANCE);
                let step = (distance * 0.15 * delta.signum()).min(distance - MIN_CAMERA_DISTANCE);
                camera.transform.translation += direction * step;
            }
            ProjectionMode::Orthographic => {
                let scale = 1.0 - delta.signum() * ORTHO_SCROLL_FACTOR;
                camera.ortho_size = (camera.ortho_size * scale).max(MIN_ORTHO_SIZE);
            }
        }
        self.state.settings.view_orientation = ViewOrientation::User;
        true
    }
}
