use std::collections::HashMap;
use std::io::Write;
use std::time::{Duration, Instant};

use zircon_runtime_interface::{
    ZrRuntimePluginEventDeliveryBatchV1, ZrRuntimePluginEventSubscribeRequestV1,
    ZrRuntimePluginEventSubscriptionHandle, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZR_RUNTIME_PLUGIN_EVENT_OUTPUT_LIMIT_V1, ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1,
};

#[cfg(test)]
use zircon_runtime_interface::ZrRuntimePluginEventDeliveryV1;

use crate::scene::{
    RuntimeEventMirrorDrainPage, RuntimeEventMirrorError, RuntimeEventMirrorPayload,
    RuntimeEventMirrorSubscription,
};

use super::super::bounded_json::{self, BoundedJsonError, BoundedJsonWriter};
use super::RuntimeDynamicSession;

pub(in crate::dynamic_api) const RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES: usize =
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1;
pub(in crate::dynamic_api) const RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES: usize =
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1;

pub(super) struct RuntimePluginEventSubscriptionState {
    subscription: RuntimeEventMirrorSubscription,
    sequence: u64,
    pending_page: Option<RuntimeEventMirrorDrainPage>,
    in_flight_delivery_count: usize,
    output_in_flight: bool,
}

impl RuntimeDynamicSession {
    pub(super) fn shutdown_plugin_event_subscriptions(&mut self) -> bool {
        let subscriptions = std::mem::take(&mut self.plugin_event_subscriptions);
        drop(subscriptions);
        self.level
            .with_world_mut(|world| world.shutdown_runtime_event_mirrors())
            .retry_pending
            == 0
    }

    pub(super) fn subscribe_plugin_event(
        &mut self,
        request: ZrRuntimePluginEventSubscribeRequestV1,
    ) -> Result<ZrRuntimePluginEventSubscriptionHandle, String> {
        let handle_raw = self.next_plugin_event_subscription.max(1);
        let next_handle = handle_raw
            .checked_add(1)
            .ok_or_else(|| "runtime plugin event subscription handle overflowed".to_string())?;
        let subscription = self
            .level
            .with_world_mut(|world| {
                world.subscribe_runtime_event_mirror(&request.event_id, &request.payload_schema)
            })
            .map_err(|error| error.to_string())?;
        let handle = ZrRuntimePluginEventSubscriptionHandle::new(handle_raw);
        self.next_plugin_event_subscription = next_handle;
        self.plugin_event_subscriptions.insert(
            handle.raw(),
            RuntimePluginEventSubscriptionState {
                subscription,
                sequence: 0,
                pending_page: None,
                in_flight_delivery_count: 0,
                output_in_flight: false,
            },
        );
        Ok(handle)
    }

    pub(super) fn unsubscribe_plugin_event(
        &mut self,
        handle: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<(), String> {
        let state = self
            .plugin_event_subscriptions
            .get_mut(&handle.raw())
            .ok_or_else(|| "runtime plugin event subscription not found".to_string())?;
        if state.output_in_flight {
            return Err("runtime plugin event output is already in flight".to_string());
        }
        let disconnected = self
            .level
            .with_world_mut(|world| world.unsubscribe_runtime_event_mirror(&mut state.subscription))
            .map_err(|error| error.to_string())?;
        if !disconnected {
            return Err("runtime did not disconnect the plugin event subscription".to_string());
        }
        self.plugin_event_subscriptions.remove(&handle.raw());
        Ok(())
    }

    pub(super) fn prepare_plugin_event_output(
        &mut self,
        play_session_id: u64,
        handle: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<Vec<u8>, BoundedJsonError> {
        let state = self
            .plugin_event_subscriptions
            .get_mut(&handle.raw())
            .ok_or_else(|| {
                BoundedJsonError::Json("runtime plugin event subscription not found".to_string())
            })?;
        if state.output_in_flight {
            return Err(BoundedJsonError::Json(
                "runtime plugin event output is already in flight".to_string(),
            ));
        }
        let delivery_limit = usize::try_from(u64::MAX - state.sequence)
            .unwrap_or(usize::MAX)
            .min(RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES);
        if state.pending_page.is_none() {
            state.pending_page = Some(
                self.level
                    .with_world(|world| {
                        world.drain_runtime_event_mirror_payloads(
                            &mut state.subscription,
                            delivery_limit,
                        )
                    })
                    .map_err(plugin_event_queue_error)?,
            );
        }
        let page = state
            .pending_page
            .as_ref()
            .expect("pending plugin event page was initialized");
        if page.payloads.is_empty() {
            if page.remaining_deliveries > 0 && delivery_limit == 0 {
                return Err(BoundedJsonError::Json(
                    "runtime plugin event sequence overflowed".to_string(),
                ));
            }
            state.in_flight_delivery_count = 0;
            state.output_in_flight = true;
            return Ok(Vec::new());
        }

        let descriptor = state.subscription.descriptor();
        let (bytes, delivery_count) = encode_largest_plugin_event_prefix(
            play_session_id,
            handle,
            state.sequence,
            &descriptor.event_id,
            &descriptor.payload_schema,
            page,
        )?;
        state.in_flight_delivery_count = delivery_count;
        state.output_in_flight = true;
        Ok(bytes)
    }

    pub(super) fn commit_plugin_event_output(
        &mut self,
        handle: ZrRuntimePluginEventSubscriptionHandle,
    ) {
        let state = self
            .plugin_event_subscriptions
            .get_mut(&handle.raw())
            .expect("an in-flight plugin event output must retain its subscription");
        debug_assert!(state.output_in_flight);
        let delivery_count = state.in_flight_delivery_count;
        state.sequence = state
            .sequence
            .checked_add(delivery_count as u64)
            .expect("plugin event sequence was preflighted before output registration");
        let page = state
            .pending_page
            .as_mut()
            .expect("an in-flight plugin event output must retain its page");
        page.payloads.drain(..delivery_count);
        if page.payloads.is_empty() {
            state.pending_page = None;
        }
        state.in_flight_delivery_count = 0;
        state.output_in_flight = false;
    }

    pub(super) fn rollback_plugin_event_output(
        &mut self,
        handle: ZrRuntimePluginEventSubscriptionHandle,
    ) {
        let state = self
            .plugin_event_subscriptions
            .get_mut(&handle.raw())
            .expect("an in-flight plugin event output must retain its subscription");
        debug_assert!(state.output_in_flight);
        state.in_flight_delivery_count = 0;
        state.output_in_flight = false;
    }
}

fn plugin_event_queue_error(error: RuntimeEventMirrorError) -> BoundedJsonError {
    match error {
        RuntimeEventMirrorError::PayloadTooLarge {
            payload_bytes,
            max_payload_bytes,
            ..
        } => BoundedJsonError::EncodedBytes {
            observed: payload_bytes,
            limit: max_payload_bytes,
        },
        RuntimeEventMirrorError::PayloadTooDeep {
            observed_depth,
            max_depth,
            ..
        } => BoundedJsonError::NestingDepth {
            observed: observed_depth.saturating_add(3),
            limit: max_depth.saturating_add(3),
        },
        RuntimeEventMirrorError::ProcessingTime { limit_micros, .. } => {
            BoundedJsonError::ProcessingTime { limit_micros }
        }
        error => BoundedJsonError::Json(error.to_string()),
    }
}

fn encode_largest_plugin_event_prefix(
    play_session_id: u64,
    handle: ZrRuntimePluginEventSubscriptionHandle,
    sequence: u64,
    event_id: &str,
    payload_schema: &str,
    page: &RuntimeEventMirrorDrainPage,
) -> Result<(Vec<u8>, usize), BoundedJsonError> {
    let started = Instant::now();
    let delivery_count = page.payloads.len();
    match encode_plugin_event_prefix(
        play_session_id,
        handle,
        sequence,
        event_id,
        payload_schema,
        &page.payloads,
        page.remaining_deliveries,
        page.oldest_pending_age_millis,
        started,
    ) {
        Ok(bytes) => return Ok((bytes, delivery_count)),
        Err(error) if !deterministic_plugin_event_payload_failure(&error) => return Err(error),
        Err(_) => {}
    }

    let first = encode_plugin_event_prefix(
        play_session_id,
        handle,
        sequence,
        event_id,
        payload_schema,
        &page.payloads[..1],
        page.remaining_deliveries
            .saturating_add(u32::try_from(delivery_count - 1).unwrap_or(u32::MAX)),
        page.oldest_pending_age_millis,
        started,
    )?;
    let mut best = (first, 1_usize);
    let mut low = 2_usize;
    let mut high = delivery_count;
    while low < high {
        let candidate = low + (high - low) / 2;
        let remaining = page
            .remaining_deliveries
            .saturating_add(u32::try_from(delivery_count - candidate).unwrap_or(u32::MAX));
        match encode_plugin_event_prefix(
            play_session_id,
            handle,
            sequence,
            event_id,
            payload_schema,
            &page.payloads[..candidate],
            remaining,
            page.oldest_pending_age_millis,
            started,
        ) {
            Ok(bytes) => {
                best = (bytes, candidate);
                low = candidate + 1;
            }
            Err(error) if deterministic_plugin_event_payload_failure(&error) => {
                high = candidate;
            }
            Err(error) => return Err(error),
        }
    }
    Ok(best)
}

fn encode_plugin_event_prefix(
    play_session_id: u64,
    handle: ZrRuntimePluginEventSubscriptionHandle,
    sequence: u64,
    event_id: &str,
    payload_schema: &str,
    payloads: &[RuntimeEventMirrorPayload],
    remaining_deliveries: u32,
    oldest_pending_age_millis: u64,
    started: Instant,
) -> Result<Vec<u8>, BoundedJsonError> {
    check_plugin_event_encoding_deadline(started)?;
    let payload_capacity = payloads
        .iter()
        .map(|payload| payload.json_bytes().len())
        .sum::<usize>();
    let mut bytes =
        BoundedJsonWriter::with_capacity(ZR_RUNTIME_PLUGIN_EVENT_OUTPUT_LIMIT_V1, payload_capacity);
    let result = (|| -> std::io::Result<()> {
        bytes.write_all(br#"{"abiVersion":"#)?;
        write_json_integer(&mut bytes, u64::from(ZIRCON_RUNTIME_ABI_VERSION_V1))?;
        bytes.write_all(br#","deliveries":["#)?;
        for (index, payload) in payloads.iter().enumerate() {
            if index != 0 {
                bytes.write_all(b",")?;
            }
            let sequence = sequence
                .checked_add(index as u64 + 1)
                .expect("runtime plugin event page sequence was preflighted");
            bytes.write_all(br#"{"playSessionId":"#)?;
            write_json_integer(&mut bytes, play_session_id)?;
            bytes.write_all(br#","subscription":"#)?;
            write_json_integer(&mut bytes, handle.raw())?;
            bytes.write_all(br#","eventId":"#)?;
            serde_json::to_writer(&mut bytes, event_id).map_err(std::io::Error::other)?;
            bytes.write_all(br#","payloadSchema":"#)?;
            serde_json::to_writer(&mut bytes, payload_schema).map_err(std::io::Error::other)?;
            bytes.write_all(br#","sequence":"#)?;
            write_json_integer(&mut bytes, sequence)?;
            bytes.write_all(br#","payload":"#)?;
            bytes.write_all(payload.json_bytes())?;
            bytes.write_all(b"}")?;
        }
        bytes.write_all(br#"],"remainingDeliveries":"#)?;
        write_json_integer(&mut bytes, u64::from(remaining_deliveries))?;
        bytes.write_all(br#","oldestPendingAgeMillis":"#)?;
        write_json_integer(&mut bytes, oldest_pending_age_millis)?;
        bytes.write_all(b"}")
    })();
    let bytes = bytes.finish_io_result(result)?;
    check_plugin_event_encoding_deadline(started)?;
    Ok(bytes)
}

fn check_plugin_event_encoding_deadline(started: Instant) -> Result<(), BoundedJsonError> {
    let limit_micros = ZR_RUNTIME_PLUGIN_EVENT_OUTPUT_LIMIT_V1.max_processing_time_micros;
    if started.elapsed() > Duration::from_micros(limit_micros) {
        return Err(BoundedJsonError::ProcessingTime { limit_micros });
    }
    Ok(())
}

fn deterministic_plugin_event_payload_failure(error: &BoundedJsonError) -> bool {
    matches!(
        error,
        BoundedJsonError::EncodedBytes { .. }
            | BoundedJsonError::Items { .. }
            | BoundedJsonError::NestingDepth { .. }
    )
}

pub(super) fn empty_plugin_event_subscriptions() -> HashMap<u64, RuntimePluginEventSubscriptionState>
{
    HashMap::new()
}

pub(super) fn encode_plugin_event_batch(
    batch: &ZrRuntimePluginEventDeliveryBatchV1,
) -> Result<Vec<u8>, BoundedJsonError> {
    if batch.deliveries.is_empty() {
        return Ok(Vec::new());
    }
    bounded_json::encode(batch, ZR_RUNTIME_PLUGIN_EVENT_OUTPUT_LIMIT_V1, || {
        batch.deliveries.len()
    })
}

fn write_json_integer(bytes: &mut impl Write, value: u64) -> std::io::Result<()> {
    write!(bytes, "{value}")
}

#[cfg(test)]
mod tests {
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
    const SEQUENCE_WINDOW_PAYLOAD_SCHEMA: &str =
        "zircon.dynamic_api.plugin_event.sequence_window.v1";

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
                .register_runtime_event_mirror(RuntimeEventMirrorRegistration::typed::<
                    DeepPayloadEvent,
                >(event_id, payload_schema))
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
        let batch =
            ZrRuntimePluginEventDeliveryBatchV1::new(ZIRCON_RUNTIME_ABI_VERSION_V1, deliveries);

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
        let payload_text_bytes = RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES
            / RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES
            - 2;
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
        let batch =
            ZrRuntimePluginEventDeliveryBatchV1::new(ZIRCON_RUNTIME_ABI_VERSION_V1, deliveries)
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
}
