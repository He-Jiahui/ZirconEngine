use std::marker::PhantomData;
use std::path::Path;
use std::slice;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use zircon_runtime::plugin::RuntimePluginRegistrationReport;
use zircon_runtime_interface::project::RelPath;
use zircon_runtime_interface::{
    ProfileControlRequest, ProfileControlResponse, ZrByteSlice, ZrOwnedByteBuffer,
    ZrRuntimeBindViewportSurfaceRequestV1, ZrRuntimeEventV1, ZrRuntimeFrameDemandV1,
    ZrRuntimeFrameRequestV1, ZrRuntimeFrameV1, ZrRuntimeHostRequestBatchV1, ZrRuntimeHostRequestV1,
    ZrRuntimePluginEventDeliveryBatchV1, ZrRuntimePluginEventDeliveryV1,
    ZrRuntimePluginEventSubscribeRequestV1, ZrRuntimePluginEventSubscriptionHandle,
    ZrRuntimeSessionConfigV3, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle,
    ZrRuntimeViewportSizeV1, ZrStatus, ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZIRCON_RUNTIME_ABI_VERSION_V3, ZR_RUNTIME_FRAME_DEMAND_AFTER_V1,
    ZR_RUNTIME_FRAME_DEMAND_IDLE_V1, ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1,
};

use super::{
    LoadedRuntime, RuntimeLibraryError, RuntimeSessionTeardownFailureState, RuntimeWakeRegistration,
};

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
    runtime: Option<LoadedRuntime>,
    handle: ZrRuntimeSessionHandle,
    wake_registration: Option<RuntimeWakeRegistration>,
    viewport_surface_bound: Arc<AtomicBool>,
    teardown_failure_state: RuntimeSessionTeardownFailureState,
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
                self.runtime().editor_gateway_api_table(),
                self.handle,
                capabilities,
            )?
        }
        .with_viewport_surface_lifecycle_state(self.viewport_surface_lifecycle_state());
        Ok(Arc::new(gateway))
    }

    #[cfg(feature = "target-editor-host")]
    fn viewport_surface_lifecycle_state(&self) -> Arc<AtomicBool> {
        self.viewport_surface_bound.clone()
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
        Ok(Self {
            runtime: Some(runtime),
            handle,
            wake_registration,
            viewport_surface_bound: Arc::new(AtomicBool::new(false)),
            teardown_failure_state: RuntimeSessionTeardownFailureState::default(),
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
            runtime: Some(runtime),
            handle,
            wake_registration: None,
            viewport_surface_bound: Arc::new(AtomicBool::new(false)),
            teardown_failure_state: RuntimeSessionTeardownFailureState::default(),
        })
    }

    pub(in crate::entry) fn teardown_failure_state(&self) -> RuntimeSessionTeardownFailureState {
        self.teardown_failure_state.clone()
    }

    fn runtime(&self) -> &LoadedRuntime {
        self.runtime
            .as_ref()
            .expect("runtime library must remain loaded until session destruction")
    }

    pub(crate) fn handle_event(&self, event: ZrRuntimeEventV1) -> Result<(), RuntimeLibraryError> {
        let handle_event = self.runtime().handle_event();
        let status = unsafe { handle_event(self.handle, event) };
        ensure_status(status, "send runtime event")
    }

    pub(crate) fn capture_frame(
        &self,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
    ) -> Result<RuntimeFrame<'_>, RuntimeLibraryError> {
        let capture_frame = self.runtime().capture_frame();
        let mut frame = ZrRuntimeFrameV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
        let status = unsafe {
            capture_frame(
                self.handle,
                ZrRuntimeFrameRequestV1::new(ZIRCON_RUNTIME_ABI_VERSION_V1, viewport, size),
                &mut frame,
            )
        };
        ensure_status_releasing_output_on_error(
            status,
            "capture runtime frame",
            frame.rgba,
            "free runtime frame output after failed capture",
        )?;
        validate_owned_buffer_releasing_on_error(
            frame.rgba,
            "capture runtime frame",
            "free runtime frame output after invalid capture",
        )?;
        validate_runtime_frame_releasing_on_error(&frame)?;
        Ok(RuntimeFrame {
            frame,
            teardown_failure_state: self.teardown_failure_state.clone(),
            _session: PhantomData,
        })
    }

    pub(crate) fn bind_viewport_surface(
        &self,
        request: ZrRuntimeBindViewportSurfaceRequestV1,
    ) -> Result<bool, RuntimeLibraryError> {
        let Some(bind) = self.runtime().bind_viewport_surface() else {
            return Ok(false);
        };
        ensure_status(
            unsafe { bind(self.handle, request) },
            "bind runtime viewport surface",
        )?;
        self.viewport_surface_bound.store(true, Ordering::Release);
        Ok(true)
    }

    pub(crate) fn unbind_viewport_surface(
        &self,
        viewport: ZrRuntimeViewportHandle,
    ) -> Result<bool, RuntimeLibraryError> {
        if !self.viewport_surface_bound.load(Ordering::Acquire) {
            return Ok(false);
        }
        let Some(unbind) = self.runtime().unbind_viewport_surface() else {
            self.viewport_surface_bound.store(false, Ordering::Release);
            return Ok(false);
        };
        ensure_status(
            unsafe { unbind(self.handle, viewport) },
            "unbind runtime viewport surface",
        )?;
        self.viewport_surface_bound.store(false, Ordering::Release);
        Ok(true)
    }

    pub(crate) fn present_viewport(
        &self,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
    ) -> Result<bool, RuntimeLibraryError> {
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
        let tick_frame = self.runtime().tick_frame();
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
        let Some(drain_host_requests) = self.runtime().drain_host_requests() else {
            return Ok(Vec::new());
        };
        let mut output = ZrOwnedByteBuffer::empty();
        let status = unsafe { drain_host_requests(self.handle, &mut output) };
        ensure_status_releasing_output_on_error(
            status,
            "drain runtime host requests",
            output,
            "free runtime host requests",
        )?;
        validate_owned_buffer_releasing_on_error(
            output,
            "decode runtime host requests",
            "free runtime host requests",
        )?;
        if output.len == 0 {
            release_owned_buffer(output, "free runtime host requests")?;
            return Ok(Vec::new());
        }

        let bytes = unsafe { slice::from_raw_parts(output.data.cast_const(), output.len) };
        let batch = serde_json::from_slice::<ZrRuntimeHostRequestBatchV1>(bytes)
            .map_err(|error| RuntimeLibraryError::new(error.to_string()))
            .and_then(|batch| {
                if batch.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
                    return Err(RuntimeLibraryError::new(
                        "runtime host request batch used an unsupported ABI version",
                    ));
                }
                Ok(batch)
            });
        let batch = release_owned_buffer_after_result(output, batch, "free runtime host requests")?;
        Ok(batch.requests)
    }

    pub(crate) fn profile_control(
        &self,
        request: &ProfileControlRequest,
    ) -> Result<Option<ProfileControlResponse>, RuntimeLibraryError> {
        let Some(profile_control) = self.runtime().profile_control() else {
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
        ensure_status_releasing_output_on_error(
            status,
            "control runtime profiling",
            output,
            "free runtime profile response",
        )?;
        validate_owned_buffer_releasing_on_error(
            output,
            "decode runtime profile response",
            "free runtime profile response",
        )?;
        if output.len == 0 {
            release_owned_buffer(output, "free runtime profile response")?;
            return Ok(None);
        }
        let response = unsafe { slice::from_raw_parts(output.data.cast_const(), output.len) };
        let response =
            serde_json::from_slice::<ProfileControlResponse>(response).map_err(|error| {
                RuntimeLibraryError::new(format!("decode runtime profile response: {error}"))
            });
        release_owned_buffer_after_result(output, response, "free runtime profile response")
            .map(Some)
    }

    pub(crate) fn supports_viewport_surface_present(&self) -> bool {
        self.runtime().supports_viewport_surface_present()
    }

    pub(crate) fn subscribe_plugin_event(
        &self,
        event_id: &str,
        payload_schema: &str,
    ) -> Result<Option<ZrRuntimePluginEventSubscriptionHandle>, RuntimeLibraryError> {
        let subscribe = self.runtime().subscribe_plugin_event();
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
        let drain = self.runtime().drain_plugin_events();
        let mut output = ZrOwnedByteBuffer::empty();
        let status = unsafe { drain(self.handle, subscription, &mut output) };
        ensure_status_releasing_output_on_error(
            status,
            "drain runtime plugin events",
            output,
            "free runtime plugin events",
        )?;
        validate_owned_buffer_releasing_on_error(
            output,
            "decode runtime plugin events",
            "free runtime plugin events",
        )?;
        if output.len == 0 {
            release_owned_buffer(output, "free runtime plugin events")?;
            return Ok(Vec::new());
        }
        if let Err(error) = validate_plugin_event_encoded_len(output.len) {
            return release_owned_buffer_after_error(output, error, "free runtime plugin events");
        }
        let bytes = unsafe { slice::from_raw_parts(output.data.cast_const(), output.len) };
        let decoded = serde_json::from_slice::<ZrRuntimePluginEventDeliveryBatchV1>(bytes)
            .map_err(|error| RuntimeLibraryError::new(error.to_string()))
            .and_then(|batch| {
                validate_plugin_event_batch(&batch, subscription)?;
                Ok(batch)
            });
        let batch =
            release_owned_buffer_after_result(output, decoded, "free runtime plugin events")?;
        Ok(batch.deliveries)
    }
}

impl Drop for RuntimeSession {
    fn drop(&mut self) {
        if let Err(error) = self.unbind_viewport_surface(ZrRuntimeViewportHandle::new(1)) {
            self.teardown_failure_state.record(error);
        }
        let destroy_session = self.runtime().destroy_session();
        let destroy_status = unsafe { destroy_session(self.handle) };
        match ensure_status(destroy_status, "destroy runtime session") {
            Ok(()) => {
                if let Some(wake_registration) = &mut self.wake_registration {
                    wake_registration.unregister();
                }
            }
            Err(error) => {
                let detail = error.to_string();
                self.teardown_failure_state.record(error);
                abort_after_runtime_session_teardown_failure(&detail);
            }
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
    frame: ZrRuntimeFrameV1,
    teardown_failure_state: RuntimeSessionTeardownFailureState,
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
        let rgba = self.frame.rgba;
        if rgba.data.is_null() || rgba.len == 0 {
            &[]
        } else {
            unsafe { slice::from_raw_parts(rgba.data.cast_const(), rgba.len) }
        }
    }
}

impl Drop for RuntimeFrame<'_> {
    fn drop(&mut self) {
        let buffer = self.frame.rgba;
        self.frame.rgba = ZrOwnedByteBuffer::empty();
        if let Err(error) = release_owned_buffer(buffer, "free runtime frame buffer") {
            self.teardown_failure_state.record(error);
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

fn release_owned_buffer(
    output: ZrOwnedByteBuffer,
    operation: &'static str,
) -> Result<(), RuntimeLibraryError> {
    let Some(free) = output.free else {
        return Ok(());
    };
    ensure_status(unsafe { free(output) }, operation)
}

fn release_owned_buffer_after_error<T>(
    output: ZrOwnedByteBuffer,
    error: RuntimeLibraryError,
    release_operation: &'static str,
) -> Result<T, RuntimeLibraryError> {
    match release_owned_buffer(output, release_operation) {
        Ok(()) => Err(error),
        Err(release_error) => Err(RuntimeLibraryError::new(format!(
            "{error}; cleanup also failed: {release_error}"
        ))),
    }
}

fn release_owned_buffer_after_result<T>(
    output: ZrOwnedByteBuffer,
    result: Result<T, RuntimeLibraryError>,
    release_operation: &'static str,
) -> Result<T, RuntimeLibraryError> {
    match (result, release_owned_buffer(output, release_operation)) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(release_error)) => Err(release_error),
        (Err(error), Err(release_error)) => Err(RuntimeLibraryError::new(format!(
            "{error}; cleanup also failed: {release_error}"
        ))),
    }
}

fn validate_owned_buffer(
    output: &ZrOwnedByteBuffer,
    operation: &'static str,
) -> Result<(), RuntimeLibraryError> {
    if output.len > output.capacity {
        return Err(RuntimeLibraryError::new(format!(
            "{operation} returned malformed storage: len {} exceeds capacity {}",
            output.len, output.capacity
        )));
    }
    if output.len > isize::MAX as usize || output.capacity > isize::MAX as usize {
        return Err(RuntimeLibraryError::new(format!(
            "{operation} returned malformed storage: len {} and capacity {} exceed the maximum Rust slice allocation",
            output.len, output.capacity
        )));
    }
    if output.data.is_null() {
        return if output.len == 0 && output.capacity == 0 {
            Ok(())
        } else {
            Err(RuntimeLibraryError::new(format!(
                "{operation} returned malformed storage: null data with len {} and capacity {}",
                output.len, output.capacity
            )))
        };
    }
    if output.free.is_none() {
        return Err(RuntimeLibraryError::new(format!(
            "{operation} returned owned storage without a free callback"
        )));
    }
    Ok(())
}

fn validate_owned_buffer_releasing_on_error(
    output: ZrOwnedByteBuffer,
    operation: &'static str,
    release_operation: &'static str,
) -> Result<(), RuntimeLibraryError> {
    match validate_owned_buffer(&output, operation) {
        Ok(()) => Ok(()),
        Err(error) => release_owned_buffer_after_error(output, error, release_operation),
    }
}

fn validate_runtime_frame(frame: &ZrRuntimeFrameV1) -> Result<(), RuntimeLibraryError> {
    if frame.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return Err(RuntimeLibraryError::new(format!(
            "runtime frame used unsupported ABI version {}",
            frame.abi_version
        )));
    }
    if frame.width == 0 || frame.height == 0 {
        return Err(RuntimeLibraryError::new(format!(
            "runtime frame returned invalid dimensions {}x{}",
            frame.width, frame.height
        )));
    }
    let expected_len = (frame.width as usize)
        .checked_mul(frame.height as usize)
        .and_then(|pixel_count| pixel_count.checked_mul(4))
        .ok_or_else(|| RuntimeLibraryError::new("runtime frame pixel length overflowed usize"))?;
    if frame.rgba.len != expected_len {
        return Err(RuntimeLibraryError::new(format!(
            "runtime frame returned {} RGBA bytes for {}x{} pixels; expected {}",
            frame.rgba.len, frame.width, frame.height, expected_len
        )));
    }
    Ok(())
}

fn validate_runtime_frame_releasing_on_error(
    frame: &ZrRuntimeFrameV1,
) -> Result<(), RuntimeLibraryError> {
    match validate_runtime_frame(frame) {
        Ok(()) => Ok(()),
        Err(error) => release_owned_buffer_after_error(
            frame.rgba,
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

fn ensure_status_releasing_output_on_error(
    status: ZrStatus,
    operation: &'static str,
    output: ZrOwnedByteBuffer,
    release_operation: &'static str,
) -> Result<(), RuntimeLibraryError> {
    let Err(error) = ensure_status(status, operation) else {
        return Ok(());
    };
    release_owned_buffer_after_error(output, error, release_operation)
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;
    use std::sync::atomic::{AtomicBool, Ordering};

    use super::{
        ensure_status_releasing_output_on_error, project_root_for_abi, release_owned_buffer,
        release_owned_buffer_after_result, validate_owned_buffer_releasing_on_error,
        validate_plugin_event_batch, validate_plugin_event_encoded_len, validate_runtime_frame,
        validate_runtime_frame_releasing_on_error, RuntimeFrame, RuntimeLibraryError,
        RuntimeSessionTeardownFailureState,
    };
    use zircon_runtime_interface::{
        ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimeFrameV1, ZrRuntimePluginEventDeliveryBatchV1,
        ZrRuntimePluginEventDeliveryV1, ZrRuntimePluginEventSubscriptionHandle, ZrStatus,
        ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1,
        ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1,
    };

    const FRAME_RELEASE_DIAGNOSTIC: &[u8] = b"frame allocation still in use";
    const CAPTURE_DIAGNOSTIC: &[u8] = b"capture submission rejected";
    static EMPTY_BUFFER_RELEASED: AtomicBool = AtomicBool::new(false);
    static BOX_BUFFER_RELEASED: AtomicBool = AtomicBool::new(false);
    static FRAME_PIXELS_RELEASED: AtomicBool = AtomicBool::new(false);

    unsafe extern "C" fn record_empty_buffer_release(buffer: ZrOwnedByteBuffer) -> ZrStatus {
        if !buffer.data.is_null() {
            unsafe {
                drop(Box::from_raw(buffer.data));
            }
        }
        EMPTY_BUFFER_RELEASED.store(true, Ordering::Release);
        ZrStatus::ok()
    }

    unsafe extern "C" fn release_box_buffer(buffer: ZrOwnedByteBuffer) -> ZrStatus {
        if !buffer.data.is_null() {
            unsafe {
                drop(Box::from_raw(buffer.data));
            }
        }
        BOX_BUFFER_RELEASED.store(true, Ordering::Release);
        ZrStatus::ok()
    }

    unsafe extern "C" fn release_frame_pixels(buffer: ZrOwnedByteBuffer) -> ZrStatus {
        if !buffer.data.is_null() {
            unsafe {
                drop(Box::from_raw(buffer.data));
            }
        }
        FRAME_PIXELS_RELEASED.store(true, Ordering::Release);
        ZrStatus::ok()
    }

    unsafe extern "C" fn reject_frame_buffer_release(buffer: ZrOwnedByteBuffer) -> ZrStatus {
        if !buffer.data.is_null() {
            unsafe {
                drop(Box::from_raw(buffer.data));
            }
        }
        ZrStatus::new(
            ZrStatusCode::Error,
            ZrByteSlice::from_static(FRAME_RELEASE_DIAGNOSTIC),
        )
    }

    #[test]
    fn runtime_frame_release_failure_is_recorded_for_terminal_teardown() {
        let teardown_failure_state = RuntimeSessionTeardownFailureState::default();
        let mut frame = ZrRuntimeFrameV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
        frame.width = 1;
        frame.height = 1;
        frame.rgba = ZrOwnedByteBuffer {
            data: Box::into_raw(Box::new(0_u8)),
            len: 1,
            capacity: 1,
            owner_token: 1,
            free: Some(reject_frame_buffer_release),
        };

        drop(RuntimeFrame {
            frame,
            teardown_failure_state: teardown_failure_state.clone(),
            _session: PhantomData,
        });

        assert_eq!(
            teardown_failure_state.take().unwrap().to_string(),
            "failed to free runtime frame buffer: error: frame allocation still in use"
        );
    }

    #[test]
    fn runtime_frame_type_retains_the_session_lifetime() {
        let source = include_str!("runtime_session.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("runtime session production source should precede its tests");

        assert!(production.contains(") -> Result<RuntimeFrame<'_>, RuntimeLibraryError> {"));
        assert!(production.contains("_session: PhantomData<&'session RuntimeSession>"));
    }

    #[test]
    fn runtime_project_root_abi_preserves_unicode_and_absence() {
        assert_eq!(project_root_for_abi(None).unwrap(), None);
        assert_eq!(
            project_root_for_abi(Some(std::path::Path::new("E:/projects/\u{9879}\u{76ee}")))
                .unwrap(),
            Some("E:/projects/\u{9879}\u{76ee}")
        );
    }

    #[test]
    #[cfg(any(windows, unix))]
    fn runtime_project_root_abi_rejects_unrepresentable_os_paths() {
        #[cfg(windows)]
        let path = {
            use std::os::windows::ffi::OsStringExt;

            std::path::PathBuf::from(std::ffi::OsString::from_wide(&[0xd800]))
        };
        #[cfg(unix)]
        let path = {
            use std::os::unix::ffi::OsStringExt;

            std::path::PathBuf::from(std::ffi::OsString::from_vec(vec![0xff]))
        };

        let error = project_root_for_abi(Some(&path))
            .expect_err("lossy project roots must not cross the runtime ABI");

        assert!(error
            .to_string()
            .contains("runtime project root cannot cross the UTF-8 ABI boundary"));
    }

    #[test]
    fn empty_owned_buffer_still_invokes_its_release_callback() {
        EMPTY_BUFFER_RELEASED.store(false, Ordering::Release);
        let output = ZrOwnedByteBuffer {
            data: Box::into_raw(Box::new(0_u8)),
            len: 0,
            capacity: 1,
            owner_token: 7,
            free: Some(record_empty_buffer_release),
        };
        assert_eq!(output.len, 0);
        assert!(!output.data.is_null());

        release_owned_buffer(output, "free empty runtime output").unwrap();

        assert!(EMPTY_BUFFER_RELEASED.load(Ordering::Acquire));
    }

    #[test]
    fn failed_output_call_retains_call_and_release_diagnostics() {
        let output = ZrOwnedByteBuffer {
            data: Box::into_raw(Box::new(0_u8)),
            len: 1,
            capacity: 1,
            owner_token: 2,
            free: Some(reject_frame_buffer_release),
        };

        let error = ensure_status_releasing_output_on_error(
            ZrStatus::new(
                ZrStatusCode::Error,
                ZrByteSlice::from_static(CAPTURE_DIAGNOSTIC),
            ),
            "capture runtime frame",
            output,
            "free runtime frame output after failed capture",
        )
        .expect_err("call and release failures must reject the runtime operation");

        assert_eq!(
            error.to_string(),
            "failed to capture runtime frame: error: capture submission rejected; cleanup also failed: failed to free runtime frame output after failed capture: error: frame allocation still in use"
        );
    }

    #[test]
    fn malformed_owned_buffer_is_released_and_rejected_before_decode() {
        BOX_BUFFER_RELEASED.store(false, Ordering::Release);
        let output = ZrOwnedByteBuffer {
            data: Box::into_raw(Box::new(0_u8)),
            len: 2,
            capacity: 1,
            owner_token: 3,
            free: Some(release_box_buffer),
        };

        let error = validate_owned_buffer_releasing_on_error(
            output,
            "decode runtime host requests",
            "free runtime host requests",
        )
        .expect_err("malformed runtime-owned storage must be rejected before slicing");

        assert_eq!(
            error.to_string(),
            "decode runtime host requests returned malformed storage: len 2 exceeds capacity 1"
        );
        assert!(BOX_BUFFER_RELEASED.load(Ordering::Acquire));
    }

    #[test]
    fn decode_and_release_failures_preserve_both_diagnostics() {
        let output = ZrOwnedByteBuffer {
            data: Box::into_raw(Box::new(0_u8)),
            len: 1,
            capacity: 1,
            owner_token: 4,
            free: Some(reject_frame_buffer_release),
        };

        let result: Result<(), RuntimeLibraryError> = release_owned_buffer_after_result(
            output,
            Err(RuntimeLibraryError::new(
                "decode runtime host requests: expected value",
            )),
            "free runtime host requests",
        );
        let error = result.expect_err("decode and cleanup failures must both remain visible");

        assert_eq!(
            error.to_string(),
            "decode runtime host requests: expected value; cleanup also failed: failed to free runtime host requests: error: frame allocation still in use"
        );
    }

    #[test]
    fn runtime_frame_validation_rejects_foreign_abi() {
        let frame = ZrRuntimeFrameV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1 + 1);

        let error = validate_runtime_frame(&frame)
            .expect_err("a successful capture must use the negotiated frame ABI");

        assert_eq!(
            error.to_string(),
            "runtime frame used unsupported ABI version 2"
        );
    }

    #[test]
    fn runtime_frame_validation_rejects_zero_dimensions_and_length_overflow() {
        let zero_width = ZrRuntimeFrameV1 {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            width: 0,
            height: 1,
            generation: 1,
            rgba: ZrOwnedByteBuffer::empty(),
        };
        assert_eq!(
            validate_runtime_frame(&zero_width)
                .expect_err("a successful capture must have non-zero dimensions")
                .to_string(),
            "runtime frame returned invalid dimensions 0x1"
        );

        let overflow = ZrRuntimeFrameV1 {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            width: u32::MAX,
            height: u32::MAX,
            generation: 1,
            rgba: ZrOwnedByteBuffer::empty(),
        };
        assert_eq!(
            validate_runtime_frame(&overflow)
                .expect_err("unrepresentable RGBA lengths must reject capture")
                .to_string(),
            "runtime frame pixel length overflowed usize"
        );
    }

    #[test]
    fn frame_protocol_and_release_failures_preserve_both_diagnostics() {
        let frame = ZrRuntimeFrameV1 {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1 + 1,
            width: 1,
            height: 1,
            generation: 1,
            rgba: ZrOwnedByteBuffer {
                data: Box::into_raw(Box::new(0_u8)),
                len: 1,
                capacity: 1,
                owner_token: 6,
                free: Some(reject_frame_buffer_release),
            },
        };

        let error = validate_runtime_frame_releasing_on_error(&frame)
            .expect_err("frame protocol and cleanup failures must both remain visible");

        assert_eq!(
            error.to_string(),
            "runtime frame used unsupported ABI version 2; cleanup also failed: failed to free runtime frame output after invalid capture: error: frame allocation still in use"
        );
    }

    #[test]
    fn plugin_event_batch_rejects_crossed_subscriptions_and_oversized_pages() {
        assert_eq!(
            validate_plugin_event_encoded_len(
                ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1 + 1
            )
            .expect_err("encoded pages above the ABI bound must be rejected")
            .to_string(),
            "runtime plugin event page returned 262145 encoded bytes; maximum is 262144"
        );

        let requested = ZrRuntimePluginEventSubscriptionHandle::new(7);
        let crossed = ZrRuntimePluginEventDeliveryBatchV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            vec![ZrRuntimePluginEventDeliveryV1::new(
                1,
                ZrRuntimePluginEventSubscriptionHandle::new(8),
                "zircon.test.event",
                "zircon.test.v1",
                1,
                serde_json::Value::Null,
            )],
        );
        assert_eq!(
            validate_plugin_event_batch(&crossed, requested)
                .expect_err("deliveries from another subscription must be rejected")
                .to_string(),
            "runtime plugin event delivery subscription 8 did not match requested subscription 7"
        );

        let delivery = ZrRuntimePluginEventDeliveryV1::new(
            1,
            requested,
            "zircon.test.event",
            "zircon.test.v1",
            1,
            serde_json::Value::Null,
        );
        let oversized = ZrRuntimePluginEventDeliveryBatchV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            vec![delivery; ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1 + 1],
        );
        assert_eq!(
            validate_plugin_event_batch(&oversized, requested)
                .expect_err("delivery pages above the ABI bound must be rejected")
                .to_string(),
            "runtime plugin event batch returned 65 deliveries; maximum is 64"
        );
    }

    #[test]
    fn runtime_frame_validation_releases_truncated_pixels() {
        FRAME_PIXELS_RELEASED.store(false, Ordering::Release);
        let frame = ZrRuntimeFrameV1 {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            width: 1,
            height: 1,
            generation: 1,
            rgba: ZrOwnedByteBuffer {
                data: Box::into_raw(Box::new(0_u8)),
                len: 1,
                capacity: 1,
                owner_token: 5,
                free: Some(release_frame_pixels),
            },
        };

        let error = validate_runtime_frame_releasing_on_error(&frame)
            .expect_err("truncated runtime frame pixels must reject capture");

        assert_eq!(
            error.to_string(),
            "runtime frame returned 1 RGBA bytes for 1x1 pixels; expected 4"
        );
        assert!(FRAME_PIXELS_RELEASED.load(Ordering::Acquire));
    }
}
