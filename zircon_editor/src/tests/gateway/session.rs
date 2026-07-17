use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use zircon_runtime_interface::{
    ProfileControlCommand, ProfileControlRequest, ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimeApiV2,
    ZrRuntimeEventV1, ZrRuntimeFrameRequestV1, ZrRuntimeFrameV1, ZrRuntimeOperationHandle,
    ZrRuntimeOperationPhase, ZrRuntimeOperationProgressV1, ZrRuntimeOperationResultV1,
    ZrRuntimeOperationSubmitRequestV1, ZrRuntimePluginEventDeliveryBatchV1,
    ZrRuntimePluginEventDeliveryV1, ZrRuntimePluginEventSubscriptionHandle, ZrRuntimeSessionHandle,
    ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1, ZrStatus, ZrStatusCode,
};

use crate::core::gateway::{
    EditorRuntimeGateway, GatewayError, PluginActivationState, PluginSummaryEntry,
    RuntimeCapabilities, SessionGateway, SessionProfileKind,
};

static TICK_CALLS: AtomicUsize = AtomicUsize::new(0);
static EVENT_CALLS: AtomicUsize = AtomicUsize::new(0);
static FREED_OUTPUTS: AtomicUsize = AtomicUsize::new(0);
static OUTPUT_TEST_LOCK: Mutex<()> = Mutex::new(());

struct OwnerDropProbe(Arc<AtomicUsize>);

impl Drop for OwnerDropProbe {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

unsafe extern "C" fn fake_tick_frame(_session: ZrRuntimeSessionHandle) -> ZrStatus {
    TICK_CALLS.fetch_add(1, Ordering::SeqCst);
    ZrStatus::ok()
}

unsafe extern "C" fn fake_tick_not_found(_session: ZrRuntimeSessionHandle) -> ZrStatus {
    ZrStatus::new(
        ZrStatusCode::NotFound,
        ZrByteSlice::from_static(b"viewport handle was not found"),
    )
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

unsafe extern "C" fn fake_capture_owned_frame(
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

unsafe extern "C" fn free_json_output(output: ZrOwnedByteBuffer) -> ZrStatus {
    drop(Vec::from_raw_parts(
        output.data,
        output.len,
        output.capacity,
    ));
    FREED_OUTPUTS.fetch_add(1, Ordering::SeqCst);
    ZrStatus::ok()
}

unsafe extern "C" fn record_output_release(_output: ZrOwnedByteBuffer) -> ZrStatus {
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
    output: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    write_json_output(
        &ZrRuntimeOperationProgressV1::new(
            zircon_runtime_interface::ZIRCON_RUNTIME_ABI_VERSION_V1,
            operation,
            ZrRuntimeOperationPhase::Running,
            1,
            2,
            "baking",
        ),
        output,
    );
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

unsafe extern "C" fn fake_poll_invalid_json(
    _session: ZrRuntimeSessionHandle,
    _operation: ZrRuntimeOperationHandle,
    output: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    write_owned_bytes(b"{".to_vec(), output);
    ZrStatus::ok()
}

unsafe extern "C" fn fake_poll_malformed_buffer(
    _session: ZrRuntimeSessionHandle,
    _operation: ZrRuntimeOperationHandle,
    output: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    output.write(ZrOwnedByteBuffer {
        data: std::ptr::null_mut(),
        len: 2,
        capacity: 1,
        owner_token: 0,
        free: Some(record_output_release),
    });
    ZrStatus::ok()
}

unsafe extern "C" fn fake_poll_error_with_output(
    _session: ZrRuntimeSessionHandle,
    operation: ZrRuntimeOperationHandle,
    output: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    write_json_output(
        &ZrRuntimeOperationProgressV1::new(
            zircon_runtime_interface::ZIRCON_RUNTIME_ABI_VERSION_V1,
            operation,
            ZrRuntimeOperationPhase::Running,
            0,
            1,
            "queued",
        ),
        output,
    );
    ZrStatus::new(
        ZrStatusCode::NotFound,
        ZrByteSlice::from_static(b"operation handle was not found"),
    )
}

unsafe extern "C" fn fake_drain_empty_owned_output(
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

fn capabilities() -> RuntimeCapabilities {
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

fn api_table() -> ZrRuntimeApiV2 {
    let mut api = ZrRuntimeApiV2::empty();
    api.tick_frame = Some(fake_tick_frame);
    api.handle_event = Some(fake_handle_event);
    api.capture_frame = Some(fake_capture_frame);
    api.subscribe_plugin_event = Some(fake_subscribe_plugin_event);
    api.unsubscribe_plugin_event = Some(fake_unsubscribe_plugin_event);
    api.drain_plugin_events = Some(fake_drain_plugin_events);
    api.submit_operation = Some(fake_submit_operation);
    api.poll_operation = Some(fake_poll_operation);
    api.harvest_operation = Some(fake_harvest_operation);
    api
}

fn gateway(api: ZrRuntimeApiV2) -> SessionGateway {
    let owner: Arc<dyn Send + Sync> = Arc::new(());
    unsafe {
        SessionGateway::new(owner, api, ZrRuntimeSessionHandle::new(17), capabilities())
            .expect("valid session gateway")
    }
}

#[test]
fn session_gateway_rejects_an_invalid_session_handle() {
    let owner: Arc<dyn Send + Sync> = Arc::new(());
    let error = unsafe {
        SessionGateway::new(
            owner,
            api_table(),
            ZrRuntimeSessionHandle::invalid(),
            capabilities(),
        )
        .expect_err("an invalid runtime session cannot back a gateway")
    };

    assert_eq!(error, GatewayError::SessionLost);
}

#[test]
fn session_gateway_rejects_a_foreign_runtime_api_version() {
    let owner: Arc<dyn Send + Sync> = Arc::new(());
    let mut api = api_table();
    api.abi_version += 1;
    let error = unsafe {
        SessionGateway::new(owner, api, ZrRuntimeSessionHandle::new(17), capabilities())
            .expect_err("a foreign runtime API cannot back a gateway")
    };

    assert!(matches!(error, GatewayError::Protocol { .. }));
}

#[test]
fn session_gateway_materializes_canonical_runtime_capabilities() {
    let gateway = gateway(api_table());

    assert_eq!(
        gateway.capabilities().session_profile(),
        SessionProfileKind::Editor
    );
    assert_eq!(
        gateway.capabilities().core_capabilities(),
        &["editor.host.scene_interaction", "editor.host.ui_shell"]
    );
    assert_eq!(gateway.capabilities().plugin_summary().len(), 1);
    assert_eq!(
        gateway.capabilities().plugin_summary()[0].activation(),
        PluginActivationState::Active
    );
}

#[test]
fn session_gateway_rejects_borrowed_world_access_without_calling_runtime() {
    let gateway = gateway(api_table());
    let mut read = |_: &zircon_runtime::scene::World| {};
    let mut write = |_: &mut zircon_runtime::scene::World| {};

    assert_eq!(
        gateway.with_world(&mut read),
        Err(GatewayError::RequiresSerializedAccess)
    );
    assert_eq!(
        gateway.with_world_mut(&mut write),
        Err(GatewayError::RequiresSerializedAccess)
    );
}

#[test]
fn session_gateway_forwards_abi_tick_event_and_frame_calls() {
    TICK_CALLS.store(0, Ordering::SeqCst);
    EVENT_CALLS.store(0, Ordering::SeqCst);
    let gateway = gateway(api_table());

    assert!(gateway.tick_frame().expect("tick runtime session"));
    gateway
        .handle_event(ZrRuntimeEventV1::new(
            zircon_runtime_interface::ZIRCON_RUNTIME_ABI_VERSION_V1,
            0,
            ZrRuntimeViewportHandle::new(3),
        ))
        .expect("forward runtime event");
    let frame = gateway
        .capture_frame(
            ZrRuntimeViewportHandle::new(3),
            ZrRuntimeViewportSizeV1::new(640, 360),
        )
        .expect("capture runtime frame");

    assert_eq!(TICK_CALLS.load(Ordering::SeqCst), 1);
    assert_eq!(EVENT_CALLS.load(Ordering::SeqCst), 1);
    assert_eq!((frame.width(), frame.height()), (640, 360));
    assert!(frame.rgba().is_empty());
}

#[test]
fn session_gateway_reports_missing_optional_pointer_as_typed_capability_error() {
    let gateway = gateway(ZrRuntimeApiV2::empty());

    assert_eq!(
        gateway.tick_frame(),
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.frame.tick",
        })
    );
}

#[test]
fn session_gateway_reports_an_unavailable_optional_profile_control_as_none() {
    let gateway = gateway(api_table());

    let response = gateway
        .profile_control(&ProfileControlRequest {
            command: ProfileControlCommand::Snapshot,
            config: None,
        })
        .expect("query optional runtime profile control");

    assert_eq!(response, None);
}

#[test]
fn session_gateway_forwards_plugin_events_and_operations_and_frees_outputs() {
    let _output_test_guard = OUTPUT_TEST_LOCK.lock().expect("lock output test fixture");
    FREED_OUTPUTS.store(0, Ordering::SeqCst);
    let gateway = gateway(api_table());

    let subscription = gateway
        .subscribe_plugin_event("navigation.path.updated", "zircon.navigation.path.v1")
        .expect("subscribe plugin event")
        .expect("session transport supports plugin events");
    let deliveries = gateway
        .drain_plugin_events(subscription)
        .expect("drain plugin events");
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0].subscription, subscription);
    assert!(gateway
        .unsubscribe_plugin_event(subscription)
        .expect("unsubscribe plugin event"));

    let operation = gateway
        .submit_operation(ZrRuntimeOperationSubmitRequestV1::new(
            zircon_runtime_interface::ZIRCON_RUNTIME_ABI_VERSION_V1,
            "navigation.bake.scene",
            serde_json::Value::Null,
        ))
        .expect("submit operation");
    let progress = gateway.poll_operation(operation).expect("poll operation");
    assert_eq!(progress.phase, ZrRuntimeOperationPhase::Running);
    let result = gateway
        .harvest_operation(operation)
        .expect("harvest operation");
    assert_eq!(
        result.succeeded_output(),
        Some(&serde_json::json!({"meshCount": 4}))
    );
    assert_eq!(FREED_OUTPUTS.load(Ordering::SeqCst), 3);
}

#[test]
fn session_gateway_maps_runtime_not_found_to_runtime_error() {
    let mut api = api_table();
    api.tick_frame = Some(fake_tick_not_found);
    let error = gateway(api).tick_frame().unwrap_err();

    assert!(matches!(error, GatewayError::Runtime { .. }));
    assert!(error.to_string().contains("NotFound"));
}

#[test]
fn session_gateway_releases_invalid_json_and_malformed_owned_outputs() {
    let _output_test_guard = OUTPUT_TEST_LOCK.lock().expect("lock output test fixture");
    FREED_OUTPUTS.store(0, Ordering::SeqCst);
    let operation = ZrRuntimeOperationHandle::new(29);

    let mut invalid_json_api = api_table();
    invalid_json_api.poll_operation = Some(fake_poll_invalid_json);
    assert!(matches!(
        gateway(invalid_json_api).poll_operation(operation),
        Err(GatewayError::Protocol { .. })
    ));

    let mut malformed_api = api_table();
    malformed_api.poll_operation = Some(fake_poll_malformed_buffer);
    assert!(matches!(
        gateway(malformed_api).poll_operation(operation),
        Err(GatewayError::Protocol { .. })
    ));
    assert_eq!(FREED_OUTPUTS.load(Ordering::SeqCst), 2);
}

#[test]
fn session_gateway_releases_output_when_runtime_returns_an_error_status() {
    let _output_test_guard = OUTPUT_TEST_LOCK.lock().expect("lock output test fixture");
    FREED_OUTPUTS.store(0, Ordering::SeqCst);
    let mut api = api_table();
    api.poll_operation = Some(fake_poll_error_with_output);

    let error = gateway(api)
        .poll_operation(ZrRuntimeOperationHandle::new(29))
        .unwrap_err();

    assert!(matches!(error, GatewayError::Runtime { .. }));
    assert_eq!(FREED_OUTPUTS.load(Ordering::SeqCst), 1);
}

#[test]
fn session_gateway_releases_an_empty_owned_plugin_event_batch() {
    let _output_test_guard = OUTPUT_TEST_LOCK.lock().expect("lock output test fixture");
    FREED_OUTPUTS.store(0, Ordering::SeqCst);
    let mut api = api_table();
    api.drain_plugin_events = Some(fake_drain_empty_owned_output);
    let gateway = gateway(api);

    let deliveries = gateway
        .drain_plugin_events(ZrRuntimePluginEventSubscriptionHandle::new(23))
        .expect("empty event delivery batch");

    assert!(deliveries.is_empty());
    assert_eq!(FREED_OUTPUTS.load(Ordering::SeqCst), 1);
}

#[test]
fn session_gateway_keeps_the_runtime_provider_alive() {
    let drops = Arc::new(AtomicUsize::new(0));
    let owner: Arc<dyn Send + Sync> = Arc::new(OwnerDropProbe(drops.clone()));
    let gateway = unsafe {
        SessionGateway::new(
            owner,
            api_table(),
            ZrRuntimeSessionHandle::new(17),
            capabilities(),
        )
        .expect("valid session gateway")
    };

    assert_eq!(drops.load(Ordering::SeqCst), 0);
    drop(gateway);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn session_gateway_copies_frame_bytes_before_the_provider_can_unload() {
    let _output_test_guard = OUTPUT_TEST_LOCK.lock().expect("lock output test fixture");
    FREED_OUTPUTS.store(0, Ordering::SeqCst);
    let drops = Arc::new(AtomicUsize::new(0));
    let owner: Arc<dyn Send + Sync> = Arc::new(OwnerDropProbe(drops.clone()));
    let mut api = api_table();
    api.capture_frame = Some(fake_capture_owned_frame);
    let gateway = unsafe {
        SessionGateway::new(owner, api, ZrRuntimeSessionHandle::new(17), capabilities())
            .expect("valid session gateway")
    };

    let frame = gateway
        .capture_frame(
            ZrRuntimeViewportHandle::new(3),
            ZrRuntimeViewportSizeV1::new(1, 1),
        )
        .expect("copy runtime frame into editor-owned storage");

    assert_eq!(FREED_OUTPUTS.load(Ordering::SeqCst), 1);
    drop(gateway);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert_eq!(frame.abi_version(), 1);
    assert_eq!(frame.generation(), 31);
    assert_eq!(frame.rgba(), &[1, 2, 3, 4]);
}

#[test]
fn runtime_capabilities_preserve_conflicts_in_deterministic_order() {
    let active =
        PluginSummaryEntry::new("zircon.navigation", "1.2.0", PluginActivationState::Active);
    let rejected = PluginSummaryEntry::new(
        "zircon.navigation",
        "1.2.0",
        PluginActivationState::Rejected,
    );
    let left = RuntimeCapabilities::new(
        SessionProfileKind::Editor,
        Vec::<String>::new(),
        [rejected.clone(), active.clone()],
    );
    let right = RuntimeCapabilities::new(
        SessionProfileKind::Editor,
        Vec::<String>::new(),
        [active, rejected],
    );

    assert_eq!(left, right);
    assert_eq!(left.plugin_summary().len(), 2);
    assert_eq!(
        left.plugin_summary()[0].activation(),
        PluginActivationState::Active
    );
    assert_eq!(
        left.plugin_summary()[1].activation(),
        PluginActivationState::Rejected
    );
}
