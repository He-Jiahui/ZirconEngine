//! Shared ABI fixtures for session gateway contract tests.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use zircon_runtime_interface::world_sync::{
    InvalidationBatch, WatchRegistration, WatchToken, WorldFact, WorldQuery, WorldQueryResult,
};
use zircon_runtime_interface::{
    ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimeApiV6, ZrRuntimeEventV1, ZrRuntimeFrameDemandV1,
    ZrRuntimeFrameRequestV1, ZrRuntimeFrameV1, ZrRuntimeOperationDetailKindV2,
    ZrRuntimeOperationHandle, ZrRuntimeOperationPhase, ZrRuntimeOperationResultV1,
    ZrRuntimeOperationStatusV2, ZrRuntimePluginEventDeliveryBatchV1,
    ZrRuntimePluginEventDeliveryV1, ZrRuntimePluginEventSubscriptionHandle, ZrRuntimeSessionHandle,
    ZrStatus, ZrStatusCode, ZR_RUNTIME_FRAME_DEMAND_IDLE_V1,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1,
};

use crate::core::gateway::{
    PluginActivationState, PluginSummaryEntry, RuntimeCapabilities, SessionGateway,
    SessionProfileKind,
};

pub(super) static TICK_CALLS: AtomicUsize = AtomicUsize::new(0);
pub(super) static EVENT_CALLS: AtomicUsize = AtomicUsize::new(0);
pub(super) static FREED_OUTPUTS: AtomicUsize = AtomicUsize::new(0);
pub(super) static OUTPUT_TEST_LOCK: Mutex<()> = Mutex::new(());
pub(super) static WORLD_QUERY_REQUESTS: Mutex<Vec<WorldQuery>> = Mutex::new(Vec::new());
pub(super) static WORLD_WATCH_REQUESTS: Mutex<Vec<WatchRegistration>> = Mutex::new(Vec::new());

pub(super) struct OwnerDropProbe(pub(super) Arc<AtomicUsize>);

impl Drop for OwnerDropProbe {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

unsafe extern "C" fn fake_tick_frame(
    _session: ZrRuntimeSessionHandle,
    output: *mut ZrRuntimeFrameDemandV1,
) -> ZrStatus {
    TICK_CALLS.fetch_add(1, Ordering::SeqCst);
    output.write(ZrRuntimeFrameDemandV1::immediate());
    ZrStatus::ok()
}

pub(super) unsafe extern "C" fn fake_tick_leaves_demand_untouched(
    _session: ZrRuntimeSessionHandle,
    _output: *mut ZrRuntimeFrameDemandV1,
) -> ZrStatus {
    ZrStatus::ok()
}

pub(super) unsafe extern "C" fn fake_tick_after(
    _session: ZrRuntimeSessionHandle,
    output: *mut ZrRuntimeFrameDemandV1,
) -> ZrStatus {
    output.write(ZrRuntimeFrameDemandV1::after(25_000_000));
    ZrStatus::ok()
}

pub(super) unsafe extern "C" fn fake_tick_after_maximum_delay(
    _session: ZrRuntimeSessionHandle,
    output: *mut ZrRuntimeFrameDemandV1,
) -> ZrStatus {
    output.write(ZrRuntimeFrameDemandV1::after(u64::MAX));
    ZrStatus::ok()
}

pub(super) unsafe extern "C" fn fake_tick_not_found(
    _session: ZrRuntimeSessionHandle,
    _output: *mut ZrRuntimeFrameDemandV1,
) -> ZrStatus {
    ZrStatus::new(
        ZrStatusCode::NotFound,
        ZrByteSlice::from_static(b"viewport handle was not found"),
    )
}

pub(super) unsafe extern "C" fn fake_tick_invalid_demand_abi(
    _session: ZrRuntimeSessionHandle,
    output: *mut ZrRuntimeFrameDemandV1,
) -> ZrStatus {
    let mut demand = ZrRuntimeFrameDemandV1::idle();
    demand.abi_version += 1;
    output.write(demand);
    ZrStatus::ok()
}

pub(super) unsafe extern "C" fn fake_tick_unknown_demand_kind(
    _session: ZrRuntimeSessionHandle,
    output: *mut ZrRuntimeFrameDemandV1,
) -> ZrStatus {
    output.write(ZrRuntimeFrameDemandV1 {
        abi_version: zircon_runtime_interface::ZIRCON_RUNTIME_ABI_VERSION_V1,
        kind: u32::MAX,
        delay_nanoseconds: 0,
    });
    ZrStatus::ok()
}

pub(super) unsafe extern "C" fn fake_tick_idle_with_delay(
    _session: ZrRuntimeSessionHandle,
    output: *mut ZrRuntimeFrameDemandV1,
) -> ZrStatus {
    output.write(ZrRuntimeFrameDemandV1 {
        abi_version: zircon_runtime_interface::ZIRCON_RUNTIME_ABI_VERSION_V1,
        kind: ZR_RUNTIME_FRAME_DEMAND_IDLE_V1,
        delay_nanoseconds: 1,
    });
    ZrStatus::ok()
}

unsafe extern "C" fn fake_handle_event(
    _session: ZrRuntimeSessionHandle,
    _event: ZrRuntimeEventV1,
) -> ZrStatus {
    EVENT_CALLS.fetch_add(1, Ordering::SeqCst);
    ZrStatus::ok()
}

unsafe extern "C" fn fake_capture_frame(
    _session: ZrRuntimeSessionHandle,
    request: ZrRuntimeFrameRequestV1,
    output: *mut ZrRuntimeFrameV1,
) -> ZrStatus {
    let mut frame =
        ZrRuntimeFrameV1::empty(zircon_runtime_interface::ZIRCON_RUNTIME_ABI_VERSION_V1);
    frame.width = request.size.width;
    frame.height = request.size.height;
    output.write(frame);
    ZrStatus::ok()
}

pub(super) unsafe extern "C" fn fake_capture_owned_frame(
    _session: ZrRuntimeSessionHandle,
    request: ZrRuntimeFrameRequestV1,
    output: *mut ZrRuntimeFrameV1,
) -> ZrStatus {
    let mut frame =
        ZrRuntimeFrameV1::empty(zircon_runtime_interface::ZIRCON_RUNTIME_ABI_VERSION_V1);
    frame.width = request.size.width;
    frame.height = request.size.height;
    frame.generation = 31;
    write_owned_bytes(vec![1, 2, 3, 4], &mut frame.rgba);
    output.write(frame);
    ZrStatus::ok()
}

pub(super) unsafe extern "C" fn fake_capture_truncated_frame(
    _session: ZrRuntimeSessionHandle,
    _request: ZrRuntimeFrameRequestV1,
    output: *mut ZrRuntimeFrameV1,
) -> ZrStatus {
    let mut frame =
        ZrRuntimeFrameV1::empty(zircon_runtime_interface::ZIRCON_RUNTIME_ABI_VERSION_V1);
    frame.width = 2;
    frame.height = 2;
    write_owned_bytes(vec![1, 2, 3, 4], &mut frame.rgba);
    output.write(frame);
    ZrStatus::ok()
}

unsafe extern "C" fn free_json_output(output: ZrOwnedByteBuffer) -> ZrStatus {
    drop(Vec::from_raw_parts(
        output.data,
        output.len,
        output.capacity,
    ));
    FREED_OUTPUTS.fetch_add(1, Ordering::SeqCst);
    ZrStatus::ok()
}

fn write_owned_bytes(mut bytes: Vec<u8>, output: *mut ZrOwnedByteBuffer) {
    let buffer = ZrOwnedByteBuffer {
        data: bytes.as_mut_ptr(),
        len: bytes.len(),
        capacity: bytes.capacity(),
        owner_token: 0,
        free: Some(free_json_output),
    };
    std::mem::forget(bytes);
    unsafe { output.write(buffer) };
}

fn write_json_output<T: serde::Serialize>(value: &T, output: *mut ZrOwnedByteBuffer) {
    write_owned_bytes(
        serde_json::to_vec(value).expect("serialize fake ABI output"),
        output,
    );
}

unsafe extern "C" fn fake_query_world(
    _session: ZrRuntimeSessionHandle,
    request: ZrByteSlice,
    output: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    let query = match serde_json::from_slice::<WorldQuery>(unsafe { request.as_slice() }) {
        Ok(query) => query,
        Err(_) => return ZrStatus::new(ZrStatusCode::InvalidArgument, ZrByteSlice::empty()),
    };
    WORLD_QUERY_REQUESTS.lock().unwrap().push(query);
    write_json_output(&WorldQueryResult::NotModified { generation: 73 }, output);
    ZrStatus::ok()
}

unsafe extern "C" fn fake_watch_world(
    _session: ZrRuntimeSessionHandle,
    request: ZrByteSlice,
    output: *mut WatchToken,
) -> ZrStatus {
    let registration =
        match serde_json::from_slice::<WatchRegistration>(unsafe { request.as_slice() }) {
            Ok(registration) => registration,
            Err(_) => return ZrStatus::new(ZrStatusCode::InvalidArgument, ZrByteSlice::empty()),
        };
    WORLD_WATCH_REQUESTS.lock().unwrap().push(registration);
    unsafe { output.write(WatchToken::new(41)) };
    ZrStatus::ok()
}

pub(super) unsafe extern "C" fn fake_watch_world_invalid_token(
    _session: ZrRuntimeSessionHandle,
    _request: ZrByteSlice,
    output: *mut WatchToken,
) -> ZrStatus {
    unsafe { output.write(WatchToken::new(0)) };
    ZrStatus::ok()
}

unsafe extern "C" fn fake_unwatch_world(
    _session: ZrRuntimeSessionHandle,
    token: WatchToken,
    output: *mut u8,
) -> ZrStatus {
    if token != WatchToken::new(41) {
        return ZrStatus::new(ZrStatusCode::NotFound, ZrByteSlice::empty());
    }
    unsafe { output.write(1) };
    ZrStatus::ok()
}

unsafe extern "C" fn fake_drain_world_invalidations(
    _session: ZrRuntimeSessionHandle,
    output: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    write_json_output(
        &vec![InvalidationBatch {
            generation: 73,
            dirty: vec![WatchToken::new(41)],
            facts: vec![WorldFact::Spawned(7)],
        }],
        output,
    );
    ZrStatus::ok()
}

unsafe extern "C" fn fake_subscribe_plugin_event(
    _session: ZrRuntimeSessionHandle,
    _request: ZrByteSlice,
    output: *mut ZrRuntimePluginEventSubscriptionHandle,
) -> ZrStatus {
    output.write(ZrRuntimePluginEventSubscriptionHandle::new(23));
    ZrStatus::ok()
}

unsafe extern "C" fn fake_unsubscribe_plugin_event(
    _session: ZrRuntimeSessionHandle,
    _subscription: ZrRuntimePluginEventSubscriptionHandle,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_drain_plugin_events(
    _session: ZrRuntimeSessionHandle,
    subscription: ZrRuntimePluginEventSubscriptionHandle,
    output: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    write_json_output(
        &ZrRuntimePluginEventDeliveryBatchV1::new(
            zircon_runtime_interface::ZIRCON_RUNTIME_ABI_VERSION_V1,
            vec![ZrRuntimePluginEventDeliveryV1::new(
                7,
                subscription,
                "navigation.path.updated",
                "zircon.navigation.path.v1",
                1,
                serde_json::json!({"pathCount": 2}),
            )],
        )
        .with_runtime_backlog(9, 17),
        output,
    );
    ZrStatus::ok()
}

pub(super) unsafe extern "C" fn fake_drain_crossed_plugin_events(
    _session: ZrRuntimeSessionHandle,
    subscription: ZrRuntimePluginEventSubscriptionHandle,
    output: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    write_json_output(
        &ZrRuntimePluginEventDeliveryBatchV1::new(
            zircon_runtime_interface::ZIRCON_RUNTIME_ABI_VERSION_V1,
            vec![ZrRuntimePluginEventDeliveryV1::new(
                7,
                ZrRuntimePluginEventSubscriptionHandle::new(subscription.raw() + 1),
                "navigation.path.updated",
                "zircon.navigation.path.v1",
                1,
                serde_json::Value::Null,
            )],
        ),
        output,
    );
    ZrStatus::ok()
}

pub(super) unsafe extern "C" fn fake_drain_oversized_plugin_event_page(
    _session: ZrRuntimeSessionHandle,
    _subscription: ZrRuntimePluginEventSubscriptionHandle,
    output: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    write_owned_bytes(
        vec![b'{'; ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1 + 1],
        output,
    );
    ZrStatus::ok()
}

pub(super) unsafe extern "C" fn fake_drain_plugin_event_page_above_delivery_limit(
    _session: ZrRuntimeSessionHandle,
    subscription: ZrRuntimePluginEventSubscriptionHandle,
    output: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    let deliveries = (0..=ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1)
        .map(|sequence| {
            ZrRuntimePluginEventDeliveryV1::new(
                7,
                subscription,
                "navigation.path.updated",
                "zircon.navigation.path.v1",
                sequence as u64 + 1,
                serde_json::Value::Null,
            )
        })
        .collect();
    write_json_output(
        &ZrRuntimePluginEventDeliveryBatchV1::new(
            zircon_runtime_interface::ZIRCON_RUNTIME_ABI_VERSION_V1,
            deliveries,
        ),
        output,
    );
    ZrStatus::ok()
}

unsafe extern "C" fn fake_submit_operation(
    _session: ZrRuntimeSessionHandle,
    _request: ZrByteSlice,
    output: *mut ZrRuntimeOperationHandle,
) -> ZrStatus {
    output.write(ZrRuntimeOperationHandle::new(29));
    ZrStatus::ok()
}

unsafe extern "C" fn fake_poll_operation(
    _session: ZrRuntimeSessionHandle,
    operation: ZrRuntimeOperationHandle,
    output: *mut ZrRuntimeOperationStatusV2,
) -> ZrStatus {
    output.write(ZrRuntimeOperationStatusV2::new(
        operation,
        ZrRuntimeOperationPhase::ReadyToApply,
        1,
        2,
        ZrRuntimeOperationDetailKindV2::None,
        0,
    ));
    ZrStatus::ok()
}

unsafe extern "C" fn fake_harvest_operation(
    _session: ZrRuntimeSessionHandle,
    operation: ZrRuntimeOperationHandle,
    output: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    write_json_output(
        &ZrRuntimeOperationResultV1::succeeded(
            zircon_runtime_interface::ZIRCON_RUNTIME_ABI_VERSION_V1,
            operation,
            "navigation.bake.scene",
            serde_json::json!({"meshCount": 4}),
        ),
        output,
    );
    ZrStatus::ok()
}

pub(super) unsafe extern "C" fn fake_poll_crossed_operation(
    _session: ZrRuntimeSessionHandle,
    operation: ZrRuntimeOperationHandle,
    output: *mut ZrRuntimeOperationStatusV2,
) -> ZrStatus {
    output.write(ZrRuntimeOperationStatusV2::new(
        ZrRuntimeOperationHandle::new(operation.raw() + 1),
        ZrRuntimeOperationPhase::ReadyToApply,
        1,
        2,
        ZrRuntimeOperationDetailKindV2::None,
        0,
    ));
    ZrStatus::ok()
}

pub(super) unsafe extern "C" fn fake_harvest_crossed_operation(
    _session: ZrRuntimeSessionHandle,
    operation: ZrRuntimeOperationHandle,
    output: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    write_json_output(
        &ZrRuntimeOperationResultV1::succeeded(
            zircon_runtime_interface::ZIRCON_RUNTIME_ABI_VERSION_V1,
            ZrRuntimeOperationHandle::new(operation.raw() + 1),
            "navigation.bake.scene",
            serde_json::Value::Null,
        ),
        output,
    );
    ZrStatus::ok()
}

pub(super) unsafe extern "C" fn fake_poll_foreign_abi(
    _session: ZrRuntimeSessionHandle,
    operation: ZrRuntimeOperationHandle,
    output: *mut ZrRuntimeOperationStatusV2,
) -> ZrStatus {
    let mut status = ZrRuntimeOperationStatusV2::new(
        operation,
        ZrRuntimeOperationPhase::Queued,
        0,
        1,
        ZrRuntimeOperationDetailKindV2::None,
        0,
    );
    status.abi_version = zircon_runtime_interface::ZIRCON_RUNTIME_ABI_VERSION_V1;
    output.write(status);
    ZrStatus::ok()
}

pub(super) unsafe extern "C" fn fake_poll_unknown_phase(
    _session: ZrRuntimeSessionHandle,
    operation: ZrRuntimeOperationHandle,
    output: *mut ZrRuntimeOperationStatusV2,
) -> ZrStatus {
    let mut status = ZrRuntimeOperationStatusV2::new(
        operation,
        ZrRuntimeOperationPhase::Queued,
        0,
        1,
        ZrRuntimeOperationDetailKindV2::None,
        0,
    );
    status.phase = 99;
    output.write(status);
    ZrStatus::ok()
}

pub(super) unsafe extern "C" fn fake_poll_unknown_detail(
    _session: ZrRuntimeSessionHandle,
    operation: ZrRuntimeOperationHandle,
    output: *mut ZrRuntimeOperationStatusV2,
) -> ZrStatus {
    let mut status = ZrRuntimeOperationStatusV2::new(
        operation,
        ZrRuntimeOperationPhase::Queued,
        0,
        1,
        ZrRuntimeOperationDetailKindV2::None,
        0,
    );
    status.detail_kind = 99;
    output.write(status);
    ZrStatus::ok()
}

pub(super) unsafe extern "C" fn fake_poll_error_without_output(
    _session: ZrRuntimeSessionHandle,
    _operation: ZrRuntimeOperationHandle,
    _output: *mut ZrRuntimeOperationStatusV2,
) -> ZrStatus {
    ZrStatus::new(
        ZrStatusCode::NotFound,
        ZrByteSlice::from_static(b"operation handle was not found"),
    )
}

pub(super) unsafe extern "C" fn fake_poll_error_with_output(
    _session: ZrRuntimeSessionHandle,
    operation: ZrRuntimeOperationHandle,
    output: *mut ZrRuntimeOperationStatusV2,
) -> ZrStatus {
    output.write(ZrRuntimeOperationStatusV2::new(
        operation,
        ZrRuntimeOperationPhase::Queued,
        0,
        1,
        ZrRuntimeOperationDetailKindV2::None,
        0,
    ));
    ZrStatus::new(
        ZrStatusCode::NotFound,
        ZrByteSlice::from_static(b"operation handle was not found"),
    )
}

pub(super) unsafe extern "C" fn fake_drain_empty_owned_output(
    _session: ZrRuntimeSessionHandle,
    _subscription: ZrRuntimePluginEventSubscriptionHandle,
    output: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    let mut bytes = Vec::with_capacity(1);
    output.write(ZrOwnedByteBuffer {
        data: bytes.as_mut_ptr(),
        len: 0,
        capacity: bytes.capacity(),
        owner_token: 0,
        free: Some(free_json_output),
    });
    std::mem::forget(bytes);
    ZrStatus::ok()
}

pub(super) unsafe extern "C" fn fake_drain_empty_plugin_event_page(
    _session: ZrRuntimeSessionHandle,
    _subscription: ZrRuntimePluginEventSubscriptionHandle,
    output: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    output.write(ZrOwnedByteBuffer::empty());
    ZrStatus::ok()
}

pub(super) fn capabilities() -> RuntimeCapabilities {
    RuntimeCapabilities::new(
        SessionProfileKind::Editor,
        [
            "editor.host.ui_shell",
            "editor.host.scene_interaction",
            "editor.host.ui_shell",
        ],
        [PluginSummaryEntry::new(
            "zircon.navigation",
            "1.2.0",
            PluginActivationState::Active,
        )],
    )
}

pub(super) fn api_table() -> ZrRuntimeApiV6 {
    let mut api = ZrRuntimeApiV6::empty();
    api.tick_frame = Some(fake_tick_frame);
    api.handle_event = Some(fake_handle_event);
    api.capture_frame = Some(fake_capture_frame);
    api.subscribe_plugin_event = Some(fake_subscribe_plugin_event);
    api.unsubscribe_plugin_event = Some(fake_unsubscribe_plugin_event);
    api.drain_plugin_events = Some(fake_drain_plugin_events);
    api.submit_operation = Some(fake_submit_operation);
    api.poll_operation = Some(fake_poll_operation);
    api.harvest_operation = Some(fake_harvest_operation);
    api.query_world = Some(fake_query_world);
    api.watch_world = Some(fake_watch_world);
    api.unwatch_world = Some(fake_unwatch_world);
    api.drain_world_invalidations = Some(fake_drain_world_invalidations);
    api
}

pub(super) fn gateway(api: ZrRuntimeApiV6) -> SessionGateway {
    let owner: Arc<dyn Send + Sync> = Arc::new(());
    unsafe {
        SessionGateway::new(owner, api, ZrRuntimeSessionHandle::new(17), capabilities())
            .expect("valid session gateway")
    }
}
