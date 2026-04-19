use zircon_runtime::core::math::{UVec2, Vec2, Vec3};
use zircon_runtime::scene::Scene;

use super::drag_state::DragState;
use super::runtime_camera_controller::RuntimeCameraController;

impl RuntimeCameraController {
    pub(in crate::entry::runtime_entry_app) fn viewport_size(&self) -> UVec2 {
        self.viewport_size
    }

    pub(in crate::entry::runtime_entry_app) fn set_orbit_target(&mut self, target: Vec3) {
        self.orbit_target = target;
    }

    #[cfg(test)]
    pub(in crate::entry::runtime_entry_app) fn orbit_target(&self) -> Vec3 {
        self.orbit_target
    }

    pub(in crate::entry::runtime_entry_app) fn pointer_moved(
        &mut self,
        scene: &mut Scene,
        position: Vec2,
    ) {
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

    pub(in crate::entry::runtime_entry_app) fn left_pressed(&mut self, _position: Vec2) {}

    pub(in crate::entry::runtime_entry_app) fn left_released(&mut self) {}

    pub(in crate::entry::runtime_entry_app) fn right_pressed(&mut self, position: Vec2) {
        self.drag = Some(DragState::Orbit { last: position });
    }

    pub(in crate::entry::runtime_entry_app) fn right_released(&mut self) {
        self.drag = None;
    }

    pub(in crate::entry::runtime_entry_app) fn middle_pressed(&mut self, position: Vec2) {
        self.drag = Some(DragState::Pan { last: position });
    }

    pub(in crate::entry::runtime_entry_app) fn middle_released(&mut self) {
        self.drag = None;
    }

    pub(in crate::entry::runtime_entry_app) fn scrolled(&mut self, scene: &mut Scene, delta: f32) {
        self.apply_zoom(scene, delta);
    }
}
