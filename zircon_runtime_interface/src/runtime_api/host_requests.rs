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
    /// The host viewport that owns this IME session. Missing only decodes older
    /// serialized output; current runtime producers always set this target.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_viewport: Option<ZrRuntimeViewportHandle>,
    pub cursor_area: Option<ZrRuntimeImeCursorAreaV1>,
    pub surrounding_text: Option<ZrRuntimeImeSurroundingTextV1>,
}

impl ZrRuntimeImeHostRequestV1 {
    pub fn enable() -> Self {
        Self {
            kind: ZrRuntimeImeHostRequestKindV1::Enable,
            target_viewport: None,
            cursor_area: None,
            surrounding_text: None,
        }
    }

    pub fn disable() -> Self {
        Self {
            kind: ZrRuntimeImeHostRequestKindV1::Disable,
            target_viewport: None,
            cursor_area: None,
            surrounding_text: None,
        }
    }

    pub fn set_cursor_area(area: ZrRuntimeImeCursorAreaV1) -> Self {
        Self {
            kind: ZrRuntimeImeHostRequestKindV1::SetCursorArea,
            target_viewport: None,
            cursor_area: Some(area),
            surrounding_text: None,
        }
    }

    pub fn set_surrounding_text(text: ZrRuntimeImeSurroundingTextV1) -> Self {
        Self {
            kind: ZrRuntimeImeHostRequestKindV1::SetSurroundingText,
            target_viewport: None,
            cursor_area: None,
            surrounding_text: Some(text),
        }
    }

    pub const fn with_target_viewport(mut self, target_viewport: ZrRuntimeViewportHandle) -> Self {
        self.target_viewport = Some(target_viewport);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZrRuntimeImeHostRequestKindV1 {
    Enable,
    Disable,
    SetCursorArea,
    SetSurroundingText,
}

/// IME cursor rectangles are window-relative logical pixels. The app submits
/// these values to winit's logical position and size APIs without DPI scaling.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZrRuntimeImeCoordinateSpaceV1 {
    #[default]
    WindowLogical,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZrRuntimeImeCursorAreaV1 {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    #[serde(default)]
    pub coordinate_space: ZrRuntimeImeCoordinateSpaceV1,
}

impl ZrRuntimeImeCursorAreaV1 {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            coordinate_space: ZrRuntimeImeCoordinateSpaceV1::WindowLogical,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ime_host_request_keeps_its_viewport_and_logical_cursor_space() {
        let viewport = ZrRuntimeViewportHandle::new(7);
        let request = ZrRuntimeImeHostRequestV1::set_cursor_area(ZrRuntimeImeCursorAreaV1::new(
            12.0, 34.0, 2.0, 18.0,
        ))
        .with_target_viewport(viewport);

        assert_eq!(request.target_viewport, Some(viewport));
        assert_eq!(
            request.cursor_area.map(|area| area.coordinate_space),
            Some(ZrRuntimeImeCoordinateSpaceV1::WindowLogical)
        );

        let encoded = serde_json::to_value(&request).expect("serialize IME host request");
        assert_eq!(encoded["target_viewport"], serde_json::json!(7));
        assert_eq!(
            encoded["cursor_area"]["coordinate_space"],
            serde_json::json!("WindowLogical")
        );
    }

    #[test]
    fn ime_host_request_decodes_legacy_payload_without_a_viewport_target() {
        let legacy_request = serde_json::json!({
            "kind": "Enable",
            "cursor_area": null,
            "surrounding_text": null,
        });

        let request: ZrRuntimeImeHostRequestV1 =
            serde_json::from_value(legacy_request).expect("decode legacy IME host request");

        assert_eq!(request.target_viewport, None);
    }

    #[test]
    fn legacy_ime_cursor_area_defaults_to_window_logical_coordinates() {
        let legacy_area = serde_json::json!({
            "x": 12.0,
            "y": 34.0,
            "width": 2.0,
            "height": 18.0,
        });

        let area: ZrRuntimeImeCursorAreaV1 =
            serde_json::from_value(legacy_area).expect("decode legacy IME cursor area");

        assert_eq!(
            area.coordinate_space,
            ZrRuntimeImeCoordinateSpaceV1::WindowLogical
        );
    }
}
