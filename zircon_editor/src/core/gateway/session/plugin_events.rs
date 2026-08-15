use std::time::{Duration, Instant};

use zircon_runtime_interface::{
    ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimePluginEventDeliveryBatchV1,
    ZrRuntimePluginEventSubscribeRequestV1, ZrRuntimePluginEventSubscriptionHandle,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1,
};

use super::super::{EditorRuntimePluginEventPage, GatewayError};
use super::gateway::SessionGateway;
use super::output::{decode_owned_output, release_output_after_error, validate_output_status};
use super::protocol::{ensure_output_abi, ensure_status};

impl SessionGateway {
    pub(super) fn subscribe_plugin_event(
        &self,
        event_id: &str,
        payload_schema: &str,
    ) -> Result<Option<ZrRuntimePluginEventSubscriptionHandle>, GatewayError> {
        let subscribe = Self::required(
            self.api.subscribe_plugin_event,
            "runtime.plugin_event.subscribe",
        )?;
        let request = serde_json::to_vec(&ZrRuntimePluginEventSubscribeRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            event_id,
            payload_schema,
        ))
        .map_err(|error| GatewayError::Protocol {
            message: format!("encode runtime plugin event subscription: {error}"),
        })?;
        let mut subscription = ZrRuntimePluginEventSubscriptionHandle::invalid();
        ensure_status(
            unsafe {
                subscribe(
                    self.session,
                    ZrByteSlice {
                        data: request.as_ptr(),
                        len: request.len(),
                    },
                    &mut subscription,
                )
            },
            "subscribe runtime plugin event",
        )?;
        if !subscription.is_valid() {
            return Err(GatewayError::Protocol {
                message: "runtime returned an invalid plugin event subscription".to_string(),
            });
        }
        Ok(Some(subscription))
    }

    pub(super) fn unsubscribe_plugin_event(
        &self,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<bool, GatewayError> {
        let unsubscribe = Self::required(
            self.api.unsubscribe_plugin_event,
            "runtime.plugin_event.unsubscribe",
        )?;
        ensure_status(
            unsafe { unsubscribe(self.session, subscription) },
            "unsubscribe runtime plugin event",
        )?;
        Ok(true)
    }

    pub(super) fn drain_plugin_events(
        &self,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<EditorRuntimePluginEventPage, GatewayError> {
        let drain = Self::required(self.api.drain_plugin_events, "runtime.plugin_event.drain")?;
        let mut output = ZrOwnedByteBuffer::empty();
        let runtime_drain_started = Instant::now();
        let status = unsafe { drain(self.session, subscription, &mut output) };
        let runtime_drain_elapsed = runtime_drain_started.elapsed();
        let output = validate_output_status(status, output, "drain runtime plugin events")?;
        if output.is_empty() {
            output.release()?;
            return Ok(EditorRuntimePluginEventPage::new(
                Vec::new(),
                0,
                runtime_drain_elapsed,
                Duration::ZERO,
            ));
        }
        let encoded_bytes = output.len();
        if encoded_bytes > ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1 {
            return release_output_after_error(
                output,
                GatewayError::Protocol {
                    message: format!(
                        "runtime plugin event page returned {encoded_bytes} encoded bytes; maximum is {ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1}"
                    ),
                },
            );
        }
        let decode_started = Instant::now();
        let batch: ZrRuntimePluginEventDeliveryBatchV1 =
            decode_owned_output(output, "drain runtime plugin events")?;
        let decode_elapsed = decode_started.elapsed();
        ensure_output_abi(batch.abi_version, "runtime plugin event batch")?;
        if batch.deliveries.len() > ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1 {
            return Err(GatewayError::Protocol {
                message: format!(
                    "runtime plugin event batch returned {} deliveries; maximum is {ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1}",
                    batch.deliveries.len()
                ),
            });
        }
        if let Some(delivery) = batch
            .deliveries
            .iter()
            .find(|delivery| delivery.subscription != subscription)
        {
            return Err(GatewayError::Protocol {
                message: format!(
                    "runtime plugin event delivery subscription {} did not match requested subscription {}",
                    delivery.subscription.raw(),
                    subscription.raw()
                ),
            });
        }
        Ok(EditorRuntimePluginEventPage::new(
            batch.deliveries,
            encoded_bytes,
            runtime_drain_elapsed,
            decode_elapsed,
        )
        .with_runtime_backlog(
            batch.remaining_deliveries as usize,
            batch.oldest_pending_age_millis,
        ))
    }
}
