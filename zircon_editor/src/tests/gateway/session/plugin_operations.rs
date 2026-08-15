use std::sync::atomic::Ordering;

use zircon_runtime_interface::{
    ZrRuntimeOperationHandle, ZrRuntimeOperationPhase, ZrRuntimePluginEventSubscriptionHandle,
    ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1,
};

use crate::core::gateway::{EditorRuntimeGateway, GatewayError};

use super::fixture::{
    api_table, fake_capture_truncated_frame, fake_drain_crossed_plugin_events,
    fake_drain_empty_owned_output, fake_drain_empty_plugin_event_page,
    fake_drain_oversized_plugin_event_page, fake_drain_plugin_event_page_above_delivery_limit,
    fake_harvest_crossed_operation, fake_poll_crossed_operation, fake_poll_error_with_output,
    fake_poll_error_without_output, fake_poll_foreign_abi, fake_poll_unknown_detail,
    fake_poll_unknown_phase, fake_tick_not_found, gateway, FREED_OUTPUTS, OUTPUT_TEST_LOCK,
};

#[test]
fn session_gateway_forwards_plugin_events_and_operations_and_frees_outputs() {
    let _output_test_guard = OUTPUT_TEST_LOCK.lock().expect("lock output test fixture");
    FREED_OUTPUTS.store(0, Ordering::SeqCst);
    let gateway = gateway(api_table());

    let subscription = gateway
        .subscribe_plugin_event("navigation.path.updated", "zircon.navigation.path.v1")
        .expect("subscribe plugin event")
        .expect("session transport supports plugin events");
    let page = gateway
        .drain_plugin_events(subscription)
        .expect("drain plugin events");
    assert_eq!(page.deliveries().len(), 1);
    assert_eq!(page.deliveries()[0].subscription, subscription);
    assert!(page.encoded_bytes() > 0);
    assert!(page.encoded_bytes() <= ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1);
    assert_eq!(page.runtime_remaining_deliveries(), 9);
    assert_eq!(page.runtime_oldest_pending_age_millis(), 17);
    assert!(gateway
        .unsubscribe_plugin_event(subscription)
        .expect("unsubscribe plugin event"));

    let operation = gateway
        .submit_operation(
            zircon_runtime_interface::ZrRuntimeOperationSubmitRequestV1::new(
                zircon_runtime_interface::ZIRCON_RUNTIME_ABI_VERSION_V1,
                "navigation.bake.scene",
                serde_json::Value::Null,
            ),
        )
        .expect("submit operation");
    let status = gateway.poll_operation(operation).expect("poll operation");
    assert_eq!(status.phase(), Some(ZrRuntimeOperationPhase::ReadyToApply));
    let result = gateway
        .harvest_operation(operation)
        .expect("harvest operation");
    assert_eq!(
        result.succeeded_output(),
        Some(&serde_json::json!({"meshCount": 4}))
    );
    assert_eq!(FREED_OUTPUTS.load(Ordering::SeqCst), 2);
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
fn session_gateway_rejects_foreign_abi_and_unknown_fixed_status_phase() {
    let _output_test_guard = OUTPUT_TEST_LOCK.lock().expect("lock output test fixture");
    FREED_OUTPUTS.store(0, Ordering::SeqCst);
    let operation = ZrRuntimeOperationHandle::new(29);

    let mut foreign_abi_api = api_table();
    foreign_abi_api.poll_operation = Some(fake_poll_foreign_abi);
    assert!(matches!(
        gateway(foreign_abi_api).poll_operation(operation),
        Err(GatewayError::Protocol { .. })
    ));

    let mut unknown_phase_api = api_table();
    unknown_phase_api.poll_operation = Some(fake_poll_unknown_phase);
    assert!(matches!(
        gateway(unknown_phase_api).poll_operation(operation),
        Err(GatewayError::Protocol { .. })
    ));
    assert_eq!(FREED_OUTPUTS.load(Ordering::SeqCst), 0);
}

#[test]
fn session_gateway_rejects_unknown_detail_and_error_status_without_output() {
    let _output_test_guard = OUTPUT_TEST_LOCK.lock().expect("lock output test fixture");
    FREED_OUTPUTS.store(0, Ordering::SeqCst);
    let operation = ZrRuntimeOperationHandle::new(29);

    let mut unknown_detail_api = api_table();
    unknown_detail_api.poll_operation = Some(fake_poll_unknown_detail);
    assert!(matches!(
        gateway(unknown_detail_api).poll_operation(operation),
        Err(GatewayError::Protocol { .. })
    ));

    let mut error_status_api = api_table();
    error_status_api.poll_operation = Some(fake_poll_error_without_output);
    assert!(matches!(
        gateway(error_status_api).poll_operation(operation),
        Err(GatewayError::Protocol { .. })
    ));
    assert_eq!(FREED_OUTPUTS.load(Ordering::SeqCst), 0);
}

#[test]
fn session_gateway_rejects_crossed_response_identity_and_truncated_frame_payloads() {
    let _output_test_guard = OUTPUT_TEST_LOCK.lock().expect("lock output test fixture");
    FREED_OUTPUTS.store(0, Ordering::SeqCst);
    let subscription = ZrRuntimePluginEventSubscriptionHandle::new(23);
    let operation = ZrRuntimeOperationHandle::new(29);

    let mut frame_api = api_table();
    frame_api.capture_frame = Some(fake_capture_truncated_frame);
    assert!(matches!(
        gateway(frame_api).capture_frame(
            ZrRuntimeViewportHandle::new(3),
            ZrRuntimeViewportSizeV1::new(2, 2),
        ),
        Err(GatewayError::Protocol { .. })
    ));

    let mut drain_api = api_table();
    drain_api.drain_plugin_events = Some(fake_drain_crossed_plugin_events);
    assert!(matches!(
        gateway(drain_api).drain_plugin_events(subscription),
        Err(GatewayError::Protocol { .. })
    ));

    let mut poll_api = api_table();
    poll_api.poll_operation = Some(fake_poll_crossed_operation);
    assert!(matches!(
        gateway(poll_api).poll_operation(operation),
        Err(GatewayError::Protocol { .. })
    ));

    let mut harvest_api = api_table();
    harvest_api.harvest_operation = Some(fake_harvest_crossed_operation);
    assert!(matches!(
        gateway(harvest_api).harvest_operation(operation),
        Err(GatewayError::Protocol { .. })
    ));
    assert_eq!(FREED_OUTPUTS.load(Ordering::SeqCst), 3);
}

#[test]
fn session_gateway_ignores_fixed_status_output_when_runtime_returns_an_error() {
    let _output_test_guard = OUTPUT_TEST_LOCK.lock().expect("lock output test fixture");
    FREED_OUTPUTS.store(0, Ordering::SeqCst);
    let mut api = api_table();
    api.poll_operation = Some(fake_poll_error_with_output);

    let error = gateway(api)
        .poll_operation(ZrRuntimeOperationHandle::new(29))
        .unwrap_err();

    assert!(matches!(error, GatewayError::Runtime { .. }));
    assert_eq!(FREED_OUTPUTS.load(Ordering::SeqCst), 0);
}

#[test]
fn session_gateway_releases_an_empty_owned_plugin_event_batch() {
    let _output_test_guard = OUTPUT_TEST_LOCK.lock().expect("lock output test fixture");
    FREED_OUTPUTS.store(0, Ordering::SeqCst);
    let mut api = api_table();
    api.drain_plugin_events = Some(fake_drain_empty_owned_output);
    let gateway = gateway(api);

    let page = gateway
        .drain_plugin_events(ZrRuntimePluginEventSubscriptionHandle::new(23))
        .expect("empty event delivery batch");

    assert!(page.is_empty());
    assert_eq!(page.encoded_bytes(), 0);
    assert_eq!(FREED_OUTPUTS.load(Ordering::SeqCst), 1);
}

#[test]
fn session_gateway_accepts_an_empty_plugin_event_buffer_without_a_free_callback() {
    let _output_test_guard = OUTPUT_TEST_LOCK.lock().expect("lock output test fixture");
    FREED_OUTPUTS.store(0, Ordering::SeqCst);
    let mut api = api_table();
    api.drain_plugin_events = Some(fake_drain_empty_plugin_event_page);
    let gateway = gateway(api);

    let page = gateway
        .drain_plugin_events(ZrRuntimePluginEventSubscriptionHandle::new(23))
        .expect("empty runtime event page");

    assert!(page.is_empty());
    assert_eq!(page.encoded_bytes(), 0);
    assert_eq!(FREED_OUTPUTS.load(Ordering::SeqCst), 0);
}

#[test]
fn session_gateway_rejects_plugin_event_pages_above_the_abi_limits() {
    let _output_test_guard = OUTPUT_TEST_LOCK.lock().expect("lock output test fixture");
    FREED_OUTPUTS.store(0, Ordering::SeqCst);
    let subscription = ZrRuntimePluginEventSubscriptionHandle::new(23);

    let mut oversized_wire_api = api_table();
    oversized_wire_api.drain_plugin_events = Some(fake_drain_oversized_plugin_event_page);
    let wire_error = gateway(oversized_wire_api)
        .drain_plugin_events(subscription)
        .expect_err("oversized event page must be rejected before JSON decoding");
    assert!(matches!(wire_error, GatewayError::Protocol { .. }));
    assert!(wire_error.to_string().contains("encoded bytes"));

    let mut oversized_delivery_api = api_table();
    oversized_delivery_api.drain_plugin_events =
        Some(fake_drain_plugin_event_page_above_delivery_limit);
    let delivery_error = gateway(oversized_delivery_api)
        .drain_plugin_events(subscription)
        .expect_err("event page above the delivery limit must be rejected");
    assert!(matches!(delivery_error, GatewayError::Protocol { .. }));
    assert!(delivery_error.to_string().contains("deliveries"));
    assert_eq!(FREED_OUTPUTS.load(Ordering::SeqCst), 2);
}
