use serde::{Deserialize, Serialize};

use crate::{ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimeSessionHandle, ZrStatus};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ZrRuntimePluginEventSubscriptionHandle(pub u64);

impl ZrRuntimePluginEventSubscriptionHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn invalid() -> Self {
        Self(0)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZrRuntimePluginEventSubscribeRequestV1 {
    pub abi_version: u32,
    pub event_id: String,
    pub payload_schema: String,
}

impl ZrRuntimePluginEventSubscribeRequestV1 {
    pub fn new(
        abi_version: u32,
        event_id: impl Into<String>,
        payload_schema: impl Into<String>,
    ) -> Self {
        Self {
            abi_version,
            event_id: event_id.into(),
            payload_schema: payload_schema.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZrRuntimePluginEventDeliveryV1 {
    pub play_session_id: u64,
    pub subscription: ZrRuntimePluginEventSubscriptionHandle,
    pub event_id: String,
    pub payload_schema: String,
    pub sequence: u64,
    pub payload: serde_json::Value,
}

impl ZrRuntimePluginEventDeliveryV1 {
    pub fn new(
        play_session_id: u64,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
        event_id: impl Into<String>,
        payload_schema: impl Into<String>,
        sequence: u64,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            play_session_id,
            subscription,
            event_id: event_id.into(),
            payload_schema: payload_schema.into(),
            sequence,
            payload,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZrRuntimePluginEventDeliveryBatchV1 {
    pub abi_version: u32,
    pub deliveries: Vec<ZrRuntimePluginEventDeliveryV1>,
}

impl ZrRuntimePluginEventDeliveryBatchV1 {
    pub fn new(abi_version: u32, deliveries: Vec<ZrRuntimePluginEventDeliveryV1>) -> Self {
        Self {
            abi_version,
            deliveries,
        }
    }

    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            deliveries: Vec::new(),
        }
    }
}

pub type ZrRuntimeSubscribePluginEventFnV1 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrByteSlice,
    *mut ZrRuntimePluginEventSubscriptionHandle,
) -> ZrStatus;

pub type ZrRuntimeUnsubscribePluginEventFnV1 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrRuntimePluginEventSubscriptionHandle,
) -> ZrStatus;

pub type ZrRuntimeDrainPluginEventsFnV1 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrRuntimePluginEventSubscriptionHandle,
    *mut ZrOwnedByteBuffer,
) -> ZrStatus;
