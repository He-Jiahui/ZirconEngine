use crate::core::framework::camera_controller::CameraControllerOutput;
use crate::core::math::{Real, Transform, Vec2, Vec3};

use super::{OrbitCameraAction, OrbitCameraInput, OrbitCameraSettings, OrbitCameraState};

const ORBIT_DISTANCE_EPSILON: Real = 0.001;
const PAN_DISTANCE_FLOOR: Real = 0.5;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrbitCameraController {
    settings: OrbitCameraSettings,
    state: OrbitCameraState,
}

impl OrbitCameraController {
    pub fn new(settings: OrbitCameraSettings, state: OrbitCameraState) -> Self {
        Self { settings, state }
    }

    pub fn with_target(target: Vec3) -> Self {
        Self::new(
            OrbitCameraSettings::default(),
            OrbitCameraState {
                target,
                ..OrbitCameraState::default()
            },
        )
    }

    pub fn settings(&self) -> &OrbitCameraSettings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut OrbitCameraSettings {
        &mut self.settings
    }

    pub fn state(&self) -> &OrbitCameraState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut OrbitCameraState {
        &mut self.state
    }

    pub fn target(&self) -> Vec3 {
        self.state.target
    }

    pub fn set_target(&mut self, target: Vec3) {
        self.state.target = target;
    }

    pub fn update(
        &mut self,
        transform: Transform,
        input: OrbitCameraInput,
    ) -> CameraControllerOutput {
        if !self.state.enabled || !input.focus_active {
            return CameraControllerOutput::unchanged(transform);
        }

        let before = transform;
        let mut transform = transform;
        match input.action {
            OrbitCameraAction::None => {}
            OrbitCameraAction::Orbit { previous, current } => {
                transform = orbit_transform(
                    self.settings,
                    self.state.target,
                    transform,
                    previous,
                    current,
                );
            }
            OrbitCameraAction::Pan { previous, current } => {
                transform =
                    pan_transform(self.settings, &mut self.state, transform, previous, current);
            }
            OrbitCameraAction::Zoom { delta } => {
                transform = zoom_transform(self.settings, self.state.target, transform, delta);
            }
            OrbitCameraAction::Focus { target } => {
                self.state.target = target;
            }
        }
        CameraControllerOutput::from_transform(before, transform)
    }
}

impl Default for OrbitCameraController {
    fn default() -> Self {
        Self::new(OrbitCameraSettings::default(), OrbitCameraState::default())
    }
}

fn orbit_transform(
    settings: OrbitCameraSettings,
    target: Vec3,
    transform: Transform,
    previous: Vec2,
    current: Vec2,
) -> Transform {
    let delta = (current - previous) * settings.orbit_sensitivity;
    if delta == Vec2::ZERO {
        return transform;
    }

    let offset = transform.translation - target;
    let distance = offset.length().max(ORBIT_DISTANCE_EPSILON);
    let mut yaw = offset.x.atan2(offset.z);
    let horizontal = (offset.x * offset.x + offset.z * offset.z)
        .sqrt()
        .max(ORBIT_DISTANCE_EPSILON);
    let mut pitch = offset.y.atan2(horizontal);
    let pitch_min = settings.pitch_min.min(settings.pitch_max);
    let pitch_max = settings.pitch_min.max(settings.pitch_max);

    yaw -= delta.x;
    pitch = (pitch + delta.y).clamp(pitch_min, pitch_max);

    let next_offset = Vec3::new(
        distance * pitch.cos() * yaw.sin(),
        distance * pitch.sin(),
        distance * pitch.cos() * yaw.cos(),
    );
    Transform::looking_at(target + next_offset, target, Vec3::Y)
}

fn pan_transform(
    settings: OrbitCameraSettings,
    state: &mut OrbitCameraState,
    transform: Transform,
    previous: Vec2,
    current: Vec2,
) -> Transform {
    let delta = current - previous;
    if delta == Vec2::ZERO {
        return transform;
    }

    let distance = (transform.translation - state.target)
        .length()
        .max(PAN_DISTANCE_FLOOR);
    let world_per_pixel = distance * settings.pan_sensitivity.max(0.0);
    let translation = (-transform.right() * delta.x + transform.up() * delta.y) * world_per_pixel;
    state.target += translation;
    Transform {
        translation: transform.translation + translation,
        ..transform
    }
}

fn zoom_transform(
    settings: OrbitCameraSettings,
    target: Vec3,
    transform: Transform,
    delta: Real,
) -> Transform {
    if delta == 0.0 {
        return transform;
    }

    let min_distance = settings.min_distance.max(ORBIT_DISTANCE_EPSILON);
    let distance = (transform.translation - target).length().max(min_distance);
    let step =
        (distance * settings.zoom_fraction.max(0.0) * delta.signum()).min(distance - min_distance);
    Transform {
        translation: transform.translation + transform.forward() * step,
        ..transform
    }
}
