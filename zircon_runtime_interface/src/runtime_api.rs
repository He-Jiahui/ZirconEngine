use crate::buffer::{ZrByteSlice, ZrOwnedByteBuffer};
use crate::handles::{ZrRuntimeSessionHandle, ZrRuntimeViewportHandle};
pub use crate::profiling::ZrRuntimeProfileControlFnV1;
use crate::status::ZrStatus;
use serde::{Deserialize, Serialize};

pub const ZR_RUNTIME_GET_API_SYMBOL_V1: &[u8] = b"zircon_runtime_get_api_v1\0";

pub type ZrRuntimeGetApiFnV1 = unsafe extern "C" fn(*const ZrHostApiV1) -> *const ZrRuntimeApiV1;
pub type ZrRuntimeCreateSessionFnV1 =
    unsafe extern "C" fn(ZrRuntimeSessionConfigV1, *mut ZrRuntimeSessionHandle) -> ZrStatus;
pub type ZrRuntimeDestroySessionFnV1 = unsafe extern "C" fn(ZrRuntimeSessionHandle) -> ZrStatus;
pub type ZrRuntimeHandleEventFnV1 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeEventV1) -> ZrStatus;
pub type ZrRuntimeCaptureFrameFnV1 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrRuntimeFrameRequestV1,
    *mut ZrRuntimeFrameV1,
) -> ZrStatus;
pub type ZrRuntimeCaptureAccessibilityTreeFnV1 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrRuntimeAccessibilityTreeRequestV1,
    *mut ZrOwnedByteBuffer,
) -> ZrStatus;
pub type ZrRuntimeBindViewportSurfaceFnV1 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeBindViewportSurfaceRequestV1) -> ZrStatus;
pub type ZrRuntimeUnbindViewportSurfaceFnV1 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeViewportHandle) -> ZrStatus;
pub type ZrRuntimePresentViewportFnV1 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeFrameRequestV1) -> ZrStatus;
pub type ZrRuntimeTickFrameFnV1 = unsafe extern "C" fn(ZrRuntimeSessionHandle) -> ZrStatus;
pub type ZrRuntimeDrainHostRequestsFnV1 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, *mut ZrOwnedByteBuffer) -> ZrStatus;
pub type ZrRuntimeHostFetchFnV1 =
    unsafe extern "C" fn(ZrRuntimeHostFetchRequestV1, *mut ZrOwnedByteBuffer) -> ZrStatus;

pub const ZR_RUNTIME_NATIVE_SURFACE_KIND_NONE_V1: u32 = 0;
pub const ZR_RUNTIME_NATIVE_SURFACE_KIND_WIN32_V1: u32 = 1;

pub const ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1: u32 = 1;
pub const ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1: u32 = 2;
pub const ZR_RUNTIME_EVENT_KIND_MOUSE_BUTTON_V1: u32 = 3;
pub const ZR_RUNTIME_EVENT_KIND_MOUSE_WHEEL_V1: u32 = 4;
pub const ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1: u32 = 5;
pub const ZR_RUNTIME_EVENT_KIND_TOUCH_V1: u32 = 6;
pub const ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1: u32 = 7;
pub const ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1: u32 = 8;
pub const ZR_RUNTIME_EVENT_KIND_GAMEPAD_CONNECTION_V1: u32 = 9;
pub const ZR_RUNTIME_EVENT_KIND_GAMEPAD_BUTTON_V1: u32 = 10;
pub const ZR_RUNTIME_EVENT_KIND_GAMEPAD_AXIS_V1: u32 = 11;
pub const ZR_RUNTIME_EVENT_KIND_MOUSE_MOTION_V1: u32 = 12;
pub const ZR_RUNTIME_EVENT_KIND_IME_V1: u32 = 13;
pub const ZR_RUNTIME_EVENT_KIND_CURSOR_ENTERED_V1: u32 = 14;
pub const ZR_RUNTIME_EVENT_KIND_CURSOR_LEFT_V1: u32 = 15;
pub const ZR_RUNTIME_EVENT_KIND_FILE_DRAG_DROP_V1: u32 = 16;
pub const ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1: u32 = 17;

pub const ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1: u32 = 1;
pub const ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1: u32 = 2;
pub const ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1: u32 = 3;

pub const ZR_RUNTIME_MOUSE_WHEEL_UNIT_LINE_V1: u32 = 1;
pub const ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1: u32 = 2;

pub const ZR_RUNTIME_BUTTON_STATE_PRESSED_V1: u32 = 1;
pub const ZR_RUNTIME_BUTTON_STATE_RELEASED_V1: u32 = 2;

pub const ZR_RUNTIME_LIFECYCLE_STATE_FOREGROUND_V1: u32 = 1;
pub const ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1: u32 = 2;
pub const ZR_RUNTIME_LIFECYCLE_STATE_SUSPENDED_V1: u32 = 3;
pub const ZR_RUNTIME_LIFECYCLE_STATE_RESUMED_V1: u32 = 4;
pub const ZR_RUNTIME_LIFECYCLE_STATE_LOW_MEMORY_V1: u32 = 5;

pub const ZR_RUNTIME_TOUCH_PHASE_STARTED_V1: u32 = 1;
pub const ZR_RUNTIME_TOUCH_PHASE_MOVED_V1: u32 = 2;
pub const ZR_RUNTIME_TOUCH_PHASE_ENDED_V1: u32 = 3;
pub const ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1: u32 = 4;

pub const ZR_RUNTIME_KEY_ACTION_PRESSED_V1: u32 = 1;
pub const ZR_RUNTIME_KEY_ACTION_RELEASED_V1: u32 = 2;
pub const ZR_RUNTIME_KEY_ACTION_TEXT_V1: u32 = 3;

pub const ZR_RUNTIME_IME_STATE_ENABLED_V1: u32 = 1;
pub const ZR_RUNTIME_IME_STATE_DISABLED_V1: u32 = 2;
pub const ZR_RUNTIME_IME_STATE_PREEDIT_V1: u32 = 3;
pub const ZR_RUNTIME_IME_STATE_COMMIT_V1: u32 = 4;
pub const ZR_RUNTIME_IME_STATE_DELETE_SURROUNDING_V1: u32 = 5;
pub const ZR_RUNTIME_IME_STATE_REQUEST_ENABLE_V1: u32 = 6;
pub const ZR_RUNTIME_IME_STATE_REQUEST_DISABLE_V1: u32 = 7;
pub const ZR_RUNTIME_IME_STATE_CURSOR_AREA_V1: u32 = 8;
pub const ZR_RUNTIME_IME_STATE_SURROUNDING_TEXT_V1: u32 = 9;
pub const ZR_RUNTIME_IME_CURSOR_HIDDEN_V1: u32 = u32::MAX;

pub const ZR_RUNTIME_FILE_DRAG_HOVERED_V1: u32 = 1;
pub const ZR_RUNTIME_FILE_DRAG_DROPPED_V1: u32 = 2;
pub const ZR_RUNTIME_FILE_DRAG_CANCELLED_V1: u32 = 3;

pub const ZR_RUNTIME_WINDOW_STATUS_MOVED_V1: u32 = 1;
pub const ZR_RUNTIME_WINDOW_STATUS_OCCLUDED_V1: u32 = 2;
pub const ZR_RUNTIME_WINDOW_STATUS_THEME_CHANGED_V1: u32 = 3;
pub const ZR_RUNTIME_WINDOW_STATUS_CLOSE_REQUESTED_V1: u32 = 4;
pub const ZR_RUNTIME_WINDOW_STATUS_DESTROYED_V1: u32 = 5;
pub const ZR_RUNTIME_WINDOW_STATUS_SCALE_FACTOR_CHANGED_V1: u32 = 6;
pub const ZR_RUNTIME_WINDOW_STATUS_BACKEND_SCALE_FACTOR_CHANGED_V1: u32 = 7;
pub const ZR_RUNTIME_WINDOW_BOOL_FALSE_V1: u32 = 0;
pub const ZR_RUNTIME_WINDOW_BOOL_TRUE_V1: u32 = 1;
pub const ZR_RUNTIME_WINDOW_THEME_UNKNOWN_V1: u32 = 0;
pub const ZR_RUNTIME_WINDOW_THEME_LIGHT_V1: u32 = 1;
pub const ZR_RUNTIME_WINDOW_THEME_DARK_V1: u32 = 2;

pub const ZR_RUNTIME_GAMEPAD_CONNECTION_CONNECTED_V1: u32 = 1;
pub const ZR_RUNTIME_GAMEPAD_CONNECTION_DISCONNECTED_V1: u32 = 2;

pub const ZR_RUNTIME_GAMEPAD_BUTTON_UNKNOWN_V1: u32 = 0;
pub const ZR_RUNTIME_GAMEPAD_BUTTON_SOUTH_V1: u32 = 1;
pub const ZR_RUNTIME_GAMEPAD_BUTTON_EAST_V1: u32 = 2;
pub const ZR_RUNTIME_GAMEPAD_BUTTON_NORTH_V1: u32 = 3;
pub const ZR_RUNTIME_GAMEPAD_BUTTON_WEST_V1: u32 = 4;
pub const ZR_RUNTIME_GAMEPAD_BUTTON_C_V1: u32 = 5;
pub const ZR_RUNTIME_GAMEPAD_BUTTON_Z_V1: u32 = 6;
pub const ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_TRIGGER_V1: u32 = 7;
pub const ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_TRIGGER2_V1: u32 = 8;
pub const ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_TRIGGER_V1: u32 = 9;
pub const ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_TRIGGER2_V1: u32 = 10;
pub const ZR_RUNTIME_GAMEPAD_BUTTON_SELECT_V1: u32 = 11;
pub const ZR_RUNTIME_GAMEPAD_BUTTON_START_V1: u32 = 12;
pub const ZR_RUNTIME_GAMEPAD_BUTTON_MODE_V1: u32 = 13;
pub const ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_THUMB_V1: u32 = 14;
pub const ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_THUMB_V1: u32 = 15;
pub const ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_UP_V1: u32 = 16;
pub const ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_DOWN_V1: u32 = 17;
pub const ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_LEFT_V1: u32 = 18;
pub const ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_RIGHT_V1: u32 = 19;

pub const ZR_RUNTIME_GAMEPAD_AXIS_UNKNOWN_V1: u32 = 0;
pub const ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_X_V1: u32 = 1;
pub const ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_Y_V1: u32 = 2;
pub const ZR_RUNTIME_GAMEPAD_AXIS_LEFT_Z_V1: u32 = 3;
pub const ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_X_V1: u32 = 4;
pub const ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_Y_V1: u32 = 5;
pub const ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_Z_V1: u32 = 6;
pub const ZR_RUNTIME_GAMEPAD_AXIS_DPAD_X_V1: u32 = 7;
pub const ZR_RUNTIME_GAMEPAD_AXIS_DPAD_Y_V1: u32 = 8;

pub const ZR_RUNTIME_FETCH_FLAG_STREAMING_V1: u32 = 1 << 0;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZrHostApiV1 {
    pub abi_version: u32,
    pub size_bytes: usize,
    pub diagnostics_sink: Option<unsafe extern "C" fn(ZrByteSlice)>,
    pub fetch_resource: Option<ZrRuntimeHostFetchFnV1>,
}

impl ZrHostApiV1 {
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            size_bytes: core::mem::size_of::<Self>(),
            diagnostics_sink: None,
            fetch_resource: None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZrRuntimeApiV1 {
    pub abi_version: u32,
    pub size_bytes: usize,
    pub create_session: Option<ZrRuntimeCreateSessionFnV1>,
    pub destroy_session: Option<ZrRuntimeDestroySessionFnV1>,
    pub handle_event: Option<ZrRuntimeHandleEventFnV1>,
    pub capture_frame: Option<ZrRuntimeCaptureFrameFnV1>,
    pub capture_accessibility_tree: Option<ZrRuntimeCaptureAccessibilityTreeFnV1>,
    pub bind_viewport_surface: Option<ZrRuntimeBindViewportSurfaceFnV1>,
    pub unbind_viewport_surface: Option<ZrRuntimeUnbindViewportSurfaceFnV1>,
    pub present_viewport: Option<ZrRuntimePresentViewportFnV1>,
    pub profile_control: Option<ZrRuntimeProfileControlFnV1>,
    pub tick_frame: Option<ZrRuntimeTickFrameFnV1>,
    pub drain_host_requests: Option<ZrRuntimeDrainHostRequestsFnV1>,
}

impl ZrRuntimeApiV1 {
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            size_bytes: core::mem::size_of::<Self>(),
            create_session: None,
            destroy_session: None,
            handle_event: None,
            capture_frame: None,
            capture_accessibility_tree: None,
            bind_viewport_surface: None,
            unbind_viewport_surface: None,
            present_viewport: None,
            profile_control: None,
            tick_frame: None,
            drain_host_requests: None,
        }
    }
}

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
}

impl ZrRuntimeHostRequestV1 {
    pub fn ime(request: ZrRuntimeImeHostRequestV1) -> Self {
        Self::Ime(request)
    }

    pub fn gamepad_rumble(request: ZrRuntimeGamepadRumbleRequestV1) -> Self {
        Self::GamepadRumble(request)
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

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeSessionConfigV1 {
    pub abi_version: u32,
    pub profile: ZrByteSlice,
    pub project_manifest: ZrByteSlice,
}

impl ZrRuntimeSessionConfigV1 {
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            profile: ZrByteSlice::empty(),
            project_manifest: ZrByteSlice::empty(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeViewportSizeV1 {
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZrRuntimeViewportMetricsV1 {
    pub logical_size: ZrRuntimeViewportSizeV1,
    pub device_scale_factor: f32,
    pub physical_size: ZrRuntimeViewportSizeV1,
}

impl ZrRuntimeViewportMetricsV1 {
    pub const fn new(
        logical_size: ZrRuntimeViewportSizeV1,
        device_scale_factor: f32,
        physical_size: ZrRuntimeViewportSizeV1,
    ) -> Self {
        Self {
            logical_size,
            device_scale_factor,
            physical_size,
        }
    }
}

impl ZrRuntimeViewportSizeV1 {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeNativeSurfaceTargetV1 {
    pub abi_version: u32,
    pub kind: u32,
    pub window_handle: u64,
    pub display_handle: u64,
}

impl ZrRuntimeNativeSurfaceTargetV1 {
    pub const fn none(abi_version: u32) -> Self {
        Self {
            abi_version,
            kind: ZR_RUNTIME_NATIVE_SURFACE_KIND_NONE_V1,
            window_handle: 0,
            display_handle: 0,
        }
    }

    pub const fn win32(abi_version: u32, hwnd: u64, hinstance: u64) -> Self {
        Self {
            abi_version,
            kind: ZR_RUNTIME_NATIVE_SURFACE_KIND_WIN32_V1,
            window_handle: hwnd,
            display_handle: hinstance,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeBindViewportSurfaceRequestV1 {
    pub abi_version: u32,
    pub viewport: ZrRuntimeViewportHandle,
    pub size: ZrRuntimeViewportSizeV1,
    pub target: ZrRuntimeNativeSurfaceTargetV1,
}

impl ZrRuntimeBindViewportSurfaceRequestV1 {
    pub const fn new(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
        target: ZrRuntimeNativeSurfaceTargetV1,
    ) -> Self {
        Self {
            abi_version,
            viewport,
            size,
            target,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZrRuntimeEventV1 {
    pub abi_version: u32,
    pub kind: u32,
    pub viewport: ZrRuntimeViewportHandle,
    pub size: ZrRuntimeViewportSizeV1,
    pub metrics: ZrRuntimeViewportMetricsV1,
    pub x: f32,
    pub y: f32,
    pub delta: f32,
    pub button: u32,
    pub state: u32,
    pub pointer_id: u64,
    pub key_code: u32,
    pub scan_code: u32,
    pub payload: ZrByteSlice,
}

impl ZrRuntimeEventV1 {
    pub const fn new(abi_version: u32, kind: u32, viewport: ZrRuntimeViewportHandle) -> Self {
        Self {
            abi_version,
            kind,
            viewport,
            size: ZrRuntimeViewportSizeV1::new(0, 0),
            metrics: ZrRuntimeViewportMetricsV1::new(
                ZrRuntimeViewportSizeV1::new(0, 0),
                1.0,
                ZrRuntimeViewportSizeV1::new(0, 0),
            ),
            x: 0.0,
            y: 0.0,
            delta: 0.0,
            button: 0,
            state: 0,
            pointer_id: 0,
            key_code: 0,
            scan_code: 0,
            payload: ZrByteSlice::empty(),
        }
    }

    pub const fn viewport_resized(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
    ) -> Self {
        Self {
            size,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1,
                viewport,
            )
        }
    }

    pub const fn viewport_metrics(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        metrics: ZrRuntimeViewportMetricsV1,
    ) -> Self {
        Self {
            size: metrics.physical_size,
            metrics,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1,
                viewport,
            )
        }
    }

    pub const fn pointer_moved(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        x: f32,
        y: f32,
    ) -> Self {
        Self {
            x,
            y,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1,
                viewport,
            )
        }
    }

    pub const fn mouse_button(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        button: u32,
        state: u32,
        x: f32,
        y: f32,
    ) -> Self {
        Self {
            x,
            y,
            button,
            state,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_MOUSE_BUTTON_V1, viewport)
        }
    }

    pub const fn mouse_wheel(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        delta: f32,
    ) -> Self {
        Self::mouse_wheel_delta(
            abi_version,
            viewport,
            ZR_RUNTIME_MOUSE_WHEEL_UNIT_LINE_V1,
            0.0,
            delta,
        )
    }

    pub const fn mouse_wheel_delta(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        unit: u32,
        x: f32,
        y: f32,
    ) -> Self {
        Self {
            x,
            y,
            delta: y,
            state: unit,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_MOUSE_WHEEL_V1, viewport)
        }
    }

    pub const fn cursor_entered(abi_version: u32, viewport: ZrRuntimeViewportHandle) -> Self {
        Self::new(
            abi_version,
            ZR_RUNTIME_EVENT_KIND_CURSOR_ENTERED_V1,
            viewport,
        )
    }

    pub const fn cursor_left(abi_version: u32, viewport: ZrRuntimeViewportHandle) -> Self {
        Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_CURSOR_LEFT_V1, viewport)
    }

    pub const fn file_hovered(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        path: ZrByteSlice,
    ) -> Self {
        Self {
            state: ZR_RUNTIME_FILE_DRAG_HOVERED_V1,
            payload: path,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_FILE_DRAG_DROP_V1,
                viewport,
            )
        }
    }

    pub const fn file_dropped(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        path: ZrByteSlice,
    ) -> Self {
        Self {
            state: ZR_RUNTIME_FILE_DRAG_DROPPED_V1,
            payload: path,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_FILE_DRAG_DROP_V1,
                viewport,
            )
        }
    }

    pub const fn file_drag_cancelled(abi_version: u32, viewport: ZrRuntimeViewportHandle) -> Self {
        Self {
            state: ZR_RUNTIME_FILE_DRAG_CANCELLED_V1,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_FILE_DRAG_DROP_V1,
                viewport,
            )
        }
    }

    pub const fn window_moved(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        x: i32,
        y: i32,
    ) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
            state: ZR_RUNTIME_WINDOW_STATUS_MOVED_V1,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1,
                viewport,
            )
        }
    }

    pub const fn window_occluded(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        occluded: bool,
    ) -> Self {
        Self {
            button: if occluded {
                ZR_RUNTIME_WINDOW_BOOL_TRUE_V1
            } else {
                ZR_RUNTIME_WINDOW_BOOL_FALSE_V1
            },
            state: ZR_RUNTIME_WINDOW_STATUS_OCCLUDED_V1,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1,
                viewport,
            )
        }
    }

    pub const fn window_theme_changed(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        theme: u32,
    ) -> Self {
        Self {
            button: theme,
            state: ZR_RUNTIME_WINDOW_STATUS_THEME_CHANGED_V1,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1,
                viewport,
            )
        }
    }

    pub const fn window_scale_factor_changed(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        scale_factor: f32,
    ) -> Self {
        Self {
            delta: scale_factor,
            state: ZR_RUNTIME_WINDOW_STATUS_SCALE_FACTOR_CHANGED_V1,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1,
                viewport,
            )
        }
    }

    pub const fn window_backend_scale_factor_changed(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        scale_factor: f32,
    ) -> Self {
        Self {
            delta: scale_factor,
            state: ZR_RUNTIME_WINDOW_STATUS_BACKEND_SCALE_FACTOR_CHANGED_V1,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1,
                viewport,
            )
        }
    }

    pub const fn window_close_requested(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
    ) -> Self {
        Self {
            state: ZR_RUNTIME_WINDOW_STATUS_CLOSE_REQUESTED_V1,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1,
                viewport,
            )
        }
    }

    pub const fn window_destroyed(abi_version: u32, viewport: ZrRuntimeViewportHandle) -> Self {
        Self {
            state: ZR_RUNTIME_WINDOW_STATUS_DESTROYED_V1,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1,
                viewport,
            )
        }
    }

    pub const fn mouse_motion(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        delta_x: f32,
        delta_y: f32,
    ) -> Self {
        Self {
            x: delta_x,
            y: delta_y,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_MOUSE_MOTION_V1, viewport)
        }
    }

    pub const fn lifecycle(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        state: u32,
    ) -> Self {
        Self {
            state,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1, viewport)
        }
    }

    pub const fn touch(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        pointer_id: u64,
        phase: u32,
        x: f32,
        y: f32,
    ) -> Self {
        Self {
            x,
            y,
            state: phase,
            pointer_id,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_TOUCH_V1, viewport)
        }
    }

    pub const fn keyboard(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        action: u32,
        key_code: u32,
        scan_code: u32,
        key_text: ZrByteSlice,
    ) -> Self {
        Self {
            button: action,
            key_code,
            scan_code,
            payload: key_text,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1, viewport)
        }
    }

    pub const fn ime_enabled(abi_version: u32, viewport: ZrRuntimeViewportHandle) -> Self {
        Self {
            state: ZR_RUNTIME_IME_STATE_ENABLED_V1,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_IME_V1, viewport)
        }
    }

    pub const fn ime_disabled(abi_version: u32, viewport: ZrRuntimeViewportHandle) -> Self {
        Self {
            state: ZR_RUNTIME_IME_STATE_DISABLED_V1,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_IME_V1, viewport)
        }
    }

    pub const fn ime_preedit(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        value: ZrByteSlice,
        cursor_start: u32,
        cursor_end: u32,
    ) -> Self {
        Self {
            state: ZR_RUNTIME_IME_STATE_PREEDIT_V1,
            payload: value,
            key_code: cursor_start,
            scan_code: cursor_end,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_IME_V1, viewport)
        }
    }

    pub const fn ime_commit(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        value: ZrByteSlice,
    ) -> Self {
        Self {
            state: ZR_RUNTIME_IME_STATE_COMMIT_V1,
            payload: value,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_IME_V1, viewport)
        }
    }

    pub const fn ime_delete_surrounding(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        before_bytes: u32,
        after_bytes: u32,
    ) -> Self {
        Self {
            state: ZR_RUNTIME_IME_STATE_DELETE_SURROUNDING_V1,
            key_code: before_bytes,
            scan_code: after_bytes,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_IME_V1, viewport)
        }
    }

    pub const fn ime_request_enable(abi_version: u32, viewport: ZrRuntimeViewportHandle) -> Self {
        Self {
            state: ZR_RUNTIME_IME_STATE_REQUEST_ENABLE_V1,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_IME_V1, viewport)
        }
    }

    pub const fn ime_request_disable(abi_version: u32, viewport: ZrRuntimeViewportHandle) -> Self {
        Self {
            state: ZR_RUNTIME_IME_STATE_REQUEST_DISABLE_V1,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_IME_V1, viewport)
        }
    }

    pub const fn ime_cursor_area(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        x: f32,
        y: f32,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            x,
            y,
            size: ZrRuntimeViewportSizeV1::new(width, height),
            state: ZR_RUNTIME_IME_STATE_CURSOR_AREA_V1,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_IME_V1, viewport)
        }
    }

    pub const fn ime_surrounding_text(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        value: ZrByteSlice,
        cursor: u32,
        anchor: u32,
    ) -> Self {
        Self {
            state: ZR_RUNTIME_IME_STATE_SURROUNDING_TEXT_V1,
            payload: value,
            key_code: cursor,
            scan_code: anchor,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_IME_V1, viewport)
        }
    }

    pub const fn accessibility_action(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        payload: ZrByteSlice,
    ) -> Self {
        Self {
            payload,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1,
                viewport,
            )
        }
    }

    pub const fn gamepad_connection(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        gamepad_id: u64,
        state: u32,
        name: ZrByteSlice,
    ) -> Self {
        Self::gamepad_connection_with_ids(abi_version, viewport, gamepad_id, state, 0, 0, name)
    }

    pub const fn gamepad_connection_with_ids(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        gamepad_id: u64,
        state: u32,
        vendor_id: u32,
        product_id: u32,
        name: ZrByteSlice,
    ) -> Self {
        Self {
            state,
            pointer_id: gamepad_id,
            key_code: vendor_id,
            scan_code: product_id,
            payload: name,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_GAMEPAD_CONNECTION_V1,
                viewport,
            )
        }
    }

    pub const fn gamepad_button(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        gamepad_id: u64,
        button: u32,
        state: u32,
        value: f32,
    ) -> Self {
        Self {
            button,
            state,
            delta: value,
            pointer_id: gamepad_id,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_GAMEPAD_BUTTON_V1,
                viewport,
            )
        }
    }

    pub const fn gamepad_axis(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        gamepad_id: u64,
        axis: u32,
        value: f32,
    ) -> Self {
        Self {
            button: axis,
            delta: value,
            pointer_id: gamepad_id,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_GAMEPAD_AXIS_V1, viewport)
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZrRuntimeTranslatedEventV1 {
    pub abi_version: u32,
    pub event: ZrRuntimeEventV1,
    pub host_reason: ZrByteSlice,
}

impl ZrRuntimeTranslatedEventV1 {
    pub const fn new(abi_version: u32, event: ZrRuntimeEventV1, host_reason: ZrByteSlice) -> Self {
        Self {
            abi_version,
            event,
            host_reason,
        }
    }

    pub const fn viewport_metrics(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        metrics: ZrRuntimeViewportMetricsV1,
    ) -> Self {
        Self::new(
            abi_version,
            ZrRuntimeEventV1::viewport_metrics(abi_version, viewport, metrics),
            ZrByteSlice::from_static(b"viewport_metrics"),
        )
    }

    pub const fn touch_moved(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        pointer_id: u64,
        x: f32,
        y: f32,
    ) -> Self {
        Self::new(
            abi_version,
            ZrRuntimeEventV1::touch(
                abi_version,
                viewport,
                pointer_id,
                ZR_RUNTIME_TOUCH_PHASE_MOVED_V1,
                x,
                y,
            ),
            ZrByteSlice::from_static(b"touch_moved"),
        )
    }

    pub const fn keyboard_text(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        key_text: ZrByteSlice,
    ) -> Self {
        Self::new(
            abi_version,
            ZrRuntimeEventV1::keyboard(
                abi_version,
                viewport,
                ZR_RUNTIME_KEY_ACTION_TEXT_V1,
                0,
                0,
                key_text,
            ),
            ZrByteSlice::from_static(b"keyboard_text"),
        )
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeHostFetchRequestV1 {
    pub abi_version: u32,
    pub uri: ZrByteSlice,
    pub flags: u32,
}

impl ZrRuntimeHostFetchRequestV1 {
    pub const fn new(abi_version: u32, uri: ZrByteSlice, flags: u32) -> Self {
        Self {
            abi_version,
            uri,
            flags,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeFrameRequestV1 {
    pub abi_version: u32,
    pub viewport: ZrRuntimeViewportHandle,
    pub size: ZrRuntimeViewportSizeV1,
}

impl ZrRuntimeFrameRequestV1 {
    pub const fn new(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
    ) -> Self {
        Self {
            abi_version,
            viewport,
            size,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeAccessibilityTreeRequestV1 {
    pub abi_version: u32,
    pub viewport: ZrRuntimeViewportHandle,
    pub size: ZrRuntimeViewportSizeV1,
    pub generation_hint: u64,
}

impl ZrRuntimeAccessibilityTreeRequestV1 {
    pub const fn new(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
        generation_hint: u64,
    ) -> Self {
        Self {
            abi_version,
            viewport,
            size,
            generation_hint,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZrRuntimeFrameV1 {
    pub abi_version: u32,
    pub width: u32,
    pub height: u32,
    pub generation: u64,
    pub rgba: ZrOwnedByteBuffer,
}

impl ZrRuntimeFrameV1 {
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            width: 0,
            height: 0,
            generation: 0,
            rgba: ZrOwnedByteBuffer::empty(),
        }
    }

    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0 || self.rgba.is_empty()
    }
}
