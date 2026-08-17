use crate::buffer::{ZrByteSlice, ZrOwnedByteBuffer, ZrOwnedResultV2};
use crate::handles::{ZrRuntimeAllocationId, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle};
pub use crate::profiling::ZrRuntimeProfileControlFnV2;
use crate::status::ZrStatus;
use crate::world_sync::WatchToken;

use super::{
    ZrRuntimeAccessibilityTreeRequestV1, ZrRuntimeBindViewportSurfaceRequestV1,
    ZrRuntimeDrainPluginEventsFnV2, ZrRuntimeEventV1, ZrRuntimeFrameDemandV1,
    ZrRuntimeFrameRequestV1, ZrRuntimeFrameV2, ZrRuntimeHarvestOperationFnV2,
    ZrRuntimeHighlightSetV1, ZrRuntimeHostFetchRequestV1, ZrRuntimePollOperationFnV2,
    ZrRuntimeSessionConfigV3, ZrRuntimeSubmitOperationFnV1, ZrRuntimeSubscribePluginEventFnV1,
    ZrRuntimeUnsubscribePluginEventFnV1,
};

pub const ZR_RUNTIME_GET_API_SYMBOL_V7: &[u8] = b"zircon_runtime_get_api_v7\0";

pub type ZrRuntimeGetApiFnV7 = unsafe extern "C" fn(*const ZrHostApiV1) -> *const ZrRuntimeApiV7;
pub type ZrRuntimeCreateSessionFnV3 =
    unsafe extern "C" fn(ZrRuntimeSessionConfigV3, *mut ZrRuntimeSessionHandle) -> ZrStatus;
pub type ZrRuntimeDestroySessionFnV1 = unsafe extern "C" fn(ZrRuntimeSessionHandle) -> ZrStatus;
pub type ZrRuntimeHandleEventFnV1 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeEventV1) -> ZrStatus;
pub type ZrRuntimeCaptureFrameFnV2 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrRuntimeFrameRequestV1,
    *mut ZrRuntimeFrameV2,
) -> ZrStatus;
pub type ZrRuntimeCaptureAccessibilityTreeFnV2 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrRuntimeAccessibilityTreeRequestV1,
    *mut ZrOwnedResultV2,
) -> ZrStatus;
pub type ZrRuntimeReleaseAllocationFnV2 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeAllocationId) -> ZrStatus;
pub type ZrRuntimeBindViewportSurfaceFnV1 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeBindViewportSurfaceRequestV1) -> ZrStatus;
pub type ZrRuntimeUnbindViewportSurfaceFnV1 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeViewportHandle) -> ZrStatus;
pub type ZrRuntimePresentViewportFnV1 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeFrameRequestV1) -> ZrStatus;
pub type ZrRuntimeSubmitHighlightSetFnV1 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeHighlightSetV1) -> ZrStatus;
pub type ZrRuntimeTickFrameFnV2 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, *mut ZrRuntimeFrameDemandV1) -> ZrStatus;
pub type ZrRuntimeDrainHostRequestsFnV2 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, *mut ZrOwnedResultV2) -> ZrStatus;
pub type ZrRuntimeHostFetchFnV1 =
    unsafe extern "C" fn(ZrRuntimeHostFetchRequestV1, *mut ZrOwnedByteBuffer) -> ZrStatus;
pub type ZrRuntimeQueryWorldFnV2 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrByteSlice, *mut ZrOwnedResultV2) -> ZrStatus;
pub type ZrRuntimeWatchWorldFnV1 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrByteSlice, *mut WatchToken) -> ZrStatus;
pub type ZrRuntimeUnwatchWorldFnV1 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, WatchToken, *mut u8) -> ZrStatus;
pub type ZrRuntimeDrainWorldInvalidationsFnV2 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, *mut ZrOwnedResultV2) -> ZrStatus;

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

/// The immutable runtime API V7 table.
///
/// This shape is frozen. Any future field addition requires a new table
/// version and a coordinated hard cutover of all dynamic hosts.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZrRuntimeApiV7 {
    pub abi_version: u32,
    pub size_bytes: usize,
    pub create_session: Option<ZrRuntimeCreateSessionFnV3>,
    pub destroy_session: Option<ZrRuntimeDestroySessionFnV1>,
    pub release_allocation: Option<ZrRuntimeReleaseAllocationFnV2>,
    pub handle_event: Option<ZrRuntimeHandleEventFnV1>,
    pub capture_frame: Option<ZrRuntimeCaptureFrameFnV2>,
    pub capture_accessibility_tree: Option<ZrRuntimeCaptureAccessibilityTreeFnV2>,
    pub bind_viewport_surface: Option<ZrRuntimeBindViewportSurfaceFnV1>,
    pub unbind_viewport_surface: Option<ZrRuntimeUnbindViewportSurfaceFnV1>,
    pub present_viewport: Option<ZrRuntimePresentViewportFnV1>,
    pub submit_highlight_set: Option<ZrRuntimeSubmitHighlightSetFnV1>,
    pub profile_control: Option<ZrRuntimeProfileControlFnV2>,
    pub tick_frame: Option<ZrRuntimeTickFrameFnV2>,
    pub drain_host_requests: Option<ZrRuntimeDrainHostRequestsFnV2>,
    pub subscribe_plugin_event: Option<ZrRuntimeSubscribePluginEventFnV1>,
    pub unsubscribe_plugin_event: Option<ZrRuntimeUnsubscribePluginEventFnV1>,
    pub drain_plugin_events: Option<ZrRuntimeDrainPluginEventsFnV2>,
    pub submit_operation: Option<ZrRuntimeSubmitOperationFnV1>,
    pub poll_operation: Option<ZrRuntimePollOperationFnV2>,
    pub harvest_operation: Option<ZrRuntimeHarvestOperationFnV2>,
    pub query_world: Option<ZrRuntimeQueryWorldFnV2>,
    pub watch_world: Option<ZrRuntimeWatchWorldFnV1>,
    pub unwatch_world: Option<ZrRuntimeUnwatchWorldFnV1>,
    pub drain_world_invalidations: Option<ZrRuntimeDrainWorldInvalidationsFnV2>,
}

impl ZrRuntimeApiV7 {
    pub const fn empty() -> Self {
        Self {
            abi_version: crate::version::ZIRCON_RUNTIME_API_VERSION_V7,
            size_bytes: core::mem::size_of::<Self>(),
            create_session: None,
            destroy_session: None,
            release_allocation: None,
            handle_event: None,
            capture_frame: None,
            capture_accessibility_tree: None,
            bind_viewport_surface: None,
            unbind_viewport_surface: None,
            present_viewport: None,
            submit_highlight_set: None,
            profile_control: None,
            tick_frame: None,
            drain_host_requests: None,
            subscribe_plugin_event: None,
            unsubscribe_plugin_event: None,
            drain_plugin_events: None,
            submit_operation: None,
            poll_operation: None,
            harvest_operation: None,
            query_world: None,
            watch_world: None,
            unwatch_world: None,
            drain_world_invalidations: None,
        }
    }
}
