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
pub type ZrRuntimeHostFetchFnV1 =
    unsafe extern "C" fn(ZrRuntimeHostFetchRequestV1, *mut ZrOwnedByteBuffer) -> ZrStatus;

pub const ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1: u32 = 1;
pub const ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1: u32 = 2;
pub const ZR_RUNTIME_EVENT_KIND_MOUSE_BUTTON_V1: u32 = 3;
pub const ZR_RUNTIME_EVENT_KIND_MOUSE_WHEEL_V1: u32 = 4;
pub const ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1: u32 = 5;
pub const ZR_RUNTIME_EVENT_KIND_TOUCH_V1: u32 = 6;
pub const ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1: u32 = 7;

pub const ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1: u32 = 1;
pub const ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1: u32 = 2;
pub const ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1: u32 = 3;

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
        Self {
            delta,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_MOUSE_WHEEL_V1, viewport)
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
