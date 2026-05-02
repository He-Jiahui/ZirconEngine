use crate::buffer::{ZrByteSlice, ZrOwnedByteBuffer};
use crate::handles::{ZrRuntimeSessionHandle, ZrRuntimeViewportHandle};
use crate::status::ZrStatus;

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

pub const ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1: u32 = 1;
pub const ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1: u32 = 2;
pub const ZR_RUNTIME_EVENT_KIND_MOUSE_BUTTON_V1: u32 = 3;
pub const ZR_RUNTIME_EVENT_KIND_MOUSE_WHEEL_V1: u32 = 4;

pub const ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1: u32 = 1;
pub const ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1: u32 = 2;
pub const ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1: u32 = 3;

pub const ZR_RUNTIME_BUTTON_STATE_PRESSED_V1: u32 = 1;
pub const ZR_RUNTIME_BUTTON_STATE_RELEASED_V1: u32 = 2;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZrHostApiV1 {
    pub abi_version: u32,
    pub size_bytes: usize,
    pub diagnostics_sink: Option<unsafe extern "C" fn(ZrByteSlice)>,
}

impl ZrHostApiV1 {
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            size_bytes: core::mem::size_of::<Self>(),
            diagnostics_sink: None,
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

impl ZrRuntimeViewportSizeV1 {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZrRuntimeEventV1 {
    pub abi_version: u32,
    pub kind: u32,
    pub viewport: ZrRuntimeViewportHandle,
    pub size: ZrRuntimeViewportSizeV1,
    pub x: f32,
    pub y: f32,
    pub delta: f32,
    pub button: u32,
    pub state: u32,
    pub payload: ZrByteSlice,
}

impl ZrRuntimeEventV1 {
    pub const fn new(abi_version: u32, kind: u32, viewport: ZrRuntimeViewportHandle) -> Self {
        Self {
            abi_version,
            kind,
            viewport,
            size: ZrRuntimeViewportSizeV1::new(0, 0),
            x: 0.0,
            y: 0.0,
            delta: 0.0,
            button: 0,
            state: 0,
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
        Self {
            delta,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_MOUSE_WHEEL_V1, viewport)
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
