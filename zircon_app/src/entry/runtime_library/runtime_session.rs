use std::marker::PhantomData;
use std::path::Path;
use std::slice;
use std::sync::Arc;

use zircon_runtime::diagnostic_log::write_log;
use zircon_runtime_host::foreign_output::{
    profile_control_response_item_count, RuntimeOwnedOutputReleaser,
};
use zircon_runtime_host::viewport_surface::ViewportSurfaceBindings;
use zircon_runtime_interface::project::RelPath;
use zircon_runtime_interface::runtime_build_set::ZrRuntimeModuleCompositionReceiptV1;
use zircon_runtime_interface::{
    validate_runtime_frame_rgba_shape, ProfileControlRequest, ProfileControlResponse, ZrByteSlice,
    ZrOwnedResultV2, ZrRuntimeBindViewportSurfaceRequestV1, ZrRuntimeEventV1,
    ZrRuntimeFrameRequestV1, ZrRuntimeFrameV2, ZrRuntimeHostRequestBatchV1, ZrRuntimeHostRequestV1,
    ZrRuntimePluginEventDeliveryBatchV1, ZrRuntimePluginEventDeliveryV1,
    ZrRuntimePluginEventSubscribeRequestV1, ZrRuntimePluginEventSubscriptionHandle,
    ZrRuntimeSessionConfigV3, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle,
    ZrRuntimeViewportSizeV1, ZrStatus, ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZIRCON_RUNTIME_ABI_VERSION_V2, ZIRCON_RUNTIME_ABI_VERSION_V3,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1,
    ZR_RUNTIME_PLUGIN_EVENT_SUBSCRIBE_REQUEST_LIMIT_V1, ZR_RUNTIME_PROFILE_REQUEST_LIMIT_V1,
    ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1,
};

use super::{
    LoadedRuntime, RuntimeLibraryError, RuntimeSessionTeardownFailureState, RuntimeWakeRegistration,
};

mod foreign_output;
mod frame_demand;
pub(super) mod module_composition_receipt;
mod operation;
mod owned_buffer;
mod request_encoding;
mod surface_bindings;

pub(crate) use frame_demand::{RuntimeFrameDemand, MAX_HOST_RUNTIME_FRAME_DELAY};

use foreign_output::{
    ForeignOutputKind, ForeignOutputState, HOST_REQUEST_OUTPUT_BUDGET, PLUGIN_EVENT_OUTPUT_BUDGET,
    PROFILE_RESPONSE_OUTPUT_BUDGET,
};
use owned_buffer::{
    release_owned_result, release_owned_result_after_error,
    validate_owned_result_releasing_on_error,
};
use request_encoding::encode_runtime_request;

pub(crate) struct RuntimeSession {
    runtime: Option<LoadedRuntime>,
    handle: ZrRuntimeSessionHandle,
    module_composition_receipt: Option<ZrRuntimeModuleCompositionReceiptV1>,
    wake_registration: Option<RuntimeWakeRegistration>,
    viewport_surface_bindings: Arc<ViewportSurfaceBindings>,
    teardown_failure_state: RuntimeSessionTeardownFailureState,
    foreign_output: Arc<ForeignOutputState>,
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
        self.foreign_output
            .ensure_session_available("create runtime editor gateway")
            .map_err(zircon_editor::core::gateway::GatewayError::from)?;
        let owner: Arc<dyn Send + Sync> = self.clone();
        let gateway = unsafe {
            zircon_editor::core::gateway::SessionGateway::new(
                owner,
                self.runtime().editor_gateway_api_table(),
                self.handle,
                capabilities,
                self.foreign_output.clone(),
            )?
        }
        .with_module_composition_receipt(self.module_composition_receipt().clone())?
        .with_viewport_surface_bindings(self.viewport_surface_bindings.clone());
        Ok(Arc::new(gateway))
    }

    #[cfg(feature = "target-editor-host")]
    pub(crate) fn create_with_profile(
        runtime: LoadedRuntime,
        profile: &'static [u8],
    ) -> Result<Self, RuntimeLibraryError> {
        Self::create_with_profile_and_project(runtime, profile, None, None, None, None)
    }

    pub(in crate::entry) fn create_with_profile_and_project(
        runtime: LoadedRuntime,
        profile: &'static [u8],
        project_root: Option<&Path>,
        play_scene: Option<&RelPath>,
        play_report_pipe: Option<&str>,
        wake_registration: Option<RuntimeWakeRegistration>,
    ) -> Result<Self, RuntimeLibraryError> {
        let create_session = runtime.create_session();
        let mut handle = ZrRuntimeSessionHandle::invalid();
        let project_root = project_root_for_abi(project_root)?;
        let project_root = project_root
            .filter(|root| !root.is_empty())
            .map(|root| ZrByteSlice {
                data: root.as_ptr(),
                len: root.len(),
            })
            .unwrap_or_else(ZrByteSlice::empty);
        let play_scene = play_scene
            .map(|scene| ZrByteSlice {
                data: scene.as_str().as_ptr(),
                len: scene.as_str().len(),
            })
            .unwrap_or_else(ZrByteSlice::empty);
        let play_report_pipe = play_report_pipe
            .map(|pipe| ZrByteSlice {
                data: pipe.as_ptr(),
                len: pipe.len(),
            })
            .unwrap_or_else(ZrByteSlice::empty);
        let status = unsafe {
            create_session(
                ZrRuntimeSessionConfigV3 {
                    abi_version: ZIRCON_RUNTIME_ABI_VERSION_V3,
                    profile: ZrByteSlice::from_static(profile),
                    project_root,
                    play_scene,
                    play_report_pipe,
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
        let mut session = Self {
            runtime: Some(runtime),
            handle,
            module_composition_receipt: None,
            wake_registration,
            viewport_surface_bindings: Arc::new(ViewportSurfaceBindings::default()),
            teardown_failure_state: RuntimeSessionTeardownFailureState::default(),
            foreign_output: Arc::new(ForeignOutputState::default()),
        };
        let receipt = module_composition_receipt::query(&session, profile)?;
        session.module_composition_receipt = Some(receipt);
        Ok(session)
    }

    pub(in crate::entry) fn module_composition_receipt(
        &self,
    ) -> &ZrRuntimeModuleCompositionReceiptV1 {
        self.module_composition_receipt
            .as_ref()
            .expect("successfully constructed runtime sessions retain a composition receipt")
    }

    pub(in crate::entry) fn teardown_failure_state(&self) -> RuntimeSessionTeardownFailureState {
        self.teardown_failure_state.clone()
    }

    /// Destroys this session while retaining the loaded library and handle on failure.
    ///
    /// App-owned Play leases call this only after Editor has detached every gateway generation.
    /// Ordinary owners may continue to rely on `Drop`, whose failure remains process-fatal.
    pub(in crate::entry) fn try_destroy(&mut self) -> Result<(), RuntimeLibraryError> {
        if !self.handle.is_valid() {
            return Ok(());
        }
        if let Some(diagnostic) = self.foreign_output.diagnostic_line() {
            write_log("runtime_foreign_output", diagnostic);
        }
        self.release_bound_viewport_surfaces_for_teardown();
        let destroy_session = self.runtime().destroy_session();
        let destroy_status = unsafe { destroy_session(self.handle) };
        ensure_status(destroy_status, "destroy runtime session")?;
        if let Some(wake_registration) = &mut self.wake_registration {
            wake_registration.unregister();
        }
        self.handle = ZrRuntimeSessionHandle::invalid();
        self.runtime.take();
        Ok(())
    }

    fn runtime(&self) -> &LoadedRuntime {
        self.runtime
            .as_ref()
            .expect("runtime library must remain loaded until session destruction")
    }

    fn output_releaser(&self) -> RuntimeOwnedOutputReleaser {
        RuntimeOwnedOutputReleaser::new(self.handle, self.runtime().release_allocation())
    }

    pub(crate) fn handle_event(&self, event: ZrRuntimeEventV1) -> Result<(), RuntimeLibraryError> {
        self.foreign_output
            .ensure_session_available("send runtime event")?;
        let handle_event = self.runtime().handle_event();
        let status = unsafe { handle_event(self.handle, event) };
        ensure_status(status, "send runtime event")
    }

    pub(crate) fn capture_frame(
        &self,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
    ) -> Result<RuntimeFrame<'_>, RuntimeLibraryError> {
        self.foreign_output
            .ensure_available(ForeignOutputKind::SessionProtocol)?;
        let capture_frame = self.runtime().capture_frame();
        let releaser = self.output_releaser();
        let mut frame = ZrRuntimeFrameV2::empty(ZIRCON_RUNTIME_ABI_VERSION_V2);
        let status = unsafe {
            capture_frame(
                self.handle,
                ZrRuntimeFrameRequestV1::new(ZIRCON_RUNTIME_ABI_VERSION_V1, viewport, size),
                &mut frame,
            )
        };
        frame.rgba = self.foreign_output.ensure_call_succeeded(
            status,
            frame.rgba,
            releaser,
            ForeignOutputKind::SessionProtocol,
            "capture runtime frame",
            "free runtime frame output after failed capture",
        )?;
        frame.rgba = match validate_owned_result_releasing_on_error(
            frame.rgba,
            releaser,
            "capture runtime frame",
            "free runtime frame output after invalid capture",
        ) {
            Ok(output) => output,
            Err(error) => {
                return self
                    .foreign_output
                    .reject_protocol(ForeignOutputKind::SessionProtocol, error)
                    .map_err(Into::into);
            }
        };
        if let Err(error) = validate_runtime_frame_releasing_on_error(&mut frame, releaser) {
            return self
                .foreign_output
                .reject_protocol(ForeignOutputKind::SessionProtocol, error)
                .map_err(Into::into);
        }
        Ok(RuntimeFrame {
            frame,
            teardown_failure_state: self.teardown_failure_state.clone(),
            foreign_output: self.foreign_output.clone(),
            releaser,
            _session: PhantomData,
        })
    }

    pub(crate) fn bind_viewport_surface(
        &self,
        request: ZrRuntimeBindViewportSurfaceRequestV1,
    ) -> Result<bool, RuntimeLibraryError> {
        self.foreign_output
            .ensure_session_available("bind runtime viewport surface")?;
        let Some(bind) = self.runtime().bind_viewport_surface() else {
            return Ok(false);
        };
        let operation = self.begin_viewport_surface_binding(request.viewport)?;
        let result = ensure_status(
            unsafe { bind(self.handle, request) },
            "bind runtime viewport surface",
        );
        self.finish_viewport_surface_binding(operation, result.is_ok());
        result.map(|()| true)
    }

    pub(crate) fn unbind_viewport_surface(
        &self,
        viewport: ZrRuntimeViewportHandle,
    ) -> Result<bool, RuntimeLibraryError> {
        self.foreign_output
            .ensure_session_available("unbind runtime viewport surface")?;
        self.unbind_viewport_surface_for_teardown(viewport)
    }

    fn unbind_viewport_surface_for_teardown(
        &self,
        viewport: ZrRuntimeViewportHandle,
    ) -> Result<bool, RuntimeLibraryError> {
        let Some(operation) = self.begin_viewport_surface_release(viewport)? else {
            return Ok(false);
        };
        let Some(unbind) = self.runtime().unbind_viewport_surface() else {
            self.finish_viewport_surface_release(operation, false);
            return Ok(false);
        };
        let result = ensure_status(
            unsafe { unbind(self.handle, viewport) },
            "unbind runtime viewport surface",
        );
        self.finish_viewport_surface_release(operation, result.is_ok());
        result.map(|()| true)
    }

    fn release_bound_viewport_surfaces_for_teardown(&self) {
        for viewport in self.bound_viewport_surfaces() {
            if let Err(error) = self.unbind_viewport_surface_for_teardown(viewport) {
                self.teardown_failure_state.record(error);
            }
        }
    }

    pub(crate) fn present_viewport(
        &self,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
    ) -> Result<bool, RuntimeLibraryError> {
        self.foreign_output
            .ensure_session_available("present runtime viewport")?;
        let Some(present) = self.runtime().present_viewport() else {
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
        self.foreign_output
            .ensure_session_available("tick runtime frame")?;
        let tick_frame = self.runtime().tick_frame();
        let mut demand = ZrRuntimeFrameDemandV1::idle();
        ensure_status(
            unsafe { tick_frame(self.handle, &mut demand) },
            "tick runtime frame",
        )?;
        match RuntimeFrameDemand::try_from(demand) {
            Ok(demand) => Ok(demand),
            Err(error) => self
                .foreign_output
                .reject_protocol(ForeignOutputKind::SessionProtocol, error)
                .map_err(Into::into),
        }
    }

    pub(crate) fn wake_host(&self) {
        if let Some(registration) = &self.wake_registration {
            registration.wake();
        }
    }

    pub(crate) fn drain_host_requests(
        &self,
    ) -> Result<Vec<ZrRuntimeHostRequestV1>, RuntimeLibraryError> {
        self.foreign_output
            .ensure_available(ForeignOutputKind::HostRequests)?;
        let Some(drain_host_requests) = self.runtime().drain_host_requests() else {
            return Ok(Vec::new());
        };
        let releaser = self.output_releaser();
        let mut output = ZrOwnedResultV2::empty();
        let status = unsafe { drain_host_requests(self.handle, &mut output) };
        output = self.foreign_output.ensure_call_succeeded(
            status,
            output,
            releaser,
            ForeignOutputKind::HostRequests,
            "drain runtime host requests",
            "free runtime host requests",
        )?;
        let batch = self
            .foreign_output
            .decode_json::<ZrRuntimeHostRequestBatchV1, _>(
                output,
                releaser,
                ForeignOutputKind::HostRequests,
                HOST_REQUEST_OUTPUT_BUDGET,
                "decode runtime host requests",
                "free runtime host requests",
                |batch| {
                    if batch.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
                        return Err(RuntimeLibraryError::new(
                            "runtime host request batch used an unsupported ABI version",
                        ));
                    }
                    Ok(batch.requests.len())
                },
            )?;
        Ok(batch.map(|batch| batch.requests).unwrap_or_default())
    }

    pub(crate) fn profile_control(
        &self,
        request: &ProfileControlRequest,
    ) -> Result<Option<ProfileControlResponse>, RuntimeLibraryError> {
        self.foreign_output
            .ensure_available(ForeignOutputKind::ProfileResponse)?;
        let Some(profile_control) = self.runtime().profile_control() else {
            return Ok(None);
        };
        let request = encode_runtime_request(
            request,
            ZR_RUNTIME_PROFILE_REQUEST_LIMIT_V1,
            1,
            "encode runtime profile request",
        )?;
        let releaser = self.output_releaser();
        let mut output = ZrOwnedResultV2::empty();
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
        output = self.foreign_output.ensure_call_succeeded(
            status,
            output,
            releaser,
            ForeignOutputKind::ProfileResponse,
            "control runtime profiling",
            "free runtime profile response",
        )?;
        Ok(self
            .foreign_output
            .decode_json::<ProfileControlResponse, &'static str>(
                output,
                releaser,
                ForeignOutputKind::ProfileResponse,
                PROFILE_RESPONSE_OUTPUT_BUDGET,
                "decode runtime profile response",
                "free runtime profile response",
                |response| Ok(profile_control_response_item_count(response)),
            )?)
    }

    pub(crate) fn supports_viewport_surface_present(&self) -> bool {
        self.runtime().supports_viewport_surface_present()
    }

    pub(crate) fn subscribe_plugin_event(
        &self,
        event_id: &str,
        payload_schema: &str,
    ) -> Result<Option<ZrRuntimePluginEventSubscriptionHandle>, RuntimeLibraryError> {
        self.foreign_output
            .ensure_session_available("subscribe runtime plugin event")?;
        let subscribe = self.runtime().subscribe_plugin_event();
        let request = encode_runtime_request(
            &ZrRuntimePluginEventSubscribeRequestV1::new(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                event_id,
                payload_schema,
            ),
            ZR_RUNTIME_PLUGIN_EVENT_SUBSCRIBE_REQUEST_LIMIT_V1,
            3,
            "encode runtime plugin event subscription request",
        )?;
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
            return self
                .foreign_output
                .reject_protocol(
                    ForeignOutputKind::PluginEvents,
                    RuntimeLibraryError::protocol_violation(
                        "runtime returned an invalid plugin event subscription",
                    ),
                )
                .map_err(Into::into);
        }
        Ok(Some(subscription))
    }

    pub(crate) fn unsubscribe_plugin_event(
        &self,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<bool, RuntimeLibraryError> {
        self.foreign_output
            .ensure_session_available("unsubscribe runtime plugin event")?;
        let unsubscribe = self.runtime().unsubscribe_plugin_event();
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
        self.foreign_output
            .ensure_available(ForeignOutputKind::PluginEvents)?;
        let drain = self.runtime().drain_plugin_events();
        let releaser = self.output_releaser();
        let mut output = ZrOwnedResultV2::empty();
        let status = unsafe { drain(self.handle, subscription, &mut output) };
        output = self.foreign_output.ensure_call_succeeded(
            status,
            output,
            releaser,
            ForeignOutputKind::PluginEvents,
            "drain runtime plugin events",
            "free runtime plugin events",
        )?;
        let batch = self
            .foreign_output
            .decode_json::<ZrRuntimePluginEventDeliveryBatchV1, RuntimeLibraryError>(
                output,
                releaser,
                ForeignOutputKind::PluginEvents,
                PLUGIN_EVENT_OUTPUT_BUDGET,
                "decode runtime plugin events",
                "free runtime plugin events",
                |batch| {
                    validate_plugin_event_batch(&batch, subscription)?;
                    Ok::<usize, RuntimeLibraryError>(batch.deliveries.len())
                },
            )?;
        Ok(batch.map(|batch| batch.deliveries).unwrap_or_default())
    }
}

impl Drop for RuntimeSession {
    fn drop(&mut self) {
        if let Err(error) = self.try_destroy() {
            let detail = error.to_string();
            self.teardown_failure_state.record(error);
            abort_after_runtime_session_teardown_failure(&detail);
        }
    }
}

/// A failed dynamic-session destroy cannot prove that copied callbacks and DLL workers stopped.
///
/// Returning into normal Rust drop would unload the library or release host callback storage while
/// that code may still execute. Process termination is the only safe terminal path; successful
/// teardown continues through ordinary drop and releases the dynamic library normally.
fn abort_after_runtime_session_teardown_failure(detail: &str) -> ! {
    eprintln!(
        "fatal runtime session teardown failure: {detail}; aborting before dynamic library unload"
    );
    std::process::abort();
}

pub(crate) struct RuntimeFrame<'session> {
    frame: ZrRuntimeFrameV2,
    teardown_failure_state: RuntimeSessionTeardownFailureState,
    foreign_output: Arc<ForeignOutputState>,
    releaser: RuntimeOwnedOutputReleaser,
    _session: PhantomData<&'session RuntimeSession>,
}

impl RuntimeFrame<'_> {
    pub(crate) fn width(&self) -> u32 {
        self.frame.width
    }

    pub(crate) fn height(&self) -> u32 {
        self.frame.height
    }

    pub(crate) fn rgba(&self) -> &[u8] {
        let rgba = &self.frame.rgba;
        if rgba.data.is_null() || rgba.len == 0 {
            &[]
        } else {
            let len = usize::try_from(rgba.len).expect("validated runtime frame length");
            unsafe { slice::from_raw_parts(rgba.data, len) }
        }
    }
}

impl Drop for RuntimeFrame<'_> {
    fn drop(&mut self) {
        let output = std::mem::replace(&mut self.frame.rgba, ZrOwnedResultV2::empty());
        if let Err(error) = release_owned_result(output, self.releaser, "free runtime frame buffer")
        {
            if let Err(protocol_error) = self
                .foreign_output
                .reject_protocol::<()>(ForeignOutputKind::SessionProtocol, error)
            {
                self.teardown_failure_state.record(protocol_error.into());
            }
        }
    }
}

fn ensure_status(status: ZrStatus, operation: &'static str) -> Result<(), RuntimeLibraryError> {
    if status.is_ok() {
        return Ok(());
    }
    let diagnostics = unsafe {
        status
            .diagnostics
            .checked_slice(ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1)
    }
    .map_err(|error| {
        RuntimeLibraryError::new(format!(
            "failed to {operation}: runtime returned invalid status diagnostics: {error:?}"
        ))
    })?;
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
        ZrStatusCode::LimitExceeded => "limit-exceeded",
    };
    Err(RuntimeLibraryError::new(format!(
        "failed to {operation}: {code}: {diagnostics}"
    )))
}

fn project_root_for_abi(project_root: Option<&Path>) -> Result<Option<&str>, RuntimeLibraryError> {
    project_root
        .map(|path| {
            path.to_str().ok_or_else(|| {
                RuntimeLibraryError::new(format!(
                    "runtime project root cannot cross the UTF-8 ABI boundary: {}",
                    path.display()
                ))
            })
        })
        .transpose()
}

fn validate_runtime_frame(frame: &ZrRuntimeFrameV2) -> Result<(), RuntimeLibraryError> {
    if frame.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V2 {
        return Err(RuntimeLibraryError::new(format!(
            "runtime frame used unsupported ABI version {}",
            frame.abi_version
        )));
    }
    validate_runtime_frame_rgba_shape(frame.width, frame.height, frame.rgba.len)
        .map_err(|error| RuntimeLibraryError::new(error.to_string()))
}

fn validate_runtime_frame_releasing_on_error(
    frame: &mut ZrRuntimeFrameV2,
    releaser: RuntimeOwnedOutputReleaser,
) -> Result<(), RuntimeLibraryError> {
    match validate_runtime_frame(frame) {
        Ok(()) => Ok(()),
        Err(error) => release_owned_result_after_error(
            std::mem::replace(&mut frame.rgba, ZrOwnedResultV2::empty()),
            releaser,
            error,
            "free runtime frame output after invalid capture",
        ),
    }
}

fn validate_plugin_event_batch(
    batch: &ZrRuntimePluginEventDeliveryBatchV1,
    subscription: ZrRuntimePluginEventSubscriptionHandle,
) -> Result<(), RuntimeLibraryError> {
    if batch.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return Err(RuntimeLibraryError::new(
            "runtime plugin event batch used an unsupported ABI version",
        ));
    }
    if batch.deliveries.len() > ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1 {
        return Err(RuntimeLibraryError::new(format!(
            "runtime plugin event batch returned {} deliveries; maximum is {}",
            batch.deliveries.len(),
            ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1
        )));
    }
    if let Some(delivery) = batch
        .deliveries
        .iter()
        .find(|delivery| delivery.subscription != subscription)
    {
        return Err(RuntimeLibraryError::new(format!(
            "runtime plugin event delivery subscription {} did not match requested subscription {}",
            delivery.subscription.raw(),
            subscription.raw()
        )));
    }
    Ok(())
}

fn validate_plugin_event_encoded_len(encoded_len: usize) -> Result<(), RuntimeLibraryError> {
    if encoded_len <= ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1 {
        return Ok(());
    }
    Err(RuntimeLibraryError::new(format!(
        "runtime plugin event page returned {encoded_len} encoded bytes; maximum is {ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1}"
    )))
}

#[cfg(test)]
mod tests;
