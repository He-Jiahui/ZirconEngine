use std::ptr;

use zircon_runtime_interface::world_sync::{WatchRegistration, WatchToken, WorldQuery};
use zircon_runtime_interface::{
    ProfileControlCommand, ProfileControlRequest, ZrByteSlice, ZrOwnedResultV2,
    ZrRuntimeAccessibilityTreeRequestV1, ZrRuntimeAllocationId,
    ZrRuntimeBindViewportSurfaceRequestV1, ZrRuntimeEventV1, ZrRuntimeFrameDemandV1,
    ZrRuntimeFrameRequestV1, ZrRuntimeFrameV2, ZrRuntimeHighlightSetV1,
    ZrRuntimePluginEventDeliveryBatchV1, ZrRuntimePluginEventSubscribeRequestV1,
    ZrRuntimePluginEventSubscriptionHandle, ZrRuntimeSessionConfigV3, ZrRuntimeSessionHandle,
    ZrRuntimeViewportHandle, ZrStatus, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZIRCON_RUNTIME_ABI_VERSION_V2, ZIRCON_RUNTIME_ABI_VERSION_V3,
};

use super::super::frame::{
    encode_accessibility_tree, encode_host_request_batch, encode_profile_response,
    encode_world_sync_payload, write_accessibility_tree, write_frame, write_host_requests,
    write_profile_response, write_world_sync_payload,
};
use super::super::surface::render_surface_descriptor;
use super::diagnostics::runtime_diagnostics_response;
use super::profile::RuntimeDynamicSessionProfile;
use super::project::RuntimeProjectConfig;
use super::registry::{
    destroy_session_slot, insert_session_with_wake, register_runtime_allocation_in_action,
    release_runtime_allocation, with_session, with_session_activity, with_session_result_finalized,
    RuntimeAllocationKind, RuntimeWakeRegistration,
};
use super::status::{error_status, invalid_argument, not_found, unsupported_version};
use super::RuntimeProjectError;
use super::{event_mirror, RuntimeDynamicSession, RuntimeDynamicSessionError, DEFAULT_VIEWPORT};

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

pub(in crate::dynamic_api) unsafe fn release_allocation(
    session: ZrRuntimeSessionHandle,
    allocation: ZrRuntimeAllocationId,
) -> ZrStatus {
    release_runtime_allocation(session, allocation)
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
    out_frame: *mut ZrRuntimeFrameV2,
) -> ZrStatus {
    crate::profile_frame!("runtime", "capture_frame");
    crate::profile_scope!("runtime", "dynamic_api", "capture_frame");
    if out_frame.is_null() {
        return write_frame(
            out_frame,
            ZrRuntimeFrameV2::empty(ZIRCON_RUNTIME_ABI_VERSION_V2),
        );
    }
    match with_session_result_finalized(
        handle,
        |session| {
            if request.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
                return Err(unsupported_version());
            }
            if request.viewport != DEFAULT_VIEWPORT {
                return Err(not_found(b"runtime viewport not found"));
            }
            session.capture_frame(request).map_err(error_status)
        },
        |active_handle, frame| {
            let rgba = register_runtime_allocation_in_action(
                active_handle,
                RuntimeAllocationKind::Frame,
                frame.rgba,
            )?;
            Ok(write_frame(
                out_frame,
                ZrRuntimeFrameV2 {
                    abi_version: ZIRCON_RUNTIME_ABI_VERSION_V2,
                    width: frame.width,
                    height: frame.height,
                    generation: frame.generation,
                    rgba,
                },
            ))
        },
    ) {
        Ok(status) | Err(status) => status,
    }
}

pub(in crate::dynamic_api) unsafe fn capture_accessibility_tree(
    handle: ZrRuntimeSessionHandle,
    request: ZrRuntimeAccessibilityTreeRequestV1,
    out_tree: *mut ZrOwnedResultV2,
) -> ZrStatus {
    crate::profile_scope!("runtime", "dynamic_api", "capture_accessibility_tree");
    if out_tree.is_null() {
        return write_accessibility_tree(out_tree, ZrOwnedResultV2::empty());
    }
    match with_session_result_finalized(
        handle,
        |session| {
            if request.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
                return Err(unsupported_version());
            }
            if request.viewport != DEFAULT_VIEWPORT {
                return Err(not_found(b"runtime viewport not found"));
            }
            session
                .capture_accessibility_tree(request)
                .and_then(|snapshot| {
                    encode_accessibility_tree(&snapshot).map_err(|source| {
                        RuntimeDynamicSessionError::EncodeAccessibilityTree { source }
                    })
                })
                .map_err(error_status)
        },
        |active_handle, bytes| {
            let output = register_runtime_allocation_in_action(
                active_handle,
                RuntimeAllocationKind::Accessibility,
                bytes,
            )?;
            Ok(write_accessibility_tree(out_tree, output))
        },
    ) {
        Ok(status) | Err(status) => status,
    }
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
    out_json: *mut ZrOwnedResultV2,
) -> ZrStatus {
    if out_json.is_null() {
        return write_profile_response(out_json, ZrOwnedResultV2::empty());
    }
    match with_session_result_finalized(
        handle,
        |session| {
            if request_json.is_empty() {
                return Err(invalid_argument(b"missing profile control request"));
            }
            let request = match serde_json::from_slice::<ProfileControlRequest>(unsafe {
                request_json.as_slice()
            }) {
                Ok(request) => request,
                Err(_) => return Err(invalid_argument(b"invalid profile control request")),
            };
            let response = if request.command == ProfileControlCommand::RuntimeDiagnosticsSnapshot {
                runtime_diagnostics_response(session)
            } else {
                crate::core::diagnostics::profiling::control(request)
            };
            encode_profile_response(&response).map_err(error_status)
        },
        |active_handle, bytes| {
            let output = register_runtime_allocation_in_action(
                active_handle,
                RuntimeAllocationKind::Profile,
                bytes,
            )?;
            Ok(write_profile_response(out_json, output))
        },
    ) {
        Ok(status) | Err(status) => status,
    }
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
    out_requests: *mut ZrOwnedResultV2,
) -> ZrStatus {
    crate::profile_scope!("runtime", "dynamic_api", "drain_host_requests");
    if out_requests.is_null() {
        return write_host_requests(out_requests, ZrOwnedResultV2::empty());
    }
    match with_session_result_finalized(
        handle,
        |session| {
            let batch = session.drain_host_requests();
            if batch.requests.is_empty() {
                return Ok(Vec::new());
            }
            encode_host_request_batch(&batch).map_err(error_status)
        },
        |active_handle, bytes| {
            let output = register_runtime_allocation_in_action(
                active_handle,
                RuntimeAllocationKind::HostRequests,
                bytes,
            )?;
            Ok(write_host_requests(out_requests, output))
        },
    ) {
        Ok(status) | Err(status) => status,
    }
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
    out_deliveries: *mut ZrOwnedResultV2,
) -> ZrStatus {
    if !subscription.is_valid() {
        return invalid_argument(b"invalid runtime plugin event subscription");
    }
    if out_deliveries.is_null() {
        return invalid_argument(b"missing runtime plugin event output");
    }
    match with_session_result_finalized(
        handle,
        |session| {
            session
                .drain_plugin_events(handle.raw(), subscription)
                .map_err(error_status)
        },
        |active_handle, bytes| {
            let output = register_runtime_allocation_in_action(
                active_handle,
                RuntimeAllocationKind::PluginEvents,
                bytes,
            )?;
            unsafe { ptr::write(out_deliveries, output) };
            Ok(ZrStatus::ok())
        },
    ) {
        Ok(status) | Err(status) => status,
    }
}

pub(in crate::dynamic_api) unsafe fn query_world(
    handle: ZrRuntimeSessionHandle,
    request_json: ZrByteSlice,
    out_result: *mut ZrOwnedResultV2,
) -> ZrStatus {
    if out_result.is_null() || request_json.is_empty() {
        return invalid_argument(b"missing runtime world query request or output");
    }
    match with_session_result_finalized(
        handle,
        |session| {
            let query =
                match serde_json::from_slice::<WorldQuery>(unsafe { request_json.as_slice() }) {
                    Ok(query) => query,
                    Err(_) => {
                        return Err(invalid_argument(b"invalid runtime world query request"));
                    }
                };
            encode_world_sync_payload(&session.query_world(query)).map_err(error_status)
        },
        |active_handle, bytes| {
            let output = register_runtime_allocation_in_action(
                active_handle,
                RuntimeAllocationKind::WorldSync,
                bytes,
            )?;
            Ok(write_world_sync_payload(out_result, output))
        },
    ) {
        Ok(status) | Err(status) => status,
    }
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
    out_batches: *mut ZrOwnedResultV2,
) -> ZrStatus {
    if out_batches.is_null() {
        return invalid_argument(b"missing runtime world invalidation output");
    }
    match with_session_result_finalized(
        handle,
        |session| {
            let batches = session.drain_world_invalidations();
            encode_world_sync_payload(&batches).map_err(error_status)
        },
        |active_handle, bytes| {
            let output = register_runtime_allocation_in_action(
                active_handle,
                RuntimeAllocationKind::WorldSync,
                bytes,
            )?;
            Ok(write_world_sync_payload(out_batches, output))
        },
    ) {
        Ok(status) | Err(status) => status,
    }
}
