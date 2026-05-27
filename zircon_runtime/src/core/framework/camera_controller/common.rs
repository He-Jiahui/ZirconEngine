use crate::core::math::{Quat, Transform, Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorGrabMode {
    Released,
    Confined,
    Locked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CursorGrabIntent {
    pub mode: CursorGrabMode,
    pub visible: bool,
    pub focused_only: bool,
}

impl CursorGrabIntent {
    pub const fn released() -> Self {
        Self {
            mode: CursorGrabMode::Released,
            visible: true,
            focused_only: false,
        }
    }

    pub const fn confined(focused_only: bool) -> Self {
        Self {
            mode: CursorGrabMode::Confined,
            visible: false,
            focused_only,
        }
    }

    pub const fn locked(focused_only: bool) -> Self {
        Self {
            mode: CursorGrabMode::Locked,
            visible: false,
            focused_only,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraControllerOutput {
    pub transform: Transform,
    pub translation_delta: Vec3,
    pub rotation_delta: Quat,
    pub scale_delta: Vec3,
    pub changed: bool,
    pub cursor_grab: Option<CursorGrabIntent>,
}

impl CameraControllerOutput {
    pub fn unchanged(transform: Transform) -> Self {
        Self {
            transform,
            translation_delta: Vec3::ZERO,
            rotation_delta: Quat::IDENTITY,
            scale_delta: Vec3::ZERO,
            changed: false,
            cursor_grab: None,
        }
    }

    pub fn from_transform(before: Transform, after: Transform) -> Self {
        Self {
            transform: after,
            translation_delta: after.translation - before.translation,
            rotation_delta: after.rotation * before.rotation.inverse(),
            scale_delta: after.scale - before.scale,
            changed: before != after,
            cursor_grab: None,
        }
    }

    pub fn with_cursor_grab(mut self, cursor_grab: CursorGrabIntent) -> Self {
        self.cursor_grab = Some(cursor_grab);
        self
    }
}
