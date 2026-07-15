use crate::buffer::{ZrByteSlice, ZrOwnedByteBuffer};
use crate::handles::{ZrRuntimeSessionHandle, ZrRuntimeViewportHandle};
pub use crate::profiling::ZrRuntimeProfileControlFnV1;
use crate::status::ZrStatus;

use super::{
    ZrRuntimeAccessibilityTreeRequestV1, ZrRuntimeBindViewportSurfaceRequestV1,
    ZrRuntimeDrainPluginEventsFnV1, ZrRuntimeEventV1, ZrRuntimeFrameRequestV1, ZrRuntimeFrameV1,
    ZrRuntimeHarvestOperationFnV1, ZrRuntimeHostFetchRequestV1, ZrRuntimePollOperationFnV1,
    ZrRuntimeSubmitOperationFnV1, ZrRuntimeSubscribePluginEventFnV1,
    ZrRuntimeUnsubscribePluginEventFnV1,
};

pub const ZR_RUNTIME_GET_API_SYMBOL_V2: &[u8] = b"zircon_runtime_get_api_v2\0";

pub type ZrRuntimeGetApiFnV2 = unsafe extern "C" fn(*const ZrHostApiV1) -> *const ZrRuntimeApiV2;
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

/// The immutable runtime API V2 table.
///
/// This shape is frozen. Any future field addition requires a new table
/// version and a coordinated hard cutover of all dynamic hosts.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZrRuntimeApiV2 {
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
    pub subscribe_plugin_event: Option<ZrRuntimeSubscribePluginEventFnV1>,
    pub unsubscribe_plugin_event: Option<ZrRuntimeUnsubscribePluginEventFnV1>,
    pub drain_plugin_events: Option<ZrRuntimeDrainPluginEventsFnV1>,
    pub submit_operation: Option<ZrRuntimeSubmitOperationFnV1>,
    pub poll_operation: Option<ZrRuntimePollOperationFnV1>,
    pub harvest_operation: Option<ZrRuntimeHarvestOperationFnV1>,
}

impl ZrRuntimeApiV2 {
    pub const fn empty() -> Self {
        Self {
            abi_version: crate::version::ZIRCON_RUNTIME_API_VERSION_V2,
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
            subscribe_plugin_event: None,
            unsubscribe_plugin_event: None,
            drain_plugin_events: None,
            submit_operation: None,
            poll_operation: None,
            harvest_operation: None,
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
