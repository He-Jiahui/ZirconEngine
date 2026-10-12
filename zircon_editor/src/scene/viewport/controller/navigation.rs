use crate::scene::viewport::{ProjectionMode, ViewOrientation};
use zircon_runtime::core::framework::camera_controller::OrbitCameraInput;
use zircon_runtime_interface::math::Vec2;

use super::SceneViewportController;

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

        self.state
            .orbit_controller
            .set_target(self.state.orbit_target);
        let output = self.state.orbit_controller.update(
            camera.transform,
            OrbitCameraInput::orbit(previous, current).with_viewport_size(self.state.viewport.size),
        );
        camera.transform = output.transform;
        self.state.settings.view_orientation = ViewOrientation::User;
        output.changed
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
        let changed = match camera.projection_mode {
            ProjectionMode::Perspective => {
                self.state
                    .orbit_controller
                    .set_target(self.state.orbit_target);
                let output = self.state.orbit_controller.update(
                    camera.transform,
                    OrbitCameraInput::pan(previous, current)
                        .with_viewport_size(self.state.viewport.size),
                );
                camera.transform = output.transform;
                self.state.orbit_target = self.state.orbit_controller.target();
                output.changed
            }
            ProjectionMode::Orthographic => {
                let world_per_pixel = camera.ortho_size.max(MIN_ORTHO_SIZE) * 2.0
                    / self.state.viewport.size.y.max(1) as f32;
                let translation = (-camera.transform.right() * delta.x
                    + camera.transform.up() * delta.y)
                    * world_per_pixel;
                camera.transform.translation += translation;
                self.state.orbit_target += translation;
                self.state
                    .orbit_controller
                    .set_target(self.state.orbit_target);
                delta != Vec2::ZERO
            }
        };

        self.state.settings.view_orientation = ViewOrientation::User;
        changed
    }

    pub(in crate::scene::viewport::controller) fn apply_zoom(&mut self, delta: f32) -> bool {
        let Some(camera) = self.state.camera.as_mut() else {
            return false;
        };

        match camera.projection_mode {
            ProjectionMode::Perspective => {
                self.state
                    .orbit_controller
                    .set_target(self.state.orbit_target);
                let output = self.state.orbit_controller.update(
                    camera.transform,
                    OrbitCameraInput::zoom(delta).with_viewport_size(self.state.viewport.size),
                );
                camera.transform = output.transform;
                if !output.changed {
                    return false;
                }
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
