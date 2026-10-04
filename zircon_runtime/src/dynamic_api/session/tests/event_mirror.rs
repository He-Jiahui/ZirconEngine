use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use serde::Serialize;
use serde_json::json;

use super::super::profile::RuntimeDynamicSessionProfile;
use super::*;
use crate::scene::{
    RuntimeEventMirrorRegistration, SceneError, RUNTIME_EVENT_MIRROR_PAGE_MAX_EVENTS,
    RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES,
};

const SEQUENCE_WINDOW_EVENT_ID: &str = "dynamic_api.plugin_event.sequence_window";
const SEQUENCE_WINDOW_PAYLOAD_SCHEMA: &str = "zircon.dynamic_api.plugin_event.sequence_window.v1";

#[derive(Clone, Debug, Serialize)]
struct SequenceWindowEvent {
    value: u8,
}

#[derive(Clone, Debug, Serialize)]
struct DeepPayloadEvent {
    payload: serde_json::Value,
}

fn sequence_window_session() -> (
    RuntimeDynamicSession,
    ZrRuntimePluginEventSubscriptionHandle,
) {
    let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless dynamic session");
    session.level.with_world_mut(|world| {
        world
            .register_runtime_event_mirror(RuntimeEventMirrorRegistration::typed::<
                SequenceWindowEvent,
            >(
                SEQUENCE_WINDOW_EVENT_ID,
                SEQUENCE_WINDOW_PAYLOAD_SCHEMA,
            ))
            .expect("sequence-window event mirror registration");
    });
    let subscription = session
        .subscribe_plugin_event(ZrRuntimePluginEventSubscribeRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            SEQUENCE_WINDOW_EVENT_ID,
            SEQUENCE_WINDOW_PAYLOAD_SCHEMA,
        ))
        .expect("sequence-window plugin subscription");
    (session, subscription)
}

#[test]
fn dropping_dynamic_session_quiesces_plugin_event_mirrors() {
    let readers = Arc::new(AtomicU32::new(0));
    let readers_for_registration = Arc::clone(&readers);
    let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless dynamic session");
    session.level.with_world_mut(|world| {
        world
            .register_runtime_event_mirror(
                RuntimeEventMirrorRegistration::typed::<SequenceWindowEvent>(
                    SEQUENCE_WINDOW_EVENT_ID,
                    SEQUENCE_WINDOW_PAYLOAD_SCHEMA,
                )
                .with_reader_count_callback(move |_world, count| {
                    readers_for_registration.store(count, Ordering::SeqCst);
                    Ok(())
                }),
            )
            .expect("session-drop event mirror registration");
    });
    session
        .subscribe_plugin_event(ZrRuntimePluginEventSubscribeRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            SEQUENCE_WINDOW_EVENT_ID,
            SEQUENCE_WINDOW_PAYLOAD_SCHEMA,
        ))
        .expect("session-drop plugin subscription");
    assert_eq!(readers.load(Ordering::SeqCst), 1);

    drop(session);

    assert_eq!(readers.load(Ordering::SeqCst), 0);
}

#[test]
fn dynamic_session_shutdown_reports_event_mirror_callback_failure_until_retry_succeeds() {
    let fail_zero = Arc::new(AtomicBool::new(true));
    let fail_zero_for_registration = Arc::clone(&fail_zero);
    let readers = Arc::new(AtomicU32::new(0));
    let readers_for_registration = Arc::clone(&readers);
    let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless dynamic session");
    session.level.with_world_mut(|world| {
        world
            .register_runtime_event_mirror(
                RuntimeEventMirrorRegistration::typed::<SequenceWindowEvent>(
                    SEQUENCE_WINDOW_EVENT_ID,
                    SEQUENCE_WINDOW_PAYLOAD_SCHEMA,
                )
                .with_reader_count_callback(move |_world, count| {
                    if count == 0 && fail_zero_for_registration.load(Ordering::SeqCst) {
                        return Err(SceneError::EmptyNodeName);
                    }
                    readers_for_registration.store(count, Ordering::SeqCst);
                    Ok(())
                }),
            )
            .expect("session-shutdown event mirror registration");
    });
    session
        .subscribe_plugin_event(ZrRuntimePluginEventSubscribeRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            SEQUENCE_WINDOW_EVENT_ID,
            SEQUENCE_WINDOW_PAYLOAD_SCHEMA,
        ))
        .expect("session-shutdown plugin subscription");
    assert_eq!(readers.load(Ordering::SeqCst), 1);

    assert!(!session.shutdown_before_library_unload());
    assert_eq!(readers.load(Ordering::SeqCst), 1);

    fail_zero.store(false, Ordering::SeqCst);
    assert!(session.shutdown_before_library_unload());
    assert_eq!(readers.load(Ordering::SeqCst), 0);
}

#[test]
fn empty_plugin_event_page_uses_an_empty_owned_buffer() {
    let batch = ZrRuntimePluginEventDeliveryBatchV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);

    let buffer = encode_plugin_event_batch(&batch).unwrap();

    assert!(buffer.is_empty());
}

#[test]
fn plugin_event_drain_uses_only_available_sequence_headroom() {
    let (mut session, subscription) = sequence_window_session();
    session.level.with_world_mut(|world| {
        assert!(world.send_event(SequenceWindowEvent { value: 1 }));
        assert!(world.send_event(SequenceWindowEvent { value: 2 }));
        world.update_events::<SequenceWindowEvent>();
    });
    session
        .plugin_event_subscriptions
        .get_mut(&subscription.raw())
        .expect("sequence-window plugin subscription state")
        .sequence = u64::MAX - 2;

    let buffer = session
        .prepare_plugin_event_output(7, subscription)
        .expect("two deliveries fit within remaining sequence headroom");
    let batch = serde_json::from_slice::<ZrRuntimePluginEventDeliveryBatchV1>(&buffer)
        .expect("sequence-window event page");
    assert_eq!(batch.deliveries.len(), 2);
    assert_eq!(batch.deliveries[0].sequence, u64::MAX - 1);
    assert_eq!(batch.deliveries[1].sequence, u64::MAX);
    assert_eq!(batch.remaining_deliveries, 0);
    session.commit_plugin_event_output(subscription);

    let idle = session
        .prepare_plugin_event_output(7, subscription)
        .expect("an idle page at the maximum sequence remains representable");
    assert!(idle.is_empty());
    session.commit_plugin_event_output(subscription);
}

#[test]
fn invalid_plugin_event_payload_is_reported_once_without_blocking_later_deliveries() {
    let event_id = "dynamic_api.plugin_event.retry";
    let payload_schema = "zircon.dynamic_api.plugin_event.retry.v1";
    let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless dynamic session");
    session.level.with_world_mut(|world| {
        world
            .register_runtime_event_mirror(
                RuntimeEventMirrorRegistration::typed::<DeepPayloadEvent>(event_id, payload_schema),
            )
            .expect("deep-payload event mirror registration");
    });
    let subscription = session
        .subscribe_plugin_event(ZrRuntimePluginEventSubscribeRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            event_id,
            payload_schema,
        ))
        .expect("deep-payload plugin subscription");
    let mut payload = serde_json::Value::Null;
    for _ in 0..ZR_RUNTIME_PLUGIN_EVENT_OUTPUT_LIMIT_V1.max_nesting_depth {
        payload = serde_json::Value::Array(vec![payload]);
    }
    session.level.with_world_mut(|world| {
        assert!(!world.send_event(DeepPayloadEvent { payload }));
        assert!(world.send_event(DeepPayloadEvent {
            payload: serde_json::Value::String("next".to_string()),
        }));
        world.update_events::<DeepPayloadEvent>();
    });

    let error = session
        .prepare_plugin_event_output(7, subscription)
        .expect_err("the nested event must exceed the bounded output depth");
    assert!(error.is_limit_exceeded());
    let state = session
        .plugin_event_subscriptions
        .get(&subscription.raw())
        .expect("retry subscription state");
    assert_eq!(state.sequence, 0);
    assert!(state.pending_page.is_none());
    assert!(!state.output_in_flight);

    let bytes = session
        .prepare_plugin_event_output(7, subscription)
        .expect("the valid delivery behind the rejected payload must make progress");
    let batch = serde_json::from_slice::<ZrRuntimePluginEventDeliveryBatchV1>(&bytes)
        .expect("valid plugin event delivery page");
    assert_eq!(batch.deliveries.len(), 1);
    assert_eq!(batch.deliveries[0].sequence, 1);
    session.commit_plugin_event_output(subscription);
}

#[test]
fn encoded_plugin_event_page_has_a_hard_wire_ceiling() {
    assert_eq!(
        RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES,
        ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1
    );
    assert_eq!(
        RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES,
        RUNTIME_EVENT_MIRROR_PAGE_MAX_EVENTS
    );
    assert_eq!(
        RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES,
        ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1
    );
    let event_id = "\"".repeat(128);
    let payload_schema = "\\".repeat(128);
    let payload = json!({ "payload": "x".repeat(1_900) });
    let payload_bytes = serde_json::to_vec(&payload).unwrap().len();
    assert!(
        payload_bytes * RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES
            <= RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES
    );
    let deliveries = (1..=RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES)
        .map(|sequence| {
            ZrRuntimePluginEventDeliveryV1::new(
                u64::MAX,
                ZrRuntimePluginEventSubscriptionHandle::new(u64::MAX),
                event_id.clone(),
                payload_schema.clone(),
                sequence as u64,
                payload.clone(),
            )
        })
        .collect();
    let batch = ZrRuntimePluginEventDeliveryBatchV1::new(ZIRCON_RUNTIME_ABI_VERSION_V1, deliveries);

    let buffer = encode_plugin_event_batch(&batch).unwrap();
    assert!(buffer.len() <= RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES);

    let oversized = ZrRuntimePluginEventDeliveryBatchV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        vec![ZrRuntimePluginEventDeliveryV1::new(
            1,
            ZrRuntimePluginEventSubscriptionHandle::new(1),
            "event",
            "schema",
            1,
            serde_json::Value::String("x".repeat(RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES)),
        )],
    );
    assert!(encode_plugin_event_batch(&oversized).is_err());
}

#[test]
fn encoded_plugin_event_full_page_with_maximum_descriptor_escaping_fits_wire_ceiling() {
    let event_id = "\0".repeat(128);
    let payload_schema = "\0".repeat(128);
    let payload_text_bytes =
        RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES / RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES - 2;
    let payload = serde_json::Value::String("x".repeat(payload_text_bytes));
    assert_eq!(
        serde_json::to_vec(&payload).unwrap().len() * RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES,
        RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES
    );
    let deliveries = (1..=RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES)
        .map(|sequence| {
            ZrRuntimePluginEventDeliveryV1::new(
                u64::MAX,
                ZrRuntimePluginEventSubscriptionHandle::new(u64::MAX),
                event_id.clone(),
                payload_schema.clone(),
                sequence as u64,
                payload.clone(),
            )
        })
        .collect();
    let batch = ZrRuntimePluginEventDeliveryBatchV1::new(ZIRCON_RUNTIME_ABI_VERSION_V1, deliveries)
        .with_runtime_backlog(u32::MAX, u64::MAX);

    let buffer = encode_plugin_event_batch(&batch).unwrap();

    assert!(buffer.len() <= RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES);
}

#[test]
fn encoded_plugin_event_page_accepts_the_largest_scene_payload_within_wire_ceiling() {
    let event_id = "\0".repeat(128);
    let payload_schema = "\0".repeat(128);
    let payload =
        serde_json::Value::String("x".repeat(RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES - 2));
    assert_eq!(
        serde_json::to_vec(&payload).unwrap().len(),
        RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES
    );
    let batch = ZrRuntimePluginEventDeliveryBatchV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        vec![ZrRuntimePluginEventDeliveryV1::new(
            u64::MAX,
            ZrRuntimePluginEventSubscriptionHandle::new(u64::MAX),
            event_id,
            payload_schema,
            u64::MAX,
            payload,
        )],
    );

    let buffer = encode_plugin_event_batch(&batch).unwrap();
    assert!(buffer.len() <= RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES);
}
