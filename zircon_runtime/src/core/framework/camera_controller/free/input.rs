use crate::core::math::{Real, Vec2, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FreeCameraInput {
    pub delta_seconds: Real,
    pub movement_axis: Vec3,
    pub look_delta: Vec2,
    pub scroll_lines: Real,
    pub run: bool,
    pub look_active: bool,
    pub focus_active: bool,
    pub cursor_grab_active: bool,
    pub cursor_grab_changed: bool,
}

impl Default for FreeCameraInput {
    fn default() -> Self {
        Self {
            delta_seconds: 0.0,
            movement_axis: Vec3::ZERO,
            look_delta: Vec2::ZERO,
            scroll_lines: 0.0,
            run: false,
            look_active: false,
            focus_active: true,
            cursor_grab_active: false,
            cursor_grab_changed: false,
        }
    }
}
