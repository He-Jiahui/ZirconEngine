use std::slice;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::de::DeserializeOwned;
use zircon_runtime_interface::{
    ProfileControlRequest, ProfileControlResponse, ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimeApiV3,
    ZrRuntimeEventV1, ZrRuntimeFrameDemandV1, ZrRuntimeFrameRequestV1, ZrRuntimeFrameV1,
    ZrRuntimeOperationHandle, ZrRuntimeOperationProgressV1, ZrRuntimeOperationResultV1,
    ZrRuntimeOperationSubmitRequestV1, ZrRuntimePluginEventDeliveryBatchV1,
    ZrRuntimePluginEventSubscribeRequestV1, ZrRuntimePluginEventSubscriptionHandle,
    ZrRuntimeSessionHandle, ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1, ZrStatus,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZIRCON_RUNTIME_API_VERSION_V3, ZR_RUNTIME_FRAME_DEMAND_AFTER_V1,
    ZR_RUNTIME_FRAME_DEMAND_IDLE_V1, ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1,
};

use super::{
    EditorRuntimeFrame, EditorRuntimeFrameDemand, EditorRuntimeFramePixels, EditorRuntimeGateway,
    EditorRuntimePluginEventPage, GatewayError, RuntimeCapabilities,
};

const MAX_EDITOR_RUNTIME_FRAME_DELAY: Duration = Duration::from_secs(60);

pub struct SessionGateway {
    _runtime_owner: Arc<dyn Send + Sync>,
    api: ZrRuntimeApiV3,
    session: ZrRuntimeSessionHandle,
    capabilities: Arc<RuntimeCapabilities>,
}

struct GatewayOwnedOutput {
    raw: Option<ZrOwnedByteBuffer>,
}

impl GatewayOwnedOutput {
    fn new(raw: ZrOwnedByteBuffer) -> Self {
        Self { raw: Some(raw) }
    }

    fn is_empty(&self) -> bool {
        self.raw.as_ref().is_none_or(|raw| raw.len == 0)
    }

    fn len(&self) -> usize {
        self.raw.as_ref().map_or(0, |raw| raw.len)
    }

    fn validate(&self, operation: &'static str) -> Result<(), GatewayError> {
        let Some(raw) = self.raw.as_ref() else {
            return Err(GatewayError::Protocol {
                message: format!("{operation} attempted to use released storage"),
            });
        };
        if raw.len > raw.capacity {
            return Err(GatewayError::Protocol {
                message: format!(
                    "{operation} returned malformed storage: len {} exceeds capacity {}",
                    raw.len, raw.capacity
                ),
            });
        }
        if raw.len > isize::MAX as usize || raw.capacity > isize::MAX as usize {
            return Err(GatewayError::Protocol {
                message: format!(
                    "{operation} returned malformed storage: len {} and capacity {} exceed the maximum Rust slice allocation",
                    raw.len, raw.capacity
                ),
            });
        }
        if raw.data.is_null() {
            return if raw.len == 0 && raw.capacity == 0 {
                Ok(())
            } else {
                Err(GatewayError::Protocol {
                    message: format!(
                        "{operation} returned malformed storage: null data with len {} and capacity {}",
                        raw.len, raw.capacity
                    ),
                })
            };
        }
        if !raw.data.is_null() && raw.free.is_none() {
            return Err(GatewayError::Protocol {
                message: format!("{operation} returned owned storage without a free callback"),
            });
        }
        Ok(())
    }

    fn bytes(&self, operation: &'static str) -> Result<&[u8], GatewayError> {
        self.validate(operation)?;
        let raw = self.raw.as_ref().ok_or_else(|| GatewayError::Protocol {
            message: format!("{operation} attempted to decode released storage"),
        })?;
        if raw.len == 0 {
            return Ok(&[]);
        }
        Ok(unsafe { slice::from_raw_parts(raw.data.cast_const(), raw.len) })
    }

    fn release(mut self) -> Result<(), GatewayError> {
        let Some(raw) = self.raw.take() else {
            return Ok(());
        };
        if let Some(free) = raw.free {
            ensure_status(unsafe { free(raw) }, "free runtime gateway output")?;
        }
        Ok(())
    }
}

impl Drop for GatewayOwnedOutput {
    fn drop(&mut self) {
        let Some(raw) = self.raw.take() else {
            return;
        };
        if let Some(free) = raw.free {
            let _ = unsafe { free(raw) };
        }
    }
}

struct SessionRuntimeFramePixels {
    _runtime_owner: Arc<dyn Send + Sync>,
    output: GatewayOwnedOutput,
    operation: &'static str,
}

impl EditorRuntimeFramePixels for SessionRuntimeFramePixels {
    fn rgba(&self) -> Result<&[u8], GatewayError> {
        self.output.bytes(self.operation)
    }

    fn release(self: Box<Self>) -> Result<(), GatewayError> {
        let Self { output, .. } = *self;
        output.release()
    }
}

impl SessionGateway {
    /// Creates a gateway over a validated runtime API table.
    ///
    /// # Safety
    ///
    /// `runtime_owner` must keep the library or linked provider that supplied every
    /// function pointer in `api` loaded until the gateway is dropped.
    pub unsafe fn new(
        runtime_owner: Arc<dyn Send + Sync>,
        api: ZrRuntimeApiV3,
        session: ZrRuntimeSessionHandle,
        capabilities: RuntimeCapabilities,
    ) -> Result<Self, GatewayError> {
        if !session.is_valid() {
            return Err(GatewayError::SessionLost);
        }
        if api.abi_version != ZIRCON_RUNTIME_API_VERSION_V3 {
            return Err(GatewayError::Protocol {
                message: format!(
                    "session gateway requires runtime API V3, received version {}",
                    api.abi_version
                ),
            });
        }
        Ok(Self {
            _runtime_owner: runtime_owner,
            api,
            session,
            capabilities: Arc::new(capabilities),
        })
    }

    fn required<T: Copy>(entry: Option<T>, capability: &'static str) -> Result<T, GatewayError> {
        entry.ok_or(GatewayError::CapabilityMissing { capability })
    }

    fn decode_owned_output<T: DeserializeOwned>(
        output: GatewayOwnedOutput,
        operation: &'static str,
    ) -> Result<T, GatewayError> {
        let decoded = match output.bytes(operation) {
            Ok([]) => Err(GatewayError::Protocol {
                message: format!("{operation} returned an empty payload"),
            }),
            Ok(bytes) => serde_json::from_slice(bytes).map_err(|error| GatewayError::Protocol {
                message: format!("{operation} returned invalid JSON: {error}"),
            }),
            Err(error) => Err(error),
        };
        let released = output.release();
        match (decoded, released) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(error), Ok(())) => Err(error),
            (Ok(_), Err(error)) => Err(error),
            (Err(error), Err(release_error)) => Err(GatewayError::Protocol {
                message: format!("{error}; cleanup also failed: {release_error}"),
            }),
        }
    }

    fn validate_output_status(
        status: ZrStatus,
        output: ZrOwnedByteBuffer,
        operation: &'static str,
    ) -> Result<GatewayOwnedOutput, GatewayError> {
        let output = GatewayOwnedOutput::new(output);
        let validation_error = output.validate(operation).err();
        let status_error = ensure_status(status, operation).err();
        if status_error.is_none() && validation_error.is_none() {
            return Ok(output);
        }

        let released = output.release();
        match (status_error, validation_error, released) {
            (Some(status_error), None, Ok(())) => Err(status_error),
            (status_error, Some(validation_error), Ok(())) => Err(GatewayError::Protocol {
                message: match status_error {
                    Some(status_error) => format!("{status_error}; {validation_error}"),
                    None => validation_error.to_string(),
                },
            }),
            (status_error, validation_error, Err(release_error)) => Err(GatewayError::Protocol {
                message: format!(
                    "{}; cleanup also failed: {release_error}",
                    status_error
                        .map(|error| error.to_string())
                        .or_else(|| validation_error.map(|error| error.to_string()))
                        .unwrap_or_else(|| operation.to_string())
                ),
            }),
            (None, None, Ok(())) => unreachable!("successful validated output returned above"),
        }
    }
}

impl std::fmt::Debug for SessionGateway {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SessionGateway")
            .field("session", &self.session)
            .field("capabilities", &self.capabilities)
            .finish_non_exhaustive()
    }
}

impl EditorRuntimeGateway for SessionGateway {
    fn capabilities(&self) -> Arc<RuntimeCapabilities> {
        self.capabilities.clone()
    }

    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        self.session
    }

    fn tick_frame(&self) -> Result<EditorRuntimeFrameDemand, GatewayError> {
        let tick = Self::required(self.api.tick_frame, "runtime.frame.tick")?;
        let mut demand = ZrRuntimeFrameDemandV1::idle();
        ensure_status(
            unsafe { tick(self.session, &mut demand) },
            "tick runtime frame",
        )?;
        frame_demand_from_runtime(demand)
    }

    fn handle_event(&self, event: ZrRuntimeEventV1) -> Result<(), GatewayError> {
        let handle_event = Self::required(self.api.handle_event, "runtime.event.handle")?;
        ensure_status(
            unsafe { handle_event(self.session, event) },
            "send runtime event",
        )
    }

    fn capture_frame(
        &self,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
    ) -> Result<EditorRuntimeFrame, GatewayError> {
        let capture = Self::required(self.api.capture_frame, "runtime.frame.capture")?;
        let mut frame = ZrRuntimeFrameV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
        let status = unsafe {
            capture(
                self.session,
                ZrRuntimeFrameRequestV1::new(ZIRCON_RUNTIME_ABI_VERSION_V1, viewport, size),
                &mut frame,
            )
        };
        let output = Self::validate_output_status(status, frame.rgba, "capture runtime frame")?;
        let validation = ensure_output_abi(frame.abi_version, "runtime frame").and_then(|()| {
            output
                .bytes("capture runtime frame")
                .and_then(|rgba| ensure_frame_rgba_shape(frame.width, frame.height, rgba))
        });
        if let Err(error) = validation {
            return release_output_after_error(output, error);
        }
        Ok(EditorRuntimeFrame::from_pixels(
            frame.abi_version,
            frame.width,
            frame.height,
            frame.generation,
            Box::new(SessionRuntimeFramePixels {
                _runtime_owner: self._runtime_owner.clone(),
                output,
                operation: "capture runtime frame",
            }),
        ))
    }

    fn profile_control(
        &self,
        request: &ProfileControlRequest,
    ) -> Result<Option<ProfileControlResponse>, GatewayError> {
        let Some(profile) = self.api.profile_control else {
            return Ok(None);
        };
        let request = serde_json::to_vec(request).map_err(|error| GatewayError::Protocol {
            message: format!("encode runtime profile request: {error}"),
        })?;
        let mut output = ZrOwnedByteBuffer::empty();
        let status = unsafe {
            profile(
                self.session,
                ZrByteSlice {
                    data: request.as_ptr(),
                    len: request.len(),
                },
                &mut output,
            )
        };
        let output = Self::validate_output_status(status, output, "control runtime profiling")?;
        Self::decode_owned_output(output, "control runtime profiling").map(Some)
    }

    fn subscribe_plugin_event(
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

    fn unsubscribe_plugin_event(
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

    fn drain_plugin_events(
        &self,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<EditorRuntimePluginEventPage, GatewayError> {
        let drain = Self::required(self.api.drain_plugin_events, "runtime.plugin_event.drain")?;
        let mut output = ZrOwnedByteBuffer::empty();
        let runtime_drain_started = Instant::now();
        let status = unsafe { drain(self.session, subscription, &mut output) };
        let runtime_drain_elapsed = runtime_drain_started.elapsed();
        let output = Self::validate_output_status(status, output, "drain runtime plugin events")?;
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
            Self::decode_owned_output(output, "drain runtime plugin events")?;
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

    fn submit_operation(
        &self,
        request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
        let submit = Self::required(self.api.submit_operation, "runtime.operation.submit")?;
        let request = serde_json::to_vec(&request).map_err(|error| GatewayError::Protocol {
            message: format!("encode runtime operation request: {error}"),
        })?;
        let mut handle = ZrRuntimeOperationHandle::invalid();
        ensure_status(
            unsafe {
                submit(
                    self.session,
                    ZrByteSlice {
                        data: request.as_ptr(),
                        len: request.len(),
                    },
                    &mut handle,
                )
            },
            "submit runtime operation",
        )?;
        if !handle.is_valid() {
            return Err(GatewayError::Protocol {
                message: "runtime returned an invalid operation handle".to_string(),
            });
        }
        Ok(handle)
    }

    fn poll_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationProgressV1, GatewayError> {
        let poll = Self::required(self.api.poll_operation, "runtime.operation.poll")?;
        let mut output = ZrOwnedByteBuffer::empty();
        let status = unsafe { poll(self.session, handle, &mut output) };
        let output = Self::validate_output_status(status, output, "poll runtime operation")?;
        let progress: ZrRuntimeOperationProgressV1 =
            Self::decode_owned_output(output, "poll runtime operation")?;
        ensure_output_abi(progress.abi_version, "runtime operation progress")?;
        ensure_operation_handle(progress.handle, handle, "runtime operation progress")?;
        Ok(progress)
    }

    fn harvest_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        let harvest = Self::required(self.api.harvest_operation, "runtime.operation.harvest")?;
        let mut output = ZrOwnedByteBuffer::empty();
        let status = unsafe { harvest(self.session, handle, &mut output) };
        let output = Self::validate_output_status(status, output, "harvest runtime operation")?;
        let result: ZrRuntimeOperationResultV1 =
            Self::decode_owned_output(output, "harvest runtime operation")?;
        ensure_output_abi(result.abi_version, "runtime operation result")?;
        ensure_operation_handle(result.handle, handle, "runtime operation result")?;
        Ok(result)
    }
}

fn ensure_frame_rgba_shape(width: u32, height: u32, rgba: &[u8]) -> Result<(), GatewayError> {
    if rgba.is_empty() {
        return Ok(());
    }
    let expected = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|pixels| pixels.checked_mul(4))
        .and_then(|bytes| usize::try_from(bytes).ok())
        .ok_or_else(|| GatewayError::Protocol {
            message: format!("runtime frame dimensions {width}x{height} overflow RGBA byte length"),
        })?;
    if rgba.len() == expected {
        return Ok(());
    }
    Err(GatewayError::Protocol {
        message: format!(
            "runtime frame {width}x{height} returned {} RGBA bytes; expected {expected}",
            rgba.len()
        ),
    })
}

fn release_output_after_error<T>(
    output: GatewayOwnedOutput,
    error: GatewayError,
) -> Result<T, GatewayError> {
    match output.release() {
        Ok(()) => Err(error),
        Err(release_error) => Err(GatewayError::Protocol {
            message: format!("{error}; cleanup also failed: {release_error}"),
        }),
    }
}

fn frame_demand_from_runtime(
    demand: ZrRuntimeFrameDemandV1,
) -> Result<EditorRuntimeFrameDemand, GatewayError> {
    if demand.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return Err(GatewayError::Protocol {
            message: format!(
                "runtime frame demand used unsupported ABI version {}",
                demand.abi_version
            ),
        });
    }
    if !demand.has_known_kind() {
        return Err(GatewayError::Protocol {
            message: format!("runtime frame demand used unknown kind {}", demand.kind),
        });
    }
    if demand.is_valid() {
        return match demand.kind {
            ZR_RUNTIME_FRAME_DEMAND_IDLE_V1 => Ok(EditorRuntimeFrameDemand::OnDemand),
            ZR_RUNTIME_FRAME_DEMAND_AFTER_V1 => Ok(EditorRuntimeFrameDemand::SleepUntil(
                Duration::from_nanos(demand.delay_nanoseconds).min(MAX_EDITOR_RUNTIME_FRAME_DELAY),
            )),
            ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1 => Ok(EditorRuntimeFrameDemand::Continuous),
            _ => unreachable!("known frame demand kind was validated above"),
        };
    }
    Err(GatewayError::Protocol {
        message: format!(
            "runtime frame demand kind {} returned invalid delay {}ns",
            demand.kind, demand.delay_nanoseconds
        ),
    })
}

fn ensure_operation_handle(
    response: ZrRuntimeOperationHandle,
    requested: ZrRuntimeOperationHandle,
    output_kind: &'static str,
) -> Result<(), GatewayError> {
    if response == requested {
        return Ok(());
    }
    Err(GatewayError::Protocol {
        message: format!(
            "{output_kind} handle {} did not match requested handle {}",
            response.raw(),
            requested.raw()
        ),
    })
}

fn ensure_output_abi(abi_version: u32, output_kind: &'static str) -> Result<(), GatewayError> {
    if abi_version == ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return Ok(());
    }
    Err(GatewayError::Protocol {
        message: format!("{output_kind} used unsupported ABI version {abi_version}"),
    })
}

fn ensure_status(status: ZrStatus, operation: &'static str) -> Result<(), GatewayError> {
    if status.is_ok() {
        return Ok(());
    }
    let diagnostics = unsafe { status.diagnostics.as_slice() };
    let diagnostics = String::from_utf8_lossy(diagnostics);
    Err(GatewayError::Runtime {
        message: format!(
            "{operation} failed with status {:?}: {diagnostics}",
            status.status_code()
        ),
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn owned_output_decode_validates_the_payload_once() {
        let source = include_str!("session.rs");
        let decode_body = source
            .split("fn decode_owned_output")
            .nth(1)
            .and_then(|body| body.split("fn validate_output_status").next())
            .expect("decode-owned-output body should remain available");
        let repeated_validation = ["output.val", "idate(operation)"].concat();
        assert!(!decode_body.contains(&repeated_validation));

        let drain_body = source
            .split("fn drain_plugin_events")
            .nth(1)
            .and_then(|body| body.split("fn submit_operation").next())
            .expect("session drain body should remain available");
        let explicit_validation = ["output.val", "idate(\"drain runtime plugin events\")"].concat();
        assert!(!drain_body.contains(&explicit_validation));
    }
}
