use crate::buffer::{ZrByteSlice, ZrOwnedByteBuffer};
use crate::handles::{ZrRuntimeSessionHandle, ZrRuntimeViewportHandle};
pub use crate::profiling::ZrRuntimeProfileControlFnV1;
use crate::status::ZrStatus;

use super::{
    ZrRuntimeAccessibilityTreeRequestV1, ZrRuntimeBindViewportSurfaceRequestV1, ZrRuntimeEventV1,
    ZrRuntimeFrameRequestV1, ZrRuntimeFrameV1, ZrRuntimeHostFetchRequestV1,
};

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
