use std::collections::HashMap;
use std::ptr;

use zircon_runtime_interface::{
    ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimePluginEventDeliveryBatchV1,
    ZrRuntimePluginEventDeliveryV1, ZrRuntimePluginEventSubscribeRequestV1,
    ZrRuntimePluginEventSubscriptionHandle, ZrStatus, ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use crate::scene::RuntimeEventMirrorSubscription;

use super::RuntimeDynamicSession;

const RUNTIME_PLUGIN_EVENT_BUFFER_OWNER_TOKEN: u64 = 0x5a52_5045_564e_5401;

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
    ) -> Result<ZrRuntimePluginEventDeliveryBatchV1, String> {
        let state = self
            .plugin_event_subscriptions
            .get_mut(&handle.raw())
            .ok_or_else(|| "runtime plugin event subscription not found".to_string())?;
        let descriptor = state.subscription.descriptor().clone();
        let payloads = self
            .level
            .with_world(|world| world.drain_runtime_event_mirror(&mut state.subscription))
            .map_err(|error| error.to_string())?;
        let mut deliveries = Vec::with_capacity(payloads.len());
        for payload in payloads {
            state.sequence = state
                .sequence
                .checked_add(1)
                .ok_or_else(|| "runtime plugin event sequence overflowed".to_string())?;
            deliveries.push(ZrRuntimePluginEventDeliveryV1::new(
                play_session_id,
                handle,
                descriptor.event_id.clone(),
                descriptor.payload_schema.clone(),
                state.sequence,
                payload,
            ));
        }
        Ok(ZrRuntimePluginEventDeliveryBatchV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            deliveries,
        ))
    }
}

pub(super) fn empty_plugin_event_subscriptions() -> HashMap<u64, RuntimePluginEventSubscriptionState>
{
    HashMap::new()
}

pub(super) fn encode_plugin_event_batch(
    batch: &ZrRuntimePluginEventDeliveryBatchV1,
) -> Result<ZrOwnedByteBuffer, serde_json::Error> {
    serde_json::to_vec(batch).map(owned_plugin_event_buffer)
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
