use crate::buffer::ZrByteSlice;
use crate::status::{ZrStatus, ZrStatusCode};

/// Stable plugin-event callback ABI used by subsystem-specific adapters.
/// Payload semantics are carried by the namespace and schema strings.
pub type ZrPluginEventCallbackFnV1 = unsafe extern "C" fn(
    ZrPluginEventCallbackRequestV1,
    *mut ZrPluginEventCallbackResultV1,
) -> ZrStatus;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZrPluginEventCallbackRequestV1 {
    pub abi_version: u32,
    pub namespace: ZrByteSlice,
    pub plugin_id: ZrByteSlice,
    pub handler_id: ZrByteSlice,
    pub event_id: ZrByteSlice,
    pub source_path: ZrByteSlice,
    pub time_seconds: f32,
    pub payload_schema: ZrByteSlice,
    pub payload: ZrByteSlice,
}

impl ZrPluginEventCallbackRequestV1 {
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            namespace: ZrByteSlice::empty(),
            plugin_id: ZrByteSlice::empty(),
            handler_id: ZrByteSlice::empty(),
            event_id: ZrByteSlice::empty(),
            source_path: ZrByteSlice::empty(),
            time_seconds: 0.0,
            payload_schema: ZrByteSlice::empty(),
            payload: ZrByteSlice::empty(),
        }
    }

    pub const fn new(
        abi_version: u32,
        namespace: ZrByteSlice,
        plugin_id: ZrByteSlice,
        handler_id: ZrByteSlice,
        event_id: ZrByteSlice,
        source_path: ZrByteSlice,
        time_seconds: f32,
        payload_schema: ZrByteSlice,
        payload: ZrByteSlice,
    ) -> Self {
        Self {
            abi_version,
            namespace,
            plugin_id,
            handler_id,
            event_id,
            source_path,
            time_seconds,
            payload_schema,
            payload,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrPluginEventCallbackResultV1 {
    pub abi_version: u32,
    pub status: ZrStatus,
}

impl ZrPluginEventCallbackResultV1 {
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            status: ZrStatus::ok(),
        }
    }

    pub const fn ok(abi_version: u32) -> Self {
        Self {
            abi_version,
            status: ZrStatus::ok(),
        }
    }

    pub const fn failed(abi_version: u32, diagnostics: ZrByteSlice) -> Self {
        Self {
            abi_version,
            status: ZrStatus::new(ZrStatusCode::Error, diagnostics),
        }
    }
}
