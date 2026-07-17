use std::path::Path;
use std::slice;
#[cfg(feature = "target-editor-host")]
use std::sync::Arc;

use zircon_runtime::plugin::RuntimePluginRegistrationReport;
use zircon_runtime_interface::{
    ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimeBindViewportSurfaceRequestV1, ZrRuntimeEventV1,
    ZrRuntimeFrameRequestV1, ZrRuntimeFrameV1, ZrRuntimeHostRequestBatchV1, ZrRuntimeHostRequestV1,
    ZrRuntimePluginEventDeliveryBatchV1, ZrRuntimePluginEventDeliveryV1,
    ZrRuntimePluginEventSubscribeRequestV1, ZrRuntimePluginEventSubscriptionHandle,
    ZrRuntimeSessionConfigV1, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle,
    ZrRuntimeViewportSizeV1, ZrStatus, ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use super::{LoadedRuntime, RuntimeLibraryError};

mod operation;

pub(crate) struct RuntimeSession {
    runtime: LoadedRuntime,
    handle: ZrRuntimeSessionHandle,
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
        Self::create_with_profile_and_project(runtime, profile, None)
    }

    pub(crate) fn create_with_profile_and_project(
        runtime: LoadedRuntime,
        profile: &'static [u8],
        project_root: Option<&Path>,
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
                ZrRuntimeSessionConfigV1 {
                    abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
                    profile: ZrByteSlice::from_static(profile),
                    project_manifest,
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
        Ok(Self { runtime, handle })
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
        Ok(Self { runtime, handle })
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

    pub(crate) fn tick_frame(&self) -> Result<bool, RuntimeLibraryError> {
        let Some(tick_frame) = self.runtime.tick_frame() else {
            return Ok(false);
        };
        ensure_status(unsafe { tick_frame(self.handle) }, "tick runtime frame")?;
        Ok(true)
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
        let _ = unsafe { destroy_session(self.handle) };
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
