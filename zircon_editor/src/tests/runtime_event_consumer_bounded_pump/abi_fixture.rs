use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use zircon_runtime_interface::{
    GatewaySessionIdentity, ZrByteSlice, ZrOwnedResultV2, ZrRuntimeAllocationId, ZrRuntimeApiV8,
    ZrRuntimePluginEventDeliveryBatchV1, ZrRuntimePluginEventDeliveryV1,
    ZrRuntimePluginEventSubscriptionHandle, ZrRuntimeSessionHandle, ZrRuntimeViewportPickRequestV1,
    ZrRuntimeViewportPickResultV1, ZrRuntimeViewportPickTicket, ZrStatus,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1,
};

use crate::core::gateway::{RuntimeCapabilities, SessionGateway};

use super::support::SCHEMA;

#[derive(Default)]
pub(super) struct AbiEventBacklog {
    pub(super) remaining: u64,
    pub(super) next_sequence: u64,
    pub(super) oldest_pending_age_millis: u64,
}

pub(super) static ABI_EVENT_BACKLOG: Mutex<AbiEventBacklog> = Mutex::new(AbiEventBacklog {
    remaining: 0,
    next_sequence: 1,
    oldest_pending_age_millis: 0,
});
pub(super) static ABI_EVENT_FIXTURE_LOCK: Mutex<()> = Mutex::new(());
static NEXT_ALLOCATION_ID: AtomicU64 = AtomicU64::new(1);
static ABI_EVENT_ALLOCATIONS: OnceLock<Mutex<HashMap<u64, Box<[u8]>>>> = OnceLock::new();

unsafe extern "C" fn abi_subscribe_plugin_event(
    _session: ZrRuntimeSessionHandle,
    _request: ZrByteSlice,
    output: *mut ZrRuntimePluginEventSubscriptionHandle,
) -> ZrStatus {
    output.write(ZrRuntimePluginEventSubscriptionHandle::new(11));
    ZrStatus::ok()
}

unsafe extern "C" fn abi_unsubscribe_plugin_event(
    _session: ZrRuntimeSessionHandle,
    _subscription: ZrRuntimePluginEventSubscriptionHandle,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn release_abi_event_page(
    _session: ZrRuntimeSessionHandle,
    allocation: ZrRuntimeAllocationId,
) -> ZrStatus {
    ABI_EVENT_ALLOCATIONS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .remove(&allocation.raw())
        .expect("ABI event allocation must be released exactly once");
    ZrStatus::ok()
}

fn write_abi_event_page(batch: &ZrRuntimePluginEventDeliveryBatchV1, output: *mut ZrOwnedResultV2) {
    let bytes = serde_json::to_vec(batch)
        .expect("serialize bounded ABI event page")
        .into_boxed_slice();
    let data = bytes.as_ptr();
    let len = bytes.len() as u64;
    let allocation = ZrRuntimeAllocationId::new(NEXT_ALLOCATION_ID.fetch_add(1, Ordering::Relaxed));
    ABI_EVENT_ALLOCATIONS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(allocation.raw(), bytes);
    unsafe {
        output.write(ZrOwnedResultV2 {
            data,
            len,
            allocation,
        })
    };
}

unsafe extern "C" fn abi_drain_plugin_events(
    _session: ZrRuntimeSessionHandle,
    subscription: ZrRuntimePluginEventSubscriptionHandle,
    output: *mut ZrOwnedResultV2,
) -> ZrStatus {
    let mut backlog = ABI_EVENT_BACKLOG.lock().expect("lock ABI event backlog");
    let count = backlog
        .remaining
        .min(ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1 as u64);
    let first_sequence = backlog.next_sequence;
    backlog.remaining -= count;
    backlog.next_sequence = backlog.next_sequence.saturating_add(count);
    let remaining_deliveries = u32::try_from(backlog.remaining)
        .expect("bounded ABI event fixture remaining count fits the protocol field");
    let oldest_pending_age_millis = if remaining_deliveries == 0 {
        0
    } else {
        backlog.oldest_pending_age_millis
    };
    drop(backlog);
    let deliveries = (first_sequence..first_sequence.saturating_add(count))
        .map(|sequence| {
            ZrRuntimePluginEventDeliveryV1::new(
                7,
                subscription,
                "tests.events.storm",
                SCHEMA,
                sequence,
                serde_json::json!({"value": sequence}),
            )
        })
        .collect();
    write_abi_event_page(
        &ZrRuntimePluginEventDeliveryBatchV1::new(ZIRCON_RUNTIME_ABI_VERSION_V1, deliveries)
            .with_runtime_backlog(remaining_deliveries, oldest_pending_age_millis),
        output,
    );
    ZrStatus::ok()
}

unsafe extern "C" fn request_test_viewport_pick(
    _session: ZrRuntimeSessionHandle,
    _request: ZrRuntimeViewportPickRequestV1,
    _out_ticket: *mut ZrRuntimeViewportPickTicket,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn poll_test_viewport_pick(
    _session: ZrRuntimeSessionHandle,
    _ticket: ZrRuntimeViewportPickTicket,
    _out_result: *mut ZrRuntimeViewportPickResultV1,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn cancel_test_viewport_pick(
    _session: ZrRuntimeSessionHandle,
    _ticket: ZrRuntimeViewportPickTicket,
) -> ZrStatus {
    ZrStatus::ok()
}

pub(super) fn abi_gateway() -> SessionGateway {
    let mut api = ZrRuntimeApiV8::empty();
    api.release_allocation = Some(release_abi_event_page);
    api.subscribe_plugin_event = Some(abi_subscribe_plugin_event);
    api.unsubscribe_plugin_event = Some(abi_unsubscribe_plugin_event);
    api.drain_plugin_events = Some(abi_drain_plugin_events);
    api.request_viewport_pick = Some(request_test_viewport_pick);
    api.poll_viewport_pick = Some(poll_test_viewport_pick);
    api.cancel_viewport_pick = Some(cancel_test_viewport_pick);
    unsafe {
        SessionGateway::new_with_identity(
            Arc::new(()),
            api,
            ZrRuntimeSessionHandle::new(7),
            GatewaySessionIdentity::new(7, ZrRuntimeSessionHandle::new(7), 1, None),
            RuntimeCapabilities::editor_default(),
            Arc::new(zircon_runtime_host::foreign_output::RuntimeForeignOutputState::default()),
        )
        .expect("construct bounded ABI session gateway")
    }
}
