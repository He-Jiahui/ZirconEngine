use std::ptr;

use zircon_runtime_interface::world_sync::{WatchRegistration, WatchToken, WorldQuery};
use zircon_runtime_interface::{
    ProfileControlCommand, ProfileControlRequest, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZIRCON_RUNTIME_ABI_VERSION_V3, ZrByteSlice, ZrOwnedByteBuffer,
    ZrRuntimeAccessibilityTreeRequestV1, ZrRuntimeBindViewportSurfaceRequestV1, ZrRuntimeEventV1,
    ZrRuntimeFrameDemandV1, ZrRuntimeFrameRequestV1, ZrRuntimeFrameV1, ZrRuntimeHighlightSetV1,
    ZrRuntimePluginEventDeliveryBatchV1, ZrRuntimePluginEventSubscribeRequestV1,
    ZrRuntimePluginEventSubscriptionHandle, ZrRuntimeSessionConfigV3, ZrRuntimeSessionHandle,
    ZrRuntimeViewportHandle, ZrStatus,
};

use super::super::frame::{
    encode_accessibility_tree, encode_host_request_batch, encode_profile_response,
    encode_world_sync_payload, write_accessibility_tree, write_frame, write_host_requests,
    write_profile_response, write_world_sync_payload,
};
use super::super::surface::render_surface_descriptor;
use super::RuntimeProjectError;
use super::diagnostics::runtime_diagnostics_response;
use super::profile::RuntimeDynamicSessionProfile;
use super::project::RuntimeProjectConfig;
use super::registry::{
    RuntimeWakeRegistration, destroy_session_slot, insert_session_with_wake, with_session,
    with_session_activity,
};
use super::status::{error_status, invalid_argument, not_found, unsupported_version};
use super::{DEFAULT_VIEWPORT, RuntimeDynamicSession, RuntimeDynamicSessionError, event_mirror};

pub(in crate::dynamic_api) unsafe fn create_session(
    config: ZrRuntimeSessionConfigV3,
    out_session: *mut ZrRuntimeSessionHandle,
) -> ZrStatus {
    crate::profile_scope!("runtime", "dynamic_api", "create_session");
    if out_session.is_null() {
        return invalid_argument(b"missing runtime session output");
    }
    if config.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V3 {
        return unsupported_version();
    }
    let wake = match RuntimeWakeRegistration::from_abi(config.wake_sink) {
        Ok(wake) => wake,
        Err(_) => return invalid_argument(b"invalid runtime wake sink"),
    };

    let profile =
        match RuntimeDynamicSessionProfile::from_bytes(unsafe { config.profile.as_slice() }) {
            Some(profile) => profile,
            None => return invalid_argument(b"unknown runtime session profile"),
        };
    let project_config = match RuntimeProjectConfig::from_abi_startup_config(
        config.project_root,
        config.play_scene,
        config.play_report_pipe,
    ) {
        Ok(project_config) => project_config,
        Err(error) => return invalid_runtime_startup_config(error),
    };

    let mut dynamic_process_log =
        crate::diagnostic_log::acquire_dynamic_unity_process_log("runtime-dynamic");
    crate::diagnostic_log::write_log("runtime_session", "dynamic_api_create_session_entered");

    match RuntimeDynamicSession::new(profile, project_config) {
        Ok(session) => {
            let session = session.with_runtime_frame_wake(wake.channel_wake());
            let handle = insert_session_with_wake(
                session.with_dynamic_process_log_lease(dynamic_process_log),
                wake,
            );
            unsafe { ptr::write(out_session, handle) };
            ZrStatus::ok()
        }
        Err(error) => {
            if !dynamic_process_log.shutdown() {
                eprintln!(
                    "fatal dynamic runtime session bootstrap teardown failure; aborting before dynamic library unload"
                );
                std::process::abort();
            }
            error_status(error)
        }
    }
}

fn invalid_runtime_startup_config(error: RuntimeProjectError) -> ZrStatus {
    match error {
        RuntimeProjectError::PlaySceneRequiresProject => {
            invalid_argument(b"runtime Play scene requires a project root")
        }
        RuntimeProjectError::PlayReportPipeRequiresProject => {
            invalid_argument(b"runtime Play report outlet requires a project root")
        }
        RuntimeProjectError::PlaySceneUtf8 { .. }
        | RuntimeProjectError::EmptyPlayScene
        | RuntimeProjectError::InvalidPlayScene { .. } => {
            invalid_argument(b"invalid runtime Play scene path")
        }
        RuntimeProjectError::PlayReportPipeUtf8 { .. }
        | RuntimeProjectError::EmptyPlayReportPipe => {
            invalid_argument(b"invalid runtime Play report outlet")
        }
        RuntimeProjectError::EmptyProjectRoot | RuntimeProjectError::ProjectRootUtf8 { .. } => {
            invalid_argument(b"invalid runtime project root")
        }
        RuntimeProjectError::ResolveProjectRoot { .. } => {
            invalid_argument(b"could not resolve runtime project root")
        }
        _ => invalid_argument(b"invalid runtime project startup configuration"),
    }
}

pub(in crate::dynamic_api) unsafe fn destroy_session(handle: ZrRuntimeSessionHandle) -> ZrStatus {
    destroy_session_slot(handle)
}

pub(in crate::dynamic_api) unsafe fn handle_event(
    handle: ZrRuntimeSessionHandle,
    event: ZrRuntimeEventV1,
) -> ZrStatus {
    crate::profile_scope!("runtime", "dynamic_api", "handle_event");
    with_session(handle, |session| {
        if event.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
            return unsupported_version();
        }
        session.handle_event(event)
    })
}

pub(in crate::dynamic_api) unsafe fn capture_frame(
    handle: ZrRuntimeSessionHandle,
    request: ZrRuntimeFrameRequestV1,
    out_frame: *mut ZrRuntimeFrameV1,
) -> ZrStatus {
    crate::profile_frame!("runtime", "capture_frame");
    crate::profile_scope!("runtime", "dynamic_api", "capture_frame");
    with_session(handle, |session| {
        if request.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
            return unsupported_version();
        }
        if request.viewport != DEFAULT_VIEWPORT {
            return not_found(b"runtime viewport not found");
        }
        if out_frame.is_null() {
            return write_frame(
                out_frame,
                ZrRuntimeFrameV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1),
            );
        }
        match session.capture_frame(request) {
            Ok(frame) => write_frame(out_frame, frame),
            Err(error) => error_status(error),
        }
    })
}

pub(in crate::dynamic_api) unsafe fn capture_accessibility_tree(
    handle: ZrRuntimeSessionHandle,
    request: ZrRuntimeAccessibilityTreeRequestV1,
    out_tree: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    crate::profile_scope!("runtime", "dynamic_api", "capture_accessibility_tree");
    with_session(handle, |session| {
        if request.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
            return unsupported_version();
        }
        if request.viewport != DEFAULT_VIEWPORT {
            return not_found(b"runtime viewport not found");
        }
        if out_tree.is_null() {
            return write_accessibility_tree(out_tree, ZrOwnedByteBuffer::empty());
        }
        match session
            .capture_accessibility_tree(request)
            .and_then(|snapshot| {
                encode_accessibility_tree(&snapshot).map_err(|source| {
                    RuntimeDynamicSessionError::EncodeAccessibilityTree { source }
                })
            }) {
            Ok(buffer) => write_accessibility_tree(out_tree, buffer),
            Err(error) => error_status(error),
        }
    })
}

pub(in crate::dynamic_api) unsafe fn bind_viewport_surface(
    handle: ZrRuntimeSessionHandle,
    request: ZrRuntimeBindViewportSurfaceRequestV1,
) -> ZrStatus {
    crate::profile_scope!("runtime", "dynamic_api", "bind_viewport_surface");
    with_session(handle, |session| {
        if request.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1
            || request.target.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1
        {
            return unsupported_version();
        }
        if request.viewport != DEFAULT_VIEWPORT {
            return not_found(b"runtime viewport not found");
        }
        let descriptor = match render_surface_descriptor(request) {
            Ok(descriptor) => descriptor,
            Err(status) => return status,
        };
        match session.bind_viewport_surface(descriptor) {
            Ok(()) => ZrStatus::ok(),
            Err(error) => error_status(error),
        }
    })
}

pub(in crate::dynamic_api) unsafe fn unbind_viewport_surface(
    handle: ZrRuntimeSessionHandle,
    viewport: ZrRuntimeViewportHandle,
) -> ZrStatus {
    crate::profile_scope!("runtime", "dynamic_api", "unbind_viewport_surface");
    with_session(handle, |session| {
        if viewport != DEFAULT_VIEWPORT {
            return not_found(b"runtime viewport not found");
        }
        match session.unbind_viewport_surface() {
            Ok(()) => ZrStatus::ok(),
            Err(error) => error_status(error),
        }
    })
}

pub(in crate::dynamic_api) unsafe fn present_viewport(
    handle: ZrRuntimeSessionHandle,
    request: ZrRuntimeFrameRequestV1,
) -> ZrStatus {
    crate::profile_frame!("runtime", "present_viewport");
    crate::profile_scope!("runtime", "dynamic_api", "present_viewport");
    with_session(handle, |session| {
        if request.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
            return unsupported_version();
        }
        if request.viewport != DEFAULT_VIEWPORT {
            return not_found(b"runtime viewport not found");
        }
        match session.present_viewport(request) {
            Ok(()) => ZrStatus::ok(),
            Err(error) => error_status(error),
        }
    })
}

pub(in crate::dynamic_api) unsafe fn submit_highlight_set(
    handle: ZrRuntimeSessionHandle,
    request: ZrRuntimeHighlightSetV1,
) -> ZrStatus {
    with_session(handle, |session| {
        if !unsafe { request.validate() } {
            return invalid_argument(b"invalid runtime highlight set");
        }
        session.submit_highlight_set(request);
        ZrStatus::ok()
    })
}

pub(in crate::dynamic_api) unsafe fn profile_control(
    handle: ZrRuntimeSessionHandle,
    request_json: ZrByteSlice,
    out_json: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    with_session(handle, |session| {
        if out_json.is_null() {
            return write_profile_response(out_json, ZrOwnedByteBuffer::empty());
        }
        if request_json.is_empty() {
            return invalid_argument(b"missing profile control request");
        }
        let request = match serde_json::from_slice::<ProfileControlRequest>(unsafe {
            request_json.as_slice()
        }) {
            Ok(request) => request,
            Err(_) => return invalid_argument(b"invalid profile control request"),
        };
        let response = if request.command == ProfileControlCommand::RuntimeDiagnosticsSnapshot {
            runtime_diagnostics_response(session)
        } else {
            crate::core::diagnostics::profiling::control(request)
        };
        match encode_profile_response(&response) {
            Ok(buffer) => write_profile_response(out_json, buffer),
            Err(error) => error_status(error),
        }
    })
}

pub(in crate::dynamic_api) unsafe fn tick_frame(
    handle: ZrRuntimeSessionHandle,
    out_demand: *mut ZrRuntimeFrameDemandV1,
) -> ZrStatus {
    crate::profile_scope!("runtime", "dynamic_api", "tick_frame");
    with_session_activity(handle, |session, activity| {
        if out_demand.is_null() {
            return invalid_argument(b"missing runtime frame demand output");
        }
        activity.begin_tick();
        match session.tick_frame() {
            Ok(()) => {
                activity.request_frame(session.frame_demand());
                let demand = activity.consume_frame_demand().into_abi();
                unsafe { ptr::write(out_demand, demand) };
                ZrStatus::ok()
            }
            Err(error) => {
                session.reset_frame_demand_after_failed_tick();
                activity.consume_frame_demand();
                error_status(error)
            }
        }
    })
}

pub(in crate::dynamic_api) unsafe fn drain_host_requests(
    handle: ZrRuntimeSessionHandle,
    out_requests: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    crate::profile_scope!("runtime", "dynamic_api", "drain_host_requests");
    with_session(handle, |session| {
        if out_requests.is_null() {
            return write_host_requests(out_requests, ZrOwnedByteBuffer::empty());
        }
        let batch = session.drain_host_requests();
        if batch.requests.is_empty() {
            return write_host_requests(out_requests, ZrOwnedByteBuffer::empty());
        }
        match encode_host_request_batch(&batch) {
            Ok(buffer) => write_host_requests(out_requests, buffer),
            Err(error) => error_status(error),
        }
    })
}

pub(in crate::dynamic_api) unsafe fn subscribe_plugin_event(
    handle: ZrRuntimeSessionHandle,
    request_json: ZrByteSlice,
    out_subscription: *mut ZrRuntimePluginEventSubscriptionHandle,
) -> ZrStatus {
    with_session(handle, |session| {
        if out_subscription.is_null() || request_json.is_empty() {
            return invalid_argument(b"missing runtime plugin event subscription request");
        }
        let request =
            match serde_json::from_slice::<ZrRuntimePluginEventSubscribeRequestV1>(unsafe {
                request_json.as_slice()
            }) {
                Ok(request) => request,
                Err(_) => {
                    return invalid_argument(b"invalid runtime plugin event subscription request");
                }
            };
        if request.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
            return unsupported_version();
        }
        match session.subscribe_plugin_event(request) {
            Ok(subscription) => {
                unsafe { ptr::write(out_subscription, subscription) };
                ZrStatus::ok()
            }
            Err(error) => error_status(error),
        }
    })
}

pub(in crate::dynamic_api) unsafe fn unsubscribe_plugin_event(
    handle: ZrRuntimeSessionHandle,
    subscription: ZrRuntimePluginEventSubscriptionHandle,
) -> ZrStatus {
    with_session(handle, |session| {
        if !subscription.is_valid() {
            return invalid_argument(b"invalid runtime plugin event subscription");
        }
        match session.unsubscribe_plugin_event(subscription) {
            Ok(()) => ZrStatus::ok(),
            Err(error) => error_status(error),
        }
    })
}

pub(in crate::dynamic_api) unsafe fn drain_plugin_events(
    handle: ZrRuntimeSessionHandle,
    subscription: ZrRuntimePluginEventSubscriptionHandle,
    out_deliveries: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    with_session(handle, |session| {
        if !subscription.is_valid() {
            return invalid_argument(b"invalid runtime plugin event subscription");
        }
        if out_deliveries.is_null() {
            return invalid_argument(b"missing runtime plugin event output");
        }
        match session.drain_plugin_events(handle.raw(), subscription) {
            Ok(buffer) => unsafe { event_mirror::write_plugin_event_batch(out_deliveries, buffer) },
            Err(error) => error_status(error),
        }
    })
}

pub(in crate::dynamic_api) unsafe fn query_world(
    handle: ZrRuntimeSessionHandle,
    request_json: ZrByteSlice,
    out_result: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    with_session(handle, |session| {
        if out_result.is_null() || request_json.is_empty() {
            return invalid_argument(b"missing runtime world query request or output");
        }
        let query = match serde_json::from_slice::<WorldQuery>(unsafe { request_json.as_slice() }) {
            Ok(query) => query,
            Err(_) => return invalid_argument(b"invalid runtime world query request"),
        };
        match encode_world_sync_payload(&session.query_world(query)) {
            Ok(payload) => write_world_sync_payload(out_result, payload),
            Err(error) => error_status(error),
        }
    })
}

pub(in crate::dynamic_api) unsafe fn watch_world(
    handle: ZrRuntimeSessionHandle,
    registration_json: ZrByteSlice,
    out_token: *mut WatchToken,
) -> ZrStatus {
    with_session(handle, |session| {
        if out_token.is_null() || registration_json.is_empty() {
            return invalid_argument(b"missing runtime world watch request or output");
        }
        let registration = match serde_json::from_slice::<WatchRegistration>(unsafe {
            registration_json.as_slice()
        }) {
            Ok(registration) => registration,
            Err(_) => return invalid_argument(b"invalid runtime world watch request"),
        };
        unsafe {
            ptr::write(out_token, session.watch_world(registration));
        }
        ZrStatus::ok()
    })
}

pub(in crate::dynamic_api) unsafe fn unwatch_world(
    handle: ZrRuntimeSessionHandle,
    token: WatchToken,
    out_removed: *mut u8,
) -> ZrStatus {
    with_session(handle, |session| {
        if out_removed.is_null() || !token.is_valid() {
            return invalid_argument(b"invalid runtime world watch token or output");
        }
        unsafe {
            ptr::write(out_removed, u8::from(session.unwatch_world(token)));
        }
        ZrStatus::ok()
    })
}

pub(in crate::dynamic_api) unsafe fn drain_world_invalidations(
    handle: ZrRuntimeSessionHandle,
    out_batches: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    with_session(handle, |session| {
        if out_batches.is_null() {
            return invalid_argument(b"missing runtime world invalidation output");
        }
        let batches = session.drain_world_invalidations();
        match encode_world_sync_payload(&batches) {
            Ok(payload) => write_world_sync_payload(out_batches, payload),
            Err(error) => error_status(error),
        }
    })
}
