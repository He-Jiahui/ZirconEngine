use std::slice;
use std::sync::Arc;

use serde::de::DeserializeOwned;
use zircon_runtime_interface::{
    ProfileControlRequest, ProfileControlResponse, ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimeApiV2,
    ZrRuntimeEventV1, ZrRuntimeFrameRequestV1, ZrRuntimeFrameV1, ZrRuntimeOperationHandle,
    ZrRuntimeOperationProgressV1, ZrRuntimeOperationResultV1, ZrRuntimeOperationSubmitRequestV1,
    ZrRuntimePluginEventDeliveryBatchV1, ZrRuntimePluginEventDeliveryV1,
    ZrRuntimePluginEventSubscribeRequestV1, ZrRuntimePluginEventSubscriptionHandle,
    ZrRuntimeSessionHandle, ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1, ZrStatus,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZIRCON_RUNTIME_API_VERSION_V2,
};

use super::{EditorRuntimeFrame, EditorRuntimeGateway, GatewayError, RuntimeCapabilities};

pub struct SessionGateway {
    _runtime_owner: Arc<dyn Send + Sync>,
    api: ZrRuntimeApiV2,
    session: ZrRuntimeSessionHandle,
    capabilities: RuntimeCapabilities,
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

    fn into_vec(self, operation: &'static str) -> Result<Vec<u8>, GatewayError> {
        let copied = self.bytes(operation).map(|bytes| bytes.to_vec());
        let released = self.release();
        match (copied, released) {
            (Ok(bytes), Ok(())) => Ok(bytes),
            (Err(error), Ok(())) => Err(error),
            (Ok(_), Err(error)) => Err(error),
            (Err(error), Err(release_error)) => Err(GatewayError::Protocol {
                message: format!("{error}; cleanup also failed: {release_error}"),
            }),
        }
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

impl SessionGateway {
    /// Creates a gateway over a validated runtime API table.
    ///
    /// # Safety
    ///
    /// `runtime_owner` must keep the library or linked provider that supplied every
    /// function pointer in `api` loaded until the gateway is dropped.
    pub unsafe fn new(
        runtime_owner: Arc<dyn Send + Sync>,
        api: ZrRuntimeApiV2,
        session: ZrRuntimeSessionHandle,
        capabilities: RuntimeCapabilities,
    ) -> Result<Self, GatewayError> {
        if !session.is_valid() {
            return Err(GatewayError::SessionLost);
        }
        if api.abi_version != ZIRCON_RUNTIME_API_VERSION_V2 {
            return Err(GatewayError::Protocol {
                message: format!(
                    "session gateway requires runtime API V2, received version {}",
                    api.abi_version
                ),
            });
        }
        Ok(Self {
            _runtime_owner: runtime_owner,
            api,
            session,
            capabilities,
        })
    }

    fn required<T: Copy>(entry: Option<T>, capability: &'static str) -> Result<T, GatewayError> {
        entry.ok_or(GatewayError::CapabilityMissing { capability })
    }

    fn decode_owned_output<T: DeserializeOwned>(
        output: GatewayOwnedOutput,
        operation: &'static str,
    ) -> Result<T, GatewayError> {
        let decoded = if let Err(error) = output.validate(operation) {
            Err(error)
        } else if output.is_empty() {
            Err(GatewayError::Protocol {
                message: format!("{operation} returned an empty payload"),
            })
        } else {
            match output.bytes(operation) {
                Ok(bytes) => {
                    serde_json::from_slice(bytes).map_err(|error| GatewayError::Protocol {
                        message: format!("{operation} returned invalid JSON: {error}"),
                    })
                }
                Err(error) => Err(error),
            }
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
        let Err(status_error) = ensure_status(status, operation) else {
            return Ok(output);
        };
        match output.release() {
            Ok(()) => Err(status_error),
            Err(release_error) => Err(GatewayError::Protocol {
                message: format!("{status_error}; cleanup also failed: {release_error}"),
            }),
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
    fn capabilities(&self) -> RuntimeCapabilities {
        self.capabilities.clone()
    }

    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        self.session
    }

    fn tick_frame(&self) -> Result<bool, GatewayError> {
        let tick = Self::required(self.api.tick_frame, "runtime.frame.tick")?;
        ensure_status(unsafe { tick(self.session) }, "tick runtime frame")?;
        Ok(true)
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
        let rgba = Self::validate_output_status(status, frame.rgba, "capture runtime frame")?
            .into_vec("capture runtime frame")?;
        ensure_output_abi(frame.abi_version, "runtime frame")?;
        Ok(EditorRuntimeFrame::new(
            frame.abi_version,
            frame.width,
            frame.height,
            frame.generation,
            rgba,
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
    ) -> Result<Vec<ZrRuntimePluginEventDeliveryV1>, GatewayError> {
        let drain = Self::required(self.api.drain_plugin_events, "runtime.plugin_event.drain")?;
        let mut output = ZrOwnedByteBuffer::empty();
        let status = unsafe { drain(self.session, subscription, &mut output) };
        let output = Self::validate_output_status(status, output, "drain runtime plugin events")?;
        output.validate("drain runtime plugin events")?;
        if output.is_empty() {
            output.release()?;
            return Ok(Vec::new());
        }
        let batch: ZrRuntimePluginEventDeliveryBatchV1 =
            Self::decode_owned_output(output, "drain runtime plugin events")?;
        ensure_output_abi(batch.abi_version, "runtime plugin event batch")?;
        Ok(batch.deliveries)
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
        Ok(result)
    }
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
