use crate::core::framework::camera_controller::{CameraControllerOutput, CursorGrabIntent};
use crate::core::math::{EulerRot, Quat, Real, Transform, Vec2, Vec3};

use super::{FreeCameraInput, FreeCameraSettings, FreeCameraState};

const RADIANS_PER_DOT: Real = 1.0 / 180.0;
const VELOCITY_STOP_EPSILON: Real = 0.000001;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FreeCameraController {
    settings: FreeCameraSettings,
    state: FreeCameraState,
}

impl FreeCameraController {
    pub fn new(settings: FreeCameraSettings, state: FreeCameraState) -> Self {
        Self { settings, state }
    }

    pub fn settings(&self) -> &FreeCameraSettings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut FreeCameraSettings {
        &mut self.settings
    }

    pub fn state(&self) -> &FreeCameraState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut FreeCameraState {
        &mut self.state
    }

    pub fn update(
        &mut self,
        transform: Transform,
        input: FreeCameraInput,
    ) -> CameraControllerOutput {
        if !self.state.enabled {
            let output = CameraControllerOutput::unchanged(transform);
            return if input.cursor_grab_active || input.cursor_grab_changed {
                output.with_cursor_grab(CursorGrabIntent::released())
            } else {
                output
            };
        }

        let before = transform;
        let mut transform = transform;
        let delta_seconds = input.delta_seconds.max(0.0);

        self.apply_speed_scroll(input.scroll_lines);
        self.apply_velocity(input.movement_axis, input.run, delta_seconds);
        apply_translation(&mut transform, self.state.velocity, delta_seconds);
        self.apply_look(&mut transform, input.look_delta, input);

        let mut output = CameraControllerOutput::from_transform(before, transform);
        if input.cursor_grab_changed {
            output.cursor_grab = Some(if input.cursor_grab_active && input.focus_active {
                CursorGrabIntent::locked(true)
            } else {
                CursorGrabIntent::released()
            });
        }
        output
    }

    fn apply_speed_scroll(&mut self, scroll_lines: Real) {
        if scroll_lines == 0.0 {
            return;
        }
        self.state.speed_multiplier *= (self.settings.scroll_factor * scroll_lines).exp();
        self.state.speed_multiplier = self.state.speed_multiplier.clamp(f32::EPSILON, f32::MAX);
    }

    fn apply_velocity(&mut self, movement_axis: Vec3, run: bool, delta_seconds: Real) {
        if movement_axis != Vec3::ZERO {
            let base_speed = if run {
                self.settings.run_speed
            } else {
                self.settings.walk_speed
            };
            self.state.velocity = movement_axis.normalize_or_zero()
                * base_speed.max(0.0)
                * self.state.speed_multiplier;
            return;
        }

        let damping = (-self.settings.friction.max(0.0) * delta_seconds).exp();
        self.state.velocity *= damping;
        if self.state.velocity.length_squared() < VELOCITY_STOP_EPSILON {
            self.state.velocity = Vec3::ZERO;
        }
    }

    fn apply_look(&mut self, transform: &mut Transform, look_delta: Vec2, input: FreeCameraInput) {
        if look_delta == Vec2::ZERO || !input.focus_active {
            return;
        }
        if !input.look_active && !input.cursor_grab_active {
            return;
        }

        let pitch_min = self.settings.pitch_min.min(self.settings.pitch_max);
        let pitch_max = self.settings.pitch_min.max(self.settings.pitch_max);
        self.state.pitch = (self.state.pitch
            - look_delta.y * RADIANS_PER_DOT * self.settings.sensitivity)
            .clamp(pitch_min, pitch_max);
        self.state.yaw -= look_delta.x * RADIANS_PER_DOT * self.settings.sensitivity;
        transform.rotation = Quat::from_euler(EulerRot::ZYX, 0.0, self.state.yaw, self.state.pitch);
    }
}

impl Default for FreeCameraController {
    fn default() -> Self {
        Self::new(FreeCameraSettings::default(), FreeCameraState::default())
    }
}

fn apply_translation(transform: &mut Transform, velocity: Vec3, delta_seconds: Real) {
    if velocity == Vec3::ZERO || delta_seconds == 0.0 {
        return;
    }
    transform.translation += velocity.x * delta_seconds * transform.right()
        + velocity.y * delta_seconds * Vec3::Y
        + velocity.z * delta_seconds * transform.forward();
}
