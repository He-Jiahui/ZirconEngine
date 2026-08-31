use crate::buffer::{ZrByteSlice, ZrOwnedByteBuffer, ZrOwnedResultV2};
use crate::handles::{ZrRuntimeAllocationId, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle};
pub use crate::profiling::ZrRuntimeProfileControlFnV2;
use crate::status::ZrStatus;
use crate::world_sync::WatchToken;

use super::super::{
    frame::{
        ZrRuntimeFrameDemandV1, ZrRuntimeHighlightSetV1, ZrRuntimeViewportPickRequestV1,
        ZrRuntimeViewportPickResultV1, ZrRuntimeViewportPickTicket,
    },
    session::{
        ZrRuntimeAccessibilityTreeRequestV1, ZrRuntimeBindViewportSurfaceRequestV1,
        ZrRuntimeDrainPluginEventsFnV2, ZrRuntimeEventV1, ZrRuntimeFrameRequestV1,
        ZrRuntimeFrameV2, ZrRuntimeHarvestOperationFnV2, ZrRuntimeHostFetchRequestV1,
        ZrRuntimePollOperationFnV2, ZrRuntimeSessionConfigV3, ZrRuntimeSubmitOperationFnV1,
        ZrRuntimeSubscribePluginEventFnV1, ZrRuntimeUnsubscribePluginEventFnV1,
    },
};

pub type ZrRuntimeGetApiFnV8 = unsafe extern "C" fn(*const ZrHostApiV1) -> *const ZrRuntimeApiV8;
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
pub type ZrRuntimeRequestViewportPickFnV1 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrRuntimeViewportPickRequestV1,
    *mut ZrRuntimeViewportPickTicket,
) -> ZrStatus;
pub type ZrRuntimePollViewportPickFnV1 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrRuntimeViewportPickTicket,
    *mut ZrRuntimeViewportPickResultV1,
) -> ZrStatus;
pub type ZrRuntimeCancelViewportPickFnV1 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeViewportPickTicket) -> ZrStatus;
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

/// The immutable runtime API V8 table.
///
/// This shape is frozen. Any future field addition requires a new table
/// version and a coordinated hard cutover of all dynamic hosts.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZrRuntimeApiV8 {
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
    pub request_viewport_pick: Option<ZrRuntimeRequestViewportPickFnV1>,
    pub poll_viewport_pick: Option<ZrRuntimePollViewportPickFnV1>,
    pub cancel_viewport_pick: Option<ZrRuntimeCancelViewportPickFnV1>,
}

impl ZrRuntimeApiV8 {
    pub const fn empty() -> Self {
        Self {
            abi_version: crate::version::ZIRCON_RUNTIME_API_VERSION_V8,
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
            request_viewport_pick: None,
            poll_viewport_pick: None,
            cancel_viewport_pick: None,
        }
    }
}

const ZR_RUNTIME_API_V8_REQUIRED_FIELD_NAMES: &[&str] = &[
    "abi_version",
    "size_bytes",
    "create_session",
    "destroy_session",
    "release_allocation",
    "handle_event",
    "capture_frame",
    "submit_highlight_set",
    "tick_frame",
    "subscribe_plugin_event",
    "unsubscribe_plugin_event",
    "drain_plugin_events",
    "submit_operation",
    "poll_operation",
    "harvest_operation",
    "query_world",
    "watch_world",
    "unwatch_world",
    "drain_world_invalidations",
    "request_viewport_pick",
    "poll_viewport_pick",
    "cancel_viewport_pick",
];
const ZR_RUNTIME_API_V8_OPTIONAL_FIELD_NAMES: &[&str] = &[
    "capture_accessibility_tree",
    "bind_viewport_surface",
    "unbind_viewport_surface",
    "present_viewport",
    "profile_control",
    "drain_host_requests",
];
const ZR_HOST_API_V1_OPTIONAL_FIELD_NAMES: &[&str] = &["diagnostics_sink", "fetch_resource"];

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{
        ZR_HOST_API_V1_OPTIONAL_FIELD_NAMES, ZR_RUNTIME_API_V8_OPTIONAL_FIELD_NAMES,
        ZR_RUNTIME_API_V8_REQUIRED_FIELD_NAMES,
    };

    #[test]
    fn interface_spec_slot_partitions_match_the_abi_table_fields() {
        assert_eq!(
            crate::runtime_build_set::ZR_RUNTIME_API_V8_REQUIRED_SLOT_NAMES,
            ZR_RUNTIME_API_V8_REQUIRED_FIELD_NAMES,
        );
        assert_eq!(
            crate::runtime_build_set::ZR_RUNTIME_API_V8_OPTIONAL_SLOT_NAMES,
            ZR_RUNTIME_API_V8_OPTIONAL_FIELD_NAMES,
        );
        assert_eq!(
            crate::runtime_build_set::ZR_HOST_API_V1_OPTIONAL_SLOT_NAMES,
            ZR_HOST_API_V1_OPTIONAL_FIELD_NAMES,
        );

        let expected_runtime_slots =
            crate::runtime_build_set::ZR_RUNTIME_API_V8_REQUIRED_SLOT_NAMES
                .iter()
                .chain(crate::runtime_build_set::ZR_RUNTIME_API_V8_OPTIONAL_SLOT_NAMES.iter())
                .filter(|field| **field != "abi_version" && **field != "size_bytes")
                .map(|field| (*field).to_owned())
                .collect::<BTreeSet<_>>();
        assert_eq!(
            concrete_table_slot_names("ZrRuntimeApiV8"),
            expected_runtime_slots,
        );

        let expected_host_slots = crate::runtime_build_set::ZR_HOST_API_V1_OPTIONAL_SLOT_NAMES
            .iter()
            .map(|field| (*field).to_owned())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            concrete_table_slot_names("ZrHostApiV1"),
            expected_host_slots,
        );
    }

    fn concrete_table_slot_names(table_name: &str) -> BTreeSet<String> {
        let source = include_str!("api_table.rs");
        let struct_needle = format!("pub struct {table_name} ");
        let struct_index = source
            .find(&struct_needle)
            .unwrap_or_else(|| panic!("{table_name} must remain a concrete ABI table declaration"));
        let body_start = source[struct_index..]
            .find('{')
            .map(|offset| struct_index + offset + 1)
            .expect("concrete ABI table must have a field body");
        let fields = source[body_start..]
            .lines()
            .map(str::trim)
            .take_while(|line| *line != "}")
            .filter_map(|line| line.strip_prefix("pub "))
            .map(|field| {
                field
                    .split_once(':')
                    .map(|(field, _)| field.trim().to_owned())
                    .expect("concrete ABI table field must have a type")
            })
            .collect::<Vec<_>>();

        assert_eq!(
            fields.first().map(String::as_str),
            Some("abi_version"),
            "{table_name} must begin with the ABI version header"
        );
        assert_eq!(
            fields.get(1).map(String::as_str),
            Some("size_bytes"),
            "{table_name} must retain the byte-size header after its ABI version"
        );
        fields.into_iter().skip(2).collect()
    }
}
