use crate::core::math::{Transform, UVec2, Vec2, Vec3};
use crate::scene::Scene;

#[derive(Clone, Copy, Debug)]
enum DragState {
    Orbit { last: Vec2 },
    Pan { last: Vec2 },
}

#[derive(Clone, Debug)]
pub(super) struct RuntimeCameraController {
    viewport_size: UVec2,
    orbit_target: Vec3,
    drag: Option<DragState>,
}

impl RuntimeCameraController {
    pub(super) fn new(viewport_size: UVec2) -> Self {
        Self {
            viewport_size: clamp_size(viewport_size),
            orbit_target: Vec3::ZERO,
            drag: None,
        }
    }

    pub(super) fn viewport_size(&self) -> UVec2 {
        self.viewport_size
    }

    pub(super) fn resize(&mut self, size: UVec2) {
        self.viewport_size = clamp_size(size);
    }

    pub(super) fn set_orbit_target(&mut self, target: Vec3) {
        self.orbit_target = target;
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

    fn apply_orbit(&self, scene: &mut Scene, previous: Vec2, current: Vec2) {
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

    fn apply_pan(&mut self, scene: &mut Scene, previous: Vec2, current: Vec2) {
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

    fn apply_zoom(&self, scene: &mut Scene, delta: f32) {
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

fn clamp_size(size: UVec2) -> UVec2 {
    UVec2::new(size.x.max(1), size.y.max(1))
}
