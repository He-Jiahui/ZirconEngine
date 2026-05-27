use crate::core::framework::camera_controller::{OrbitCameraController, OrbitCameraInput};
use crate::core::math::{clamp_viewport_size, UVec2, Vec2, Vec3};
use crate::scene::Scene;

#[derive(Clone, Copy, Debug)]
enum DragState {
    Orbit { last: Vec2 },
    Pan { last: Vec2 },
}

#[derive(Clone, Debug)]
pub(super) struct RuntimeCameraController {
    viewport_size: UVec2,
    orbit: OrbitCameraController,
    drag: Option<DragState>,
}

impl RuntimeCameraController {
    pub(super) fn new(viewport_size: UVec2) -> Self {
        Self {
            viewport_size: clamp_viewport_size(viewport_size),
            orbit: OrbitCameraController::with_target(Vec3::ZERO),
            drag: None,
        }
    }

    pub(super) fn viewport_size(&self) -> UVec2 {
        self.viewport_size
    }

    pub(super) fn resize(&mut self, size: UVec2) {
        self.viewport_size = clamp_viewport_size(size);
    }

    pub(super) fn set_orbit_target(&mut self, target: Vec3) {
        self.orbit.set_target(target);
    }

    pub(super) fn pointer_moved(&mut self, scene: &mut Scene, position: Vec2) {
        match self.drag.take() {
            Some(DragState::Orbit { last }) => {
                self.apply_orbit(scene, last, position);
                self.drag = Some(DragState::Orbit { last: position });
            }
            Some(DragState::Pan { last }) => {
                self.apply_pan(scene, last, position);
                self.drag = Some(DragState::Pan { last: position });
            }
            None => {}
        }
    }

    pub(super) fn left_pressed(&mut self, _position: Vec2) {}

    pub(super) fn left_released(&mut self) {}

    pub(super) fn right_pressed(&mut self, position: Vec2) {
        self.drag = Some(DragState::Orbit { last: position });
    }

    pub(super) fn right_released(&mut self) {
        self.drag = None;
    }

    pub(super) fn middle_pressed(&mut self, position: Vec2) {
        self.drag = Some(DragState::Pan { last: position });
    }

    pub(super) fn middle_released(&mut self) {
        self.drag = None;
    }

    pub(super) fn scrolled(&mut self, scene: &mut Scene, delta: f32) {
        self.apply_zoom(scene, delta);
    }

    fn apply_orbit(&mut self, scene: &mut Scene, previous: Vec2, current: Vec2) {
        let Some(camera) = scene.find_node(scene.active_camera()) else {
            return;
        };
        let output = self.orbit.update(
            camera.transform,
            OrbitCameraInput::orbit(previous, current).with_viewport_size(self.viewport_size),
        );
        let _ = scene.update_transform(camera.id, output.transform);
    }

    fn apply_pan(&mut self, scene: &mut Scene, previous: Vec2, current: Vec2) {
        let Some(camera) = scene.find_node(scene.active_camera()) else {
            return;
        };
        let output = self.orbit.update(
            camera.transform,
            OrbitCameraInput::pan(previous, current).with_viewport_size(self.viewport_size),
        );
        let _ = scene.update_transform(camera.id, output.transform);
    }

    fn apply_zoom(&mut self, scene: &mut Scene, delta: f32) {
        let Some(camera) = scene.find_node(scene.active_camera()) else {
            return;
        };
        let output = self.orbit.update(
            camera.transform,
            OrbitCameraInput::zoom(delta).with_viewport_size(self.viewport_size),
        );
        let _ = scene.update_transform(camera.id, output.transform);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_runtime_camera_controller_scroll_uses_runtime_orbit_controller() {
        let mut scene = Scene::new();
        let camera = scene.active_camera();
        let before = scene.find_node(camera).unwrap().transform;
        let mut controller = RuntimeCameraController::new(UVec2::new(800, 600));

        controller.set_orbit_target(Vec3::ZERO);
        controller.scrolled(&mut scene, 1.0);

        let after = scene.find_node(camera).unwrap().transform;
        assert!(after.translation.length() < before.translation.length());
    }
}
