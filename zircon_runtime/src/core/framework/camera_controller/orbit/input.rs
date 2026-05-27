use crate::core::math::{clamp_viewport_size, Real, UVec2, Vec2, Vec3};

use super::OrbitCameraAction;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrbitCameraInput {
    pub action: OrbitCameraAction,
    pub viewport_size: UVec2,
    pub focus_active: bool,
}

impl OrbitCameraInput {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn orbit(previous: Vec2, current: Vec2) -> Self {
        Self {
            action: OrbitCameraAction::Orbit { previous, current },
            ..Self::default()
        }
    }

    pub fn pan(previous: Vec2, current: Vec2) -> Self {
        Self {
            action: OrbitCameraAction::Pan { previous, current },
            ..Self::default()
        }
    }

    pub fn zoom(delta: Real) -> Self {
        Self {
            action: OrbitCameraAction::Zoom { delta },
            ..Self::default()
        }
    }

    pub fn focus(target: Vec3) -> Self {
        Self {
            action: OrbitCameraAction::Focus { target },
            ..Self::default()
        }
    }

    pub fn with_viewport_size(mut self, viewport_size: UVec2) -> Self {
        self.viewport_size = clamp_viewport_size(viewport_size);
        self
    }

    pub fn with_focus_active(mut self, focus_active: bool) -> Self {
        self.focus_active = focus_active;
        self
    }
}

impl Default for OrbitCameraInput {
    fn default() -> Self {
        Self {
            action: OrbitCameraAction::None,
            viewport_size: UVec2::ONE,
            focus_active: true,
        }
    }
}
