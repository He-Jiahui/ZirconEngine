use std::path::Path;
use std::slice;
#[cfg(feature = "target-editor-host")]
use std::sync::Arc;
use std::time::Duration;

use zircon_runtime::plugin::RuntimePluginRegistrationReport;
use zircon_runtime_interface::{
    ProfileControlRequest, ProfileControlResponse, ZrByteSlice, ZrOwnedByteBuffer,
    ZrRuntimeBindViewportSurfaceRequestV1, ZrRuntimeEventV1, ZrRuntimeFrameDemandV1,
    ZrRuntimeFrameRequestV1, ZrRuntimeFrameV1, ZrRuntimeHostRequestBatchV1, ZrRuntimeHostRequestV1,
    ZrRuntimePluginEventDeliveryBatchV1, ZrRuntimePluginEventDeliveryV1,
    ZrRuntimePluginEventSubscribeRequestV1, ZrRuntimePluginEventSubscriptionHandle,
    ZrRuntimeSessionConfigV2, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle,
    ZrRuntimeViewportSizeV1, ZrStatus, ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZIRCON_RUNTIME_ABI_VERSION_V2, ZR_RUNTIME_FRAME_DEMAND_AFTER_V1,
    ZR_RUNTIME_FRAME_DEMAND_IDLE_V1, ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1,
};

use super::{LoadedRuntime, RuntimeLibraryError, RuntimeWakeRegistration};

mod operation;

pub(crate) const MAX_HOST_RUNTIME_FRAME_DELAY: Duration = Duration::from_secs(60);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RuntimeFrameDemand {
    Idle,
    Immediate,
    After(Duration),
}

impl TryFrom<ZrRuntimeFrameDemandV1> for RuntimeFrameDemand {
    type Error = RuntimeLibraryError;

    fn try_from(demand: ZrRuntimeFrameDemandV1) -> Result<Self, Self::Error> {
        if demand.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
            return Err(RuntimeLibraryError::new(format!(
                "runtime frame demand used unsupported ABI version {}",
                demand.abi_version
            )));
        }
        match demand.kind {
            ZR_RUNTIME_FRAME_DEMAND_IDLE_V1 | ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1
                if demand.delay_nanoseconds != 0 =>
            {
                Err(RuntimeLibraryError::new(format!(
                    "runtime frame demand kind {} requires zero delay",
                    demand.kind
                )))
            }
            ZR_RUNTIME_FRAME_DEMAND_IDLE_V1 => Ok(Self::Idle),
            ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1 => Ok(Self::Immediate),
            ZR_RUNTIME_FRAME_DEMAND_AFTER_V1 => Ok(Self::After(
                Duration::from_nanos(demand.delay_nanoseconds).min(MAX_HOST_RUNTIME_FRAME_DELAY),
            )),
            kind => Err(RuntimeLibraryError::new(format!(
                "unsupported runtime frame demand kind {kind}"
            ))),
        }
    }
}

pub(crate) struct RuntimeSession {
    runtime: LoadedRuntime,
    handle: ZrRuntimeSessionHandle,
    wake_registration: Option<RuntimeWakeRegistration>,
}

impl RuntimeSession {
    #[cfg(feature = "target-editor-host")]
    pub(crate) fn editor_gateway(
        self: &Arc<Self>,
        capabilities: zircon_editor::core::gateway::RuntimeCapabilities,
    ) -> Result<
        Arc<zircon_editor::core::gateway::SessionGateway>,
        zircon_editor::core::gateway::GatewayError,
    > {
        let owner: Arc<dyn Send + Sync> = self.clone();
        let gateway = unsafe {
            zircon_editor::core::gateway::SessionGateway::new(
                owner,
                self.runtime.editor_gateway_api_table(),
                self.handle,
                capabilities,
            )?
        };
        Ok(Arc::new(gateway))
    }

    #[cfg(feature = "target-editor-host")]
    pub(crate) fn create_with_profile(
        runtime: LoadedRuntime,
        profile: &'static [u8],
    ) -> Result<Self, RuntimeLibraryError> {
        Self::create_with_profile_and_project(runtime, profile, None, None)
    }

    pub(in crate::entry) fn create_with_profile_and_project(
        runtime: LoadedRuntime,
        profile: &'static [u8],
        project_root: Option<&Path>,
        wake_registration: Option<RuntimeWakeRegistration>,
    ) -> Result<Self, RuntimeLibraryError> {
        let create_session = runtime.create_session();
        let mut handle = ZrRuntimeSessionHandle::invalid();
        let project_root = project_root
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_default();
        let project_manifest = if project_root.is_empty() {
            ZrByteSlice::empty()
        } else {
            ZrByteSlice {
                data: project_root.as_ptr(),
                len: project_root.len(),
            }
        };
        let status = unsafe {
            create_session(
                ZrRuntimeSessionConfigV2 {
                    abi_version: ZIRCON_RUNTIME_ABI_VERSION_V2,
                    profile: ZrByteSlice::from_static(profile),
                    project_manifest,
                    wake_sink: wake_registration
                        .as_ref()
                        .map(RuntimeWakeRegistration::sink)
                        .unwrap_or_else(zircon_runtime_interface::ZrRuntimeWakeSinkV1::disabled),
                },
                &mut handle,
            )
        };
        ensure_status(status, "create runtime session")?;
        if !handle.is_valid() {
            return Err(RuntimeLibraryError::new(
                "runtime returned an invalid session handle",
            ));
        }
        Ok(Self {
            runtime,
            handle,
            wake_registration,
        })
    }

    pub(crate) fn create_linked_with_profile_and_project(
        runtime: LoadedRuntime,
        profile: &[u8],
        project_root: Option<&Path>,
        registrations: Vec<RuntimePluginRegistrationReport>,
    ) -> Result<Self, RuntimeLibraryError> {
        let handle = zircon_runtime::dynamic_api::create_linked_runtime_session(
            profile,
            project_root,
            registrations,
        )
        .map_err(|error| RuntimeLibraryError::new(error.to_string()))?;
        if !handle.is_valid() {
            return Err(RuntimeLibraryError::new(
                "linked runtime returned an invalid session handle",
            ));
        }
        Ok(Self {
            runtime,
            handle,
            wake_registration: None,
        })
    }

    pub(crate) fn handle_event(&self, event: ZrRuntimeEventV1) -> Result<(), RuntimeLibraryError> {
        let handle_event = self.runtime.handle_event();
        let status = unsafe { handle_event(self.handle, event) };
        ensure_status(status, "send runtime event")
    }

    pub(crate) fn capture_frame(
        &self,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
    ) -> Result<RuntimeFrame, RuntimeLibraryError> {
        let capture_frame = self.runtime.capture_frame();
        let mut frame = ZrRuntimeFrameV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
        let status = unsafe {
            capture_frame(
                self.handle,
                ZrRuntimeFrameRequestV1::new(ZIRCON_RUNTIME_ABI_VERSION_V1, viewport, size),
                &mut frame,
            )
        };
        ensure_status(status, "capture runtime frame")?;
        Ok(RuntimeFrame { frame })
    }

    pub(crate) fn bind_viewport_surface(
        &self,
        request: ZrRuntimeBindViewportSurfaceRequestV1,
    ) -> Result<bool, RuntimeLibraryError> {
        let Some(bind) = self.runtime.bind_viewport_surface() else {
            return Ok(false);
        };
        ensure_status(
            unsafe { bind(self.handle, request) },
            "bind runtime viewport surface",
        )?;
        Ok(true)
    }

    pub(crate) fn unbind_viewport_surface(
        &self,
        viewport: ZrRuntimeViewportHandle,
    ) -> Result<bool, RuntimeLibraryError> {
        let Some(unbind) = self.runtime.unbind_viewport_surface() else {
            return Ok(false);
        };
        ensure_status(
            unsafe { unbind(self.handle, viewport) },
            "unbind runtime viewport surface",
        )?;
        Ok(true)
    }

    pub(crate) fn present_viewport(
        &self,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
    ) -> Result<bool, RuntimeLibraryError> {
        let Some(present) = self.runtime.present_viewport() else {
            return Ok(false);
        };
        ensure_status(
            unsafe {
                present(
                    self.handle,
                    ZrRuntimeFrameRequestV1::new(ZIRCON_RUNTIME_ABI_VERSION_V1, viewport, size),
                )
            },
            "present runtime viewport",
        )?;
        Ok(true)
    }

    pub(crate) fn tick_frame(&self) -> Result<RuntimeFrameDemand, RuntimeLibraryError> {
        let tick_frame = self.runtime.tick_frame();
        let mut demand = ZrRuntimeFrameDemandV1::idle();
        ensure_status(
            unsafe { tick_frame(self.handle, &mut demand) },
            "tick runtime frame",
        )?;
        RuntimeFrameDemand::try_from(demand)
    }

    pub(crate) fn wake_host(&self) {
        if let Some(registration) = &self.wake_registration {
            registration.wake();
        }
    }

    pub(crate) fn drain_host_requests(
        &self,
    ) -> Result<Vec<ZrRuntimeHostRequestV1>, RuntimeLibraryError> {
        let Some(drain_host_requests) = self.runtime.drain_host_requests() else {
            return Ok(Vec::new());
        };
        let mut output = ZrOwnedByteBuffer::empty();
        let status = unsafe { drain_host_requests(self.handle, &mut output) };
        ensure_status(status, "drain runtime host requests")?;
        if output.is_empty() {
            return Ok(Vec::new());
        }

        let bytes = unsafe { slice::from_raw_parts(output.data.cast_const(), output.len) };
        let batch = serde_json::from_slice::<ZrRuntimeHostRequestBatchV1>(bytes)
            .map_err(|error| RuntimeLibraryError::new(error.to_string()));
        if let Some(free) = output.free {
            ensure_status(unsafe { free(output) }, "free runtime host requests")?;
        }
        let batch = batch?;
        if batch.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
            return Err(RuntimeLibraryError::new(
                "runtime host request batch used an unsupported ABI version",
            ));
        }
        Ok(batch.requests)
    }

    pub(crate) fn profile_control(
        &self,
        request: &ProfileControlRequest,
    ) -> Result<Option<ProfileControlResponse>, RuntimeLibraryError> {
        let Some(profile_control) = self.runtime.profile_control() else {
            return Ok(None);
        };
        let request = serde_json::to_vec(request).map_err(|error| {
            RuntimeLibraryError::new(format!("encode runtime profile request: {error}"))
        })?;
        let mut output = ZrOwnedByteBuffer::empty();
        let status = unsafe {
            profile_control(
                self.handle,
                ZrByteSlice {
                    data: request.as_ptr(),
                    len: request.len(),
                },
                &mut output,
            )
        };
        if let Err(error) = ensure_status(status, "control runtime profiling") {
            if let Some(free) = output.free {
                let _ = unsafe { free(output) };
            }
            return Err(error);
        }
        if output.is_empty() {
            if let Some(free) = output.free {
                ensure_status(
                    unsafe { free(output) },
                    "free empty runtime profile response",
                )?;
            }
            return Ok(None);
        }
        let response = unsafe { slice::from_raw_parts(output.data.cast_const(), output.len) };
        let response =
            serde_json::from_slice::<ProfileControlResponse>(response).map_err(|error| {
                RuntimeLibraryError::new(format!("decode runtime profile response: {error}"))
            });
        if let Some(free) = output.free {
            ensure_status(unsafe { free(output) }, "free runtime profile response")?;
        }
        response.map(Some)
    }

    pub(crate) fn supports_viewport_surface_present(&self) -> bool {
        self.runtime.supports_viewport_surface_present()
    }

    pub(crate) fn subscribe_plugin_event(
        &self,
        event_id: &str,
        payload_schema: &str,
    ) -> Result<Option<ZrRuntimePluginEventSubscriptionHandle>, RuntimeLibraryError> {
        let subscribe = self.runtime.subscribe_plugin_event();
        let request = serde_json::to_vec(&ZrRuntimePluginEventSubscribeRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            event_id,
            payload_schema,
        ))
        .map_err(|error| RuntimeLibraryError::new(error.to_string()))?;
        let mut subscription = ZrRuntimePluginEventSubscriptionHandle::invalid();
        ensure_status(
            unsafe {
                subscribe(
                    self.handle,
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
            return Err(RuntimeLibraryError::new(
                "runtime returned an invalid plugin event subscription",
            ));
        }
        Ok(Some(subscription))
    }

    pub(crate) fn unsubscribe_plugin_event(
        &self,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<bool, RuntimeLibraryError> {
        let unsubscribe = self.runtime.unsubscribe_plugin_event();
        ensure_status(
            unsafe { unsubscribe(self.handle, subscription) },
            "unsubscribe runtime plugin event",
        )?;
        Ok(true)
    }

    pub(crate) fn drain_plugin_events(
        &self,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<Vec<ZrRuntimePluginEventDeliveryV1>, RuntimeLibraryError> {
        let drain = self.runtime.drain_plugin_events();
        let mut output = ZrOwnedByteBuffer::empty();
        ensure_status(
            unsafe { drain(self.handle, subscription, &mut output) },
            "drain runtime plugin events",
        )?;
        if output.is_empty() {
            return Ok(Vec::new());
        }
        let bytes = unsafe { slice::from_raw_parts(output.data.cast_const(), output.len) };
        let decoded = serde_json::from_slice::<ZrRuntimePluginEventDeliveryBatchV1>(bytes)
            .map_err(|error| RuntimeLibraryError::new(error.to_string()));
        if let Some(free) = output.free {
            ensure_status(unsafe { free(output) }, "free runtime plugin events")?;
        }
        let batch = decoded?;
        if batch.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
            return Err(RuntimeLibraryError::new(
                "runtime plugin event batch used an unsupported ABI version",
            ));
        }
        Ok(batch.deliveries)
    }
}

impl Drop for RuntimeSession {
    fn drop(&mut self) {
        let _ = self.unbind_viewport_surface(ZrRuntimeViewportHandle::new(1));
        let destroy_session = self.runtime.destroy_session();
        let destroy_status = unsafe { destroy_session(self.handle) };
        if destroy_status.is_ok() {
            if let Some(wake_registration) = &mut self.wake_registration {
                wake_registration.unregister();
            }
        } else if let Some(wake_registration) = self.wake_registration.take() {
            // A failed destroy cannot prove the runtime released its copied callback.
            std::mem::forget(wake_registration);
        }
    }
}

pub(crate) struct RuntimeFrame {
    frame: ZrRuntimeFrameV1,
}

impl RuntimeFrame {
    pub(crate) fn width(&self) -> u32 {
        self.frame.width
    }

    pub(crate) fn height(&self) -> u32 {
        self.frame.height
    }

    pub(crate) fn rgba(&self) -> &[u8] {
        let rgba = self.frame.rgba;
        if rgba.data.is_null() || rgba.len == 0 {
            &[]
        } else {
            unsafe { slice::from_raw_parts(rgba.data.cast_const(), rgba.len) }
        }
    }
}

impl Drop for RuntimeFrame {
    fn drop(&mut self) {
        if let Some(free) = self.frame.rgba.free {
            let buffer = self.frame.rgba;
            let _ = unsafe { free(buffer) };
            self.frame.rgba = ZrOwnedByteBuffer::empty();
        }
    }
}

fn ensure_status(status: ZrStatus, operation: &'static str) -> Result<(), RuntimeLibraryError> {
    if status.is_ok() {
        return Ok(());
    }
    let diagnostics = unsafe { status.diagnostics.as_slice() };
    let diagnostics = String::from_utf8_lossy(diagnostics);
    let code = match status.status_code() {
        ZrStatusCode::Ok => "ok",
        ZrStatusCode::Error => "error",
        ZrStatusCode::UnsupportedVersion => "unsupported-version",
        ZrStatusCode::InvalidArgument => "invalid-argument",
        ZrStatusCode::NotFound => "not-found",
        ZrStatusCode::CapabilityDenied => "capability-denied",
        ZrStatusCode::BridgeNotEnabled => "bridge-not-enabled",
        ZrStatusCode::Panic => "panic",
    };
    Err(RuntimeLibraryError::new(format!(
        "failed to {operation}: {code}: {diagnostics}"
    )))
}
