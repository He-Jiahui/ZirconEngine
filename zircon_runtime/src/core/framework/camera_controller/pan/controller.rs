use crate::core::framework::camera_controller::CameraControllerOutput;
use crate::core::math::{clamp_viewport_size, Quat, Real, Transform, Vec2, Vec3};

use super::{PanCameraInput, PanCameraSettings, PanCameraState};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PanCameraController {
    settings: PanCameraSettings,
    state: PanCameraState,
}

impl PanCameraController {
    pub fn new(settings: PanCameraSettings, state: PanCameraState) -> Self {
        Self { settings, state }
    }

    pub fn settings(&self) -> &PanCameraSettings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut PanCameraSettings {
        &mut self.settings
    }

    pub fn state(&self) -> &PanCameraState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut PanCameraState {
        &mut self.state
    }

    pub fn update(
        &mut self,
        transform: Transform,
        input: PanCameraInput,
    ) -> CameraControllerOutput {
        if !self.state.enabled {
            return CameraControllerOutput::unchanged(transform);
        }

        let before = transform;
        let mut transform = transform;
        let delta_seconds = input.delta_seconds.max(0.0);

        apply_keyboard_pan(
            &mut transform,
            input.pan_axis,
            self.settings.pan_speed.max(0.0),
            delta_seconds,
        );
        apply_drag_pan(
            &mut transform,
            input.drag_delta,
            self.state.zoom_factor,
            self.settings.drag_pan_speed.max(0.0),
            input.viewport_size,
        );
        apply_rotation(
            &mut transform,
            input.rotate_axis,
            self.settings.rotation_speed,
            delta_seconds,
        );
        self.apply_zoom(&mut transform, input.zoom_delta);

        CameraControllerOutput::from_transform(before, transform)
    }

    fn apply_zoom(&mut self, transform: &mut Transform, zoom_delta: Real) {
        if zoom_delta == 0.0 {
            return;
        }
        let min_zoom = self.settings.min_zoom.min(self.settings.max_zoom);
        let max_zoom = self.settings.min_zoom.max(self.settings.max_zoom);
        self.state.zoom_factor = (self.state.zoom_factor - zoom_delta * self.settings.zoom_speed)
            .clamp(min_zoom, max_zoom);
        transform.scale = Vec3::splat(self.state.zoom_factor);
    }
}

impl Default for PanCameraController {
    fn default() -> Self {
        Self::new(PanCameraSettings::default(), PanCameraState::default())
    }
}

fn apply_keyboard_pan(
    transform: &mut Transform,
    pan_axis: Vec2,
    pan_speed: Real,
    delta_seconds: Real,
) {
    if pan_axis == Vec2::ZERO || delta_seconds == 0.0 {
        return;
    }
    let direction =
        (transform.right() * pan_axis.x + transform.up() * pan_axis.y).normalize_or_zero();
    transform.translation += direction * pan_speed * delta_seconds;
}

fn apply_drag_pan(
    transform: &mut Transform,
    drag_delta: Vec2,
    zoom_factor: Real,
    drag_pan_speed: Real,
    viewport_size: crate::core::math::UVec2,
) {
    if drag_delta == Vec2::ZERO {
        return;
    }
    let viewport_size = clamp_viewport_size(viewport_size);
    let reference_extent = viewport_size.x.min(viewport_size.y) as Real;
    let world_per_pixel = zoom_factor.max(0.0) * drag_pan_speed / reference_extent.max(1.0);
    transform.translation +=
        (-transform.right() * drag_delta.x + transform.up() * drag_delta.y) * world_per_pixel;
}

fn apply_rotation(
    transform: &mut Transform,
    rotate_axis: Real,
    rotation_speed: Real,
    delta_seconds: Real,
) {
    if rotate_axis == 0.0 || delta_seconds == 0.0 {
        return;
    }
    transform.rotation =
        Quat::from_rotation_z(rotation_speed * rotate_axis * delta_seconds) * transform.rotation;
}
