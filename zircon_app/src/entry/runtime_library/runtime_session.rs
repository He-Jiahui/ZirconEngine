use std::marker::PhantomData;
use std::path::Path;
use std::slice;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use zircon_runtime::diagnostic_log::write_log;
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

mod foreign_output;
mod operation;
mod owned_buffer;

use foreign_output::{
    ForeignOutputKind, ForeignOutputState, HOST_REQUEST_OUTPUT_BUDGET, PLUGIN_EVENT_OUTPUT_BUDGET,
    PROFILE_RESPONSE_OUTPUT_BUDGET,
};
#[cfg(test)]
use owned_buffer::release_owned_buffer_after_result;
use owned_buffer::{
    ensure_status_releasing_output_on_error, release_owned_buffer,
    release_owned_buffer_after_error, validate_owned_buffer,
    validate_owned_buffer_releasing_on_error,
};

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
    foreign_output: ForeignOutputState,
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
            .map_err(
                |error| zircon_editor::core::gateway::GatewayError::Protocol {
                    message: error.to_string(),
                },
            )?;
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
            foreign_output: ForeignOutputState::default(),
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
            foreign_output: ForeignOutputState::default(),
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
            .ensure_session_available("capture runtime frame")?;
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
        self.foreign_output
            .ensure_session_available("bind runtime viewport surface")?;
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
        self.foreign_output
            .ensure_session_available("unbind runtime viewport surface")?;
        self.unbind_viewport_surface_for_teardown(viewport)
    }

    fn unbind_viewport_surface_for_teardown(
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
                .reject_protocol(ForeignOutputKind::SessionProtocol, error),
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
        let mut output = ZrOwnedByteBuffer::empty();
        let status = unsafe { drain_host_requests(self.handle, &mut output) };
        self.foreign_output.ensure_call_succeeded(
            status,
            output,
            ForeignOutputKind::HostRequests,
            "drain runtime host requests",
            "free runtime host requests",
        )?;
        let batch = self
            .foreign_output
            .decode_json::<ZrRuntimeHostRequestBatchV1>(
                output,
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
        self.foreign_output.ensure_call_succeeded(
            status,
            output,
            ForeignOutputKind::ProfileResponse,
            "control runtime profiling",
            "free runtime profile response",
        )?;
        self.foreign_output.decode_json::<ProfileControlResponse>(
            output,
            ForeignOutputKind::ProfileResponse,
            PROFILE_RESPONSE_OUTPUT_BUDGET,
            "decode runtime profile response",
            "free runtime profile response",
            |response| Ok(profile_control_response_item_count(response)),
        )
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
            return self.foreign_output.reject_protocol(
                ForeignOutputKind::PluginEvents,
                RuntimeLibraryError::protocol_violation(
                    "runtime returned an invalid plugin event subscription",
                ),
            );
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
        let mut output = ZrOwnedByteBuffer::empty();
        let status = unsafe { drain(self.handle, subscription, &mut output) };
        self.foreign_output.ensure_call_succeeded(
            status,
            output,
            ForeignOutputKind::PluginEvents,
            "drain runtime plugin events",
            "free runtime plugin events",
        )?;
        let batch = self
            .foreign_output
            .decode_json::<ZrRuntimePluginEventDeliveryBatchV1>(
                output,
                ForeignOutputKind::PluginEvents,
                PLUGIN_EVENT_OUTPUT_BUDGET,
                "decode runtime plugin events",
                "free runtime plugin events",
                |batch| {
                    validate_plugin_event_batch(&batch, subscription)?;
                    Ok(batch.deliveries.len())
                },
            )?;
        Ok(batch.map(|batch| batch.deliveries).unwrap_or_default())
    }
}

impl Drop for RuntimeSession {
    fn drop(&mut self) {
        if let Some(diagnostic) = self.foreign_output.diagnostic_line() {
            write_log("runtime_foreign_output", diagnostic);
        }
        if let Err(error) =
            self.unbind_viewport_surface_for_teardown(ZrRuntimeViewportHandle::new(1))
        {
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

fn profile_control_response_item_count(response: &ProfileControlResponse) -> usize {
    let mut count = 1_usize.saturating_add(response.files.len());
    if let Some(snapshot) = &response.snapshot {
        count = count.saturating_add(profile_snapshot_item_count(snapshot));
    }
    if let Some(diagnostics) = &response.runtime_diagnostics {
        count = diagnostics.diagnostic_series.iter().fold(
            count.saturating_add(diagnostics.diagnostic_series.len()),
            |count, series| {
                count
                    .saturating_add(series.subsystem_tags.len())
                    .saturating_add(series.history.len())
            },
        );
        count = count.saturating_add(profile_snapshot_item_count(&diagnostics.profile));
    }
    if let Some(report) = &response.hotspot_report {
        count = count
            .saturating_add(report.hotspots.len())
            .saturating_add(report.hints.len());
    }
    if let Some(report) = &response.counter_hotspot_report {
        count = count
            .saturating_add(report.counters.len())
            .saturating_add(report.hints.len());
    }
    if let Some(report) = &response.ui_hotspot_report {
        count = count
            .saturating_add(report.scenarios.len())
            .saturating_add(report.alerts.len());
    }
    count
}

fn profile_snapshot_item_count(snapshot: &zircon_runtime_interface::ProfileSnapshot) -> usize {
    snapshot
        .frames
        .len()
        .saturating_add(snapshot.spans.len())
        .saturating_add(snapshot.counters.len())
        .saturating_add(snapshot.recorder_retention.len())
}

#[cfg(test)]
mod tests;
