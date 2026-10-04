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
    event_id_json: Box<[u8]>,
    payload_schema_json: Box<[u8]>,
    sequence: u64,
    pending_page: Option<RuntimeEventMirrorDrainPage>,
    in_flight_delivery_count: usize,
    output_in_flight: bool,
}

impl RuntimeDynamicSession {
    pub(super) fn shutdown_plugin_event_subscriptions(&mut self) -> bool {
        let Some(deadline) = Instant::now().checked_add(Duration::from_secs(5)) else {
            return false;
        };
        self.shutdown_plugin_event_subscriptions_until(deadline)
    }

    pub(super) fn shutdown_plugin_event_subscriptions_until(&mut self, deadline: Instant) -> bool {
        let subscriptions = std::mem::take(&mut self.plugin_event_subscriptions);
        drop(subscriptions);
        self.level
            .with_world_mut(|world| world.shutdown_runtime_event_mirrors_until(deadline))
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
        let event_id_json = encode_plugin_event_descriptor(&request.event_id)?;
        let payload_schema_json = encode_plugin_event_descriptor(&request.payload_schema)?;
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
                event_id_json,
                payload_schema_json,
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

        let (bytes, delivery_count) = encode_largest_plugin_event_prefix(
            play_session_id,
            handle,
            state.sequence,
            &state.event_id_json,
            &state.payload_schema_json,
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

fn encode_plugin_event_descriptor(value: &str) -> Result<Box<[u8]>, String> {
    serde_json::to_vec(value)
        .map(Vec::into_boxed_slice)
        .map_err(|error| format!("failed to encode runtime plugin event descriptor: {error}"))
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
    event_id_json: &[u8],
    payload_schema_json: &[u8],
    page: &RuntimeEventMirrorDrainPage,
) -> Result<(Vec<u8>, usize), BoundedJsonError> {
    let started = Instant::now();
    let delivery_count = page.payloads.len();
    match encode_plugin_event_prefix(
        play_session_id,
        handle,
        sequence,
        event_id_json,
        payload_schema_json,
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
        event_id_json,
        payload_schema_json,
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
            event_id_json,
            payload_schema_json,
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
    event_id_json: &[u8],
    payload_schema_json: &[u8],
    payloads: &[RuntimeEventMirrorPayload],
    remaining_deliveries: u32,
    oldest_pending_age_millis: u64,
    started: Instant,
) -> Result<Vec<u8>, BoundedJsonError> {
    check_plugin_event_encoding_deadline(started)?;
    crate::profile_counter!("runtime", "plugin_event.page_encode_attempt", 1);
    let payload_capacity = payloads
        .iter()
        .map(|payload| payload.json_bytes().len())
        .sum::<usize>();
    let descriptor_capacity = event_id_json
        .len()
        .saturating_add(payload_schema_json.len())
        .saturating_mul(payloads.len());
    let mut bytes = BoundedJsonWriter::with_capacity(
        ZR_RUNTIME_PLUGIN_EVENT_OUTPUT_LIMIT_V1,
        payload_capacity.saturating_add(descriptor_capacity),
    );
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
            bytes.write_all(event_id_json)?;
            bytes.write_all(br#","payloadSchema":"#)?;
            bytes.write_all(payload_schema_json)?;
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
#[path = "tests/event_mirror.rs"]
mod tests;
