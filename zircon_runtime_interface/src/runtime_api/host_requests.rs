use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZrRuntimeHostRequestBatchV1 {
    pub abi_version: u32,
    pub requests: Vec<ZrRuntimeHostRequestV1>,
}

impl ZrRuntimeHostRequestBatchV1 {
    pub fn new(abi_version: u32, requests: Vec<ZrRuntimeHostRequestV1>) -> Self {
        Self {
            abi_version,
            requests,
        }
    }

    pub fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            requests: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ZrRuntimeHostRequestV1 {
    Ime(ZrRuntimeImeHostRequestV1),
    GamepadRumble(ZrRuntimeGamepadRumbleRequestV1),
    Cursor(ZrRuntimeCursorHostRequestV1),
}

impl ZrRuntimeHostRequestV1 {
    pub fn ime(request: ZrRuntimeImeHostRequestV1) -> Self {
        Self::Ime(request)
    }

    pub fn gamepad_rumble(request: ZrRuntimeGamepadRumbleRequestV1) -> Self {
        Self::GamepadRumble(request)
    }

    pub fn cursor(request: ZrRuntimeCursorHostRequestV1) -> Self {
        Self::Cursor(request)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZrRuntimeCursorHostRequestV1 {
    pub kind: ZrRuntimeCursorHostRequestKindV1,
    pub grab_mode: Option<ZrRuntimeCursorGrabModeV1>,
    pub position: Option<ZrRuntimeCursorPositionV1>,
    pub value: bool,
}

impl ZrRuntimeCursorHostRequestV1 {
    pub const fn set_visible(visible: bool) -> Self {
        Self {
            kind: ZrRuntimeCursorHostRequestKindV1::SetVisible,
            grab_mode: None,
            position: None,
            value: visible,
        }
    }

    pub const fn set_grab_mode(grab_mode: ZrRuntimeCursorGrabModeV1) -> Self {
        Self {
            kind: ZrRuntimeCursorHostRequestKindV1::SetGrabMode,
            grab_mode: Some(grab_mode),
            position: None,
            value: false,
        }
    }

    pub const fn set_hit_test(hit_test: bool) -> Self {
        Self {
            kind: ZrRuntimeCursorHostRequestKindV1::SetHitTest,
            grab_mode: None,
            position: None,
            value: hit_test,
        }
    }

    pub const fn set_position(position: ZrRuntimeCursorPositionV1) -> Self {
        Self {
            kind: ZrRuntimeCursorHostRequestKindV1::SetPosition,
            grab_mode: None,
            position: Some(position),
            value: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZrRuntimeCursorHostRequestKindV1 {
    SetVisible,
    SetGrabMode,
    SetHitTest,
    SetPosition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZrRuntimeCursorGrabModeV1 {
    None,
    Confined,
    Locked,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZrRuntimeCursorPositionV1 {
    pub x: f32,
    pub y: f32,
}

impl ZrRuntimeCursorPositionV1 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZrRuntimeGamepadRumbleRequestV1 {
    pub gamepad_id: u64,
    pub kind: ZrRuntimeGamepadRumbleRequestKindV1,
    pub strong_motor: f32,
    pub weak_motor: f32,
    pub duration_millis: u32,
}

impl ZrRuntimeGamepadRumbleRequestV1 {
    pub const fn add(
        gamepad_id: u64,
        strong_motor: f32,
        weak_motor: f32,
        duration_millis: u32,
    ) -> Self {
        Self {
            gamepad_id,
            kind: ZrRuntimeGamepadRumbleRequestKindV1::Add,
            strong_motor,
            weak_motor,
            duration_millis,
        }
    }

    pub const fn stop(gamepad_id: u64) -> Self {
        Self {
            gamepad_id,
            kind: ZrRuntimeGamepadRumbleRequestKindV1::Stop,
            strong_motor: 0.0,
            weak_motor: 0.0,
            duration_millis: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZrRuntimeGamepadRumbleRequestKindV1 {
    Add,
    Stop,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZrRuntimeImeHostRequestV1 {
    pub kind: ZrRuntimeImeHostRequestKindV1,
    pub cursor_area: Option<ZrRuntimeImeCursorAreaV1>,
    pub surrounding_text: Option<ZrRuntimeImeSurroundingTextV1>,
}

impl ZrRuntimeImeHostRequestV1 {
    pub fn enable() -> Self {
        Self {
            kind: ZrRuntimeImeHostRequestKindV1::Enable,
            cursor_area: None,
            surrounding_text: None,
        }
    }

    pub fn disable() -> Self {
        Self {
            kind: ZrRuntimeImeHostRequestKindV1::Disable,
            cursor_area: None,
            surrounding_text: None,
        }
    }

    pub fn set_cursor_area(area: ZrRuntimeImeCursorAreaV1) -> Self {
        Self {
            kind: ZrRuntimeImeHostRequestKindV1::SetCursorArea,
            cursor_area: Some(area),
            surrounding_text: None,
        }
    }

    pub fn set_surrounding_text(text: ZrRuntimeImeSurroundingTextV1) -> Self {
        Self {
            kind: ZrRuntimeImeHostRequestKindV1::SetSurroundingText,
            cursor_area: None,
            surrounding_text: Some(text),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZrRuntimeImeHostRequestKindV1 {
    Enable,
    Disable,
    SetCursorArea,
    SetSurroundingText,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZrRuntimeImeCursorAreaV1 {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ZrRuntimeImeCursorAreaV1 {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZrRuntimeImeSurroundingTextV1 {
    pub value: String,
    pub cursor: usize,
    pub anchor: usize,
}

impl ZrRuntimeImeSurroundingTextV1 {
    pub fn new(value: impl Into<String>, cursor: usize, anchor: usize) -> Self {
        Self {
            value: value.into(),
            cursor,
            anchor,
        }
    }
}
