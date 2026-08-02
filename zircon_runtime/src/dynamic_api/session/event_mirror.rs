use std::collections::HashMap;
use std::io::Write;
use std::ptr;

use zircon_runtime_interface::{
    ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimePluginEventDeliveryBatchV1,
    ZrRuntimePluginEventSubscribeRequestV1, ZrRuntimePluginEventSubscriptionHandle, ZrStatus,
    ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1,
};

#[cfg(test)]
use zircon_runtime_interface::ZrRuntimePluginEventDeliveryV1;

use crate::scene::RuntimeEventMirrorSubscription;

use super::RuntimeDynamicSession;

const RUNTIME_PLUGIN_EVENT_BUFFER_OWNER_TOKEN: u64 = 0x5a52_5045_564e_5401;
pub(in crate::dynamic_api) const RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES: usize =
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1;
pub(in crate::dynamic_api) const RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES: usize =
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1;

pub(super) struct RuntimePluginEventSubscriptionState {
    subscription: RuntimeEventMirrorSubscription,
    sequence: u64,
}

impl RuntimeDynamicSession {
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

    pub(super) fn drain_plugin_events(
        &mut self,
        play_session_id: u64,
        handle: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<ZrOwnedByteBuffer, String> {
        let state = self
            .plugin_event_subscriptions
            .get_mut(&handle.raw())
            .ok_or_else(|| "runtime plugin event subscription not found".to_string())?;
        let delivery_limit = usize::try_from(u64::MAX - state.sequence)
            .unwrap_or(usize::MAX)
            .min(RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES);
        let page = self
            .level
            .with_world(|world| {
                world.drain_runtime_event_mirror_payloads(&mut state.subscription, delivery_limit)
            })
            .map_err(|error| error.to_string())?;
        if page.payloads.is_empty() {
            if page.remaining_deliveries > 0 && delivery_limit == 0 {
                return Err("runtime plugin event sequence overflowed".to_string());
            }
            return Ok(ZrOwnedByteBuffer::empty());
        }

        let descriptor = state.subscription.descriptor();
        let mut bytes = Vec::with_capacity(
            page.payloads
                .iter()
                .map(|payload| payload.json_bytes().len())
                .sum::<usize>(),
        );
        bytes.extend_from_slice(br#"{"abiVersion":"#);
        write_json_integer(&mut bytes, u64::from(ZIRCON_RUNTIME_ABI_VERSION_V1))?;
        bytes.extend_from_slice(br#","deliveries":["#);
        for (index, payload) in page.payloads.into_iter().enumerate() {
            if index != 0 {
                bytes.push(b',');
            }
            state.sequence = state
                .sequence
                .checked_add(1)
                .expect("runtime plugin event page sequence was preflighted");
            bytes.extend_from_slice(br#"{"playSessionId":"#);
            write_json_integer(&mut bytes, play_session_id)?;
            bytes.extend_from_slice(br#","subscription":"#);
            write_json_integer(&mut bytes, handle.raw())?;
            bytes.extend_from_slice(br#","eventId":"#);
            serde_json::to_writer(&mut bytes, &descriptor.event_id)
                .map_err(|error| error.to_string())?;
            bytes.extend_from_slice(br#","payloadSchema":"#);
            serde_json::to_writer(&mut bytes, &descriptor.payload_schema)
                .map_err(|error| error.to_string())?;
            bytes.extend_from_slice(br#","sequence":"#);
            write_json_integer(&mut bytes, state.sequence)?;
            bytes.extend_from_slice(br#","payload":"#);
            bytes.extend_from_slice(payload.json_bytes());
            bytes.push(b'}');
        }
        bytes.extend_from_slice(br#"],"remainingDeliveries":"#);
        write_json_integer(&mut bytes, u64::from(page.remaining_deliveries))?;
        bytes.extend_from_slice(br#","oldestPendingAgeMillis":"#);
        write_json_integer(&mut bytes, page.oldest_pending_age_millis)?;
        bytes.extend_from_slice(b"}");
        owned_plugin_event_buffer_with_wire_ceiling(bytes)
    }
}

pub(super) fn empty_plugin_event_subscriptions() -> HashMap<u64, RuntimePluginEventSubscriptionState>
{
    HashMap::new()
}

pub(super) fn encode_plugin_event_batch(
    batch: &ZrRuntimePluginEventDeliveryBatchV1,
) -> Result<ZrOwnedByteBuffer, String> {
    if batch.deliveries.is_empty() {
        return Ok(ZrOwnedByteBuffer::empty());
    }
    let bytes = serde_json::to_vec(batch).map_err(|error| error.to_string())?;
    owned_plugin_event_buffer_with_wire_ceiling(bytes)
}

fn write_json_integer(bytes: &mut Vec<u8>, value: u64) -> Result<(), String> {
    write!(bytes, "{value}").map_err(|error| error.to_string())
}

fn owned_plugin_event_buffer_with_wire_ceiling(
    bytes: Vec<u8>,
) -> Result<ZrOwnedByteBuffer, String> {
    // The scene mirror caps each page at a fixed event / encoded-payload budget and
    // registration caps both descriptor strings at 128 bytes. Even worst-case JSON
    // escaping therefore stays below this wire ceiling; keep the runtime check so a
    // future contract change cannot silently make the ABI page unbounded.
    if bytes.len() > RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES {
        return Err(format!(
            "runtime plugin event page encoded {} bytes, maximum is {}",
            bytes.len(),
            RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES
        ));
    }
    Ok(owned_plugin_event_buffer(bytes))
}

pub(super) unsafe fn write_plugin_event_batch(
    output: *mut ZrOwnedByteBuffer,
    buffer: ZrOwnedByteBuffer,
) -> ZrStatus {
    if output.is_null() {
        return ZrStatus::new(
            ZrStatusCode::InvalidArgument,
            ZrByteSlice::from_static(b"missing runtime plugin event output"),
        );
    }
    unsafe { ptr::write(output, buffer) };
    ZrStatus::ok()
}

fn owned_plugin_event_buffer(mut bytes: Vec<u8>) -> ZrOwnedByteBuffer {
    if bytes.is_empty() {
        return ZrOwnedByteBuffer::empty();
    }
    let buffer = ZrOwnedByteBuffer {
        data: bytes.as_mut_ptr(),
        len: bytes.len(),
        capacity: bytes.capacity(),
        owner_token: RUNTIME_PLUGIN_EVENT_BUFFER_OWNER_TOKEN,
        free: Some(free_runtime_plugin_event_bytes),
    };
    std::mem::forget(bytes);
    buffer
}

unsafe extern "C" fn free_runtime_plugin_event_bytes(buffer: ZrOwnedByteBuffer) -> ZrStatus {
    if buffer.is_empty() {
        return ZrStatus::ok();
    }
    if buffer.owner_token != RUNTIME_PLUGIN_EVENT_BUFFER_OWNER_TOKEN || buffer.data.is_null() {
        return ZrStatus::new(
            ZrStatusCode::InvalidArgument,
            ZrByteSlice::from_static(b"invalid runtime plugin event buffer"),
        );
    }
    if buffer.len > buffer.capacity {
        return ZrStatus::new(
            ZrStatusCode::InvalidArgument,
            ZrByteSlice::from_static(b"invalid runtime plugin event buffer"),
        );
    }
    let _ = unsafe { Vec::from_raw_parts(buffer.data, buffer.len, buffer.capacity) };
    ZrStatus::ok()
}

#[cfg(test)]
mod tests {
    use serde::Serialize;
    use serde_json::json;

    use super::super::profile::RuntimeDynamicSessionProfile;
    use super::*;
    use crate::scene::{
        RuntimeEventMirrorRegistration, RUNTIME_EVENT_MIRROR_PAGE_MAX_EVENTS,
        RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES,
    };

    const SEQUENCE_WINDOW_EVENT_ID: &str = "dynamic_api.plugin_event.sequence_window";
    const SEQUENCE_WINDOW_PAYLOAD_SCHEMA: &str =
        "zircon.dynamic_api.plugin_event.sequence_window.v1";

    #[derive(Clone, Debug, Serialize)]
    struct SequenceWindowEvent {
        value: u8,
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
    fn empty_plugin_event_page_uses_an_empty_owned_buffer() {
        let batch = ZrRuntimePluginEventDeliveryBatchV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);

        let buffer = encode_plugin_event_batch(&batch).unwrap();

        assert!(buffer.is_empty());
        assert!(buffer.free.is_none());
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
            .drain_plugin_events(7, subscription)
            .expect("two deliveries fit within remaining sequence headroom");
        let bytes = unsafe { std::slice::from_raw_parts(buffer.data, buffer.len) };
        let batch = serde_json::from_slice::<ZrRuntimePluginEventDeliveryBatchV1>(bytes)
            .expect("sequence-window event page");
        assert_eq!(batch.deliveries.len(), 2);
        assert_eq!(batch.deliveries[0].sequence, u64::MAX - 1);
        assert_eq!(batch.deliveries[1].sequence, u64::MAX);
        assert_eq!(batch.remaining_deliveries, 0);
        assert_eq!(
            unsafe { free_runtime_plugin_event_bytes(buffer) }.status_code(),
            ZrStatusCode::Ok
        );

        let idle = session
            .drain_plugin_events(7, subscription)
            .expect("an idle page at the maximum sequence remains representable");
        assert!(idle.is_empty());
        assert!(idle.free.is_none());
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
        assert!(buffer.len <= RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES);
        assert_eq!(
            unsafe { free_runtime_plugin_event_bytes(buffer) }.status_code(),
            ZrStatusCode::Ok
        );

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

        assert!(buffer.len <= RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES);
        assert_eq!(
            unsafe { free_runtime_plugin_event_bytes(buffer) }.status_code(),
            ZrStatusCode::Ok
        );
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
        assert!(buffer.len <= RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES);
        assert_eq!(
            unsafe { free_runtime_plugin_event_bytes(buffer) }.status_code(),
            ZrStatusCode::Ok
        );
    }
}
