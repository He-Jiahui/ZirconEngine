use serde::{Deserialize, Serialize};
use serde_json::value::{to_raw_value, RawValue};

use crate::{ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimeSessionHandle, ZrStatus};

/// Maximum event deliveries returned by one V1 plugin-event drain page.
pub const ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1: usize = 64;

/// Maximum JSON-encoded bytes returned by one V1 plugin-event drain page.
pub const ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1: usize = 256 * 1024;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZrRuntimePluginEventDeliveryV1 {
    pub play_session_id: u64,
    pub subscription: ZrRuntimePluginEventSubscriptionHandle,
    pub event_id: String,
    pub payload_schema: String,
    pub sequence: u64,
    /// Owned JSON bytes forwarded to the typed consumer without an intermediate value tree.
    pub payload: Box<RawValue>,
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
            payload: to_raw_value(&payload)
                .expect("serde_json::Value always serializes to raw JSON payloads"),
        }
    }
}

impl PartialEq for ZrRuntimePluginEventDeliveryV1 {
    fn eq(&self, other: &Self) -> bool {
        self.play_session_id == other.play_session_id
            && self.subscription == other.subscription
            && self.event_id == other.event_id
            && self.payload_schema == other.payload_schema
            && self.sequence == other.sequence
            && serde_json::from_str::<serde_json::Value>(self.payload.get())
                .expect("RawValue always contains valid JSON")
                == serde_json::from_str::<serde_json::Value>(other.payload.get())
                    .expect("RawValue always contains valid JSON")
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZrRuntimePluginEventDeliveryBatchV1 {
    pub abi_version: u32,
    pub deliveries: Vec<ZrRuntimePluginEventDeliveryV1>,
    /// Events still retained by the Runtime subscription after this page commits.
    #[serde(default)]
    pub remaining_deliveries: u32,
    /// Monotonic elapsed age of the oldest event still retained by the Runtime subscription.
    #[serde(default)]
    pub oldest_pending_age_millis: u64,
}

impl ZrRuntimePluginEventDeliveryBatchV1 {
    pub fn new(abi_version: u32, deliveries: Vec<ZrRuntimePluginEventDeliveryV1>) -> Self {
        Self {
            abi_version,
            deliveries,
            remaining_deliveries: 0,
            oldest_pending_age_millis: 0,
        }
    }

    pub const fn with_runtime_backlog(
        mut self,
        remaining_deliveries: u32,
        oldest_pending_age_millis: u64,
    ) -> Self {
        self.remaining_deliveries = remaining_deliveries;
        self.oldest_pending_age_millis = oldest_pending_age_millis;
        self
    }

    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            deliveries: Vec::new(),
            remaining_deliveries: 0,
            oldest_pending_age_millis: 0,
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
