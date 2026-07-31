use std::ptr;

use zircon_runtime_interface::{
    ProfileControlCommand, ProfileControlRequest, ZrByteSlice, ZrOwnedByteBuffer,
    ZrRuntimeAccessibilityTreeRequestV1, ZrRuntimeBindViewportSurfaceRequestV1, ZrRuntimeEventV1,
    ZrRuntimeFrameDemandV1, ZrRuntimeFrameRequestV1, ZrRuntimeFrameV1,
    ZrRuntimePluginEventDeliveryBatchV1, ZrRuntimePluginEventSubscribeRequestV1,
    ZrRuntimePluginEventSubscriptionHandle, ZrRuntimeSessionConfigV2, ZrRuntimeSessionHandle,
    ZrRuntimeViewportHandle, ZrStatus, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZIRCON_RUNTIME_ABI_VERSION_V2,
};

use super::super::frame::{
    encode_accessibility_tree, encode_host_request_batch, encode_profile_response,
    write_accessibility_tree, write_frame, write_host_requests, write_profile_response,
};
use super::super::surface::render_surface_descriptor;
use super::diagnostics::runtime_diagnostics_response;
use super::profile::RuntimeDynamicSessionProfile;
use super::project::RuntimeProjectConfig;
use super::registry::{
    destroy_session_slot, insert_session_with_wake, with_session, with_session_activity,
    RuntimeWakeRegistration,
};
use super::status::{error_status, invalid_argument, not_found, unsupported_version};
use super::{event_mirror, RuntimeDynamicSession, RuntimeDynamicSessionError, DEFAULT_VIEWPORT};

pub(in crate::dynamic_api) unsafe fn create_session(
    config: ZrRuntimeSessionConfigV2,
    out_session: *mut ZrRuntimeSessionHandle,
) -> ZrStatus {
    crate::profile_scope!("runtime", "dynamic_api", "create_session");
    crate::diagnostic_log::initialize_unity_process_log("runtime-dynamic");
    crate::diagnostic_log::write_log("runtime_session", "dynamic_api_create_session_entered");
    if out_session.is_null() {
        return invalid_argument(b"missing runtime session output");
    }
    if config.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V2 {
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
    let project_config = match RuntimeProjectConfig::from_abi_slice(config.project_manifest) {
        Ok(project_config) => project_config,
        Err(_) => return invalid_argument(b"invalid runtime project root"),
    };

    match RuntimeDynamicSession::new(profile, project_config) {
        Ok(session) => {
            let handle = insert_session_with_wake(session, wake);
            unsafe { ptr::write(out_session, handle) };
            ZrStatus::ok()
        }
        Err(error) => error_status(error),
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
                    return invalid_argument(b"invalid runtime plugin event subscription request")
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
