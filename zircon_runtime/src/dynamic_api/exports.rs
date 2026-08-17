use std::panic::{catch_unwind, AssertUnwindSafe};

use zircon_runtime_interface::world_sync::WatchToken;
use zircon_runtime_interface::{
    ZrByteSlice, ZrHostApiV1, ZrOwnedResultV2, ZrRuntimeAccessibilityTreeRequestV1,
    ZrRuntimeAllocationId, ZrRuntimeApiV7, ZrRuntimeBindViewportSurfaceRequestV1, ZrRuntimeEventV1,
    ZrRuntimeFrameDemandV1, ZrRuntimeFrameRequestV1, ZrRuntimeFrameV2, ZrRuntimeHighlightSetV1,
    ZrRuntimeOperationHandle, ZrRuntimePluginEventSubscriptionHandle, ZrRuntimeSessionConfigV3,
    ZrRuntimeSessionHandle, ZrRuntimeViewportHandle, ZrStatus, ZrStatusCode,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZIRCON_RUNTIME_API_VERSION_V7,
};

use super::session::{
    bind_viewport_surface, capture_accessibility_tree, capture_frame, create_session,
    destroy_session, drain_host_requests, drain_plugin_events, drain_world_invalidations,
    handle_event, harvest_operation, poll_operation, present_viewport, profile_control,
    query_world, release_allocation, submit_highlight_set, submit_operation,
    subscribe_plugin_event, tick_frame, unbind_viewport_surface, unsubscribe_plugin_event,
    unwatch_world, watch_world,
};

static RUNTIME_API_V7: ZrRuntimeApiV7 = ZrRuntimeApiV7 {
    abi_version: ZIRCON_RUNTIME_API_VERSION_V7,
    size_bytes: core::mem::size_of::<ZrRuntimeApiV7>(),
    create_session: Some(create_session_ffi),
    destroy_session: Some(destroy_session_ffi),
    release_allocation: Some(release_allocation_ffi),
    handle_event: Some(handle_event_ffi),
    capture_frame: Some(capture_frame_ffi),
    capture_accessibility_tree: Some(capture_accessibility_tree_ffi),
    bind_viewport_surface: Some(bind_viewport_surface_ffi),
    unbind_viewport_surface: Some(unbind_viewport_surface_ffi),
    present_viewport: Some(present_viewport_ffi),
    submit_highlight_set: Some(submit_highlight_set_ffi),
    profile_control: Some(profile_control_ffi),
    tick_frame: Some(tick_frame_ffi),
    drain_host_requests: Some(drain_host_requests_ffi),
    subscribe_plugin_event: Some(subscribe_plugin_event_ffi),
    unsubscribe_plugin_event: Some(unsubscribe_plugin_event_ffi),
    drain_plugin_events: Some(drain_plugin_events_ffi),
    submit_operation: Some(submit_operation_ffi),
    poll_operation: Some(poll_operation_ffi),
    harvest_operation: Some(harvest_operation_ffi),
    query_world: Some(query_world_ffi),
    watch_world: Some(watch_world_ffi),
    unwatch_world: Some(unwatch_world_ffi),
    drain_world_invalidations: Some(drain_world_invalidations_ffi),
};

#[no_mangle]
pub unsafe extern "C" fn zircon_runtime_get_api_v7(
    host: *const ZrHostApiV1,
) -> *const ZrRuntimeApiV7 {
    match catch_unwind(AssertUnwindSafe(|| unsafe {
        zircon_runtime_get_api_v7_inner(host)
    })) {
        Ok(api) => api,
        Err(_) => core::ptr::null(),
    }
}

unsafe fn zircon_runtime_get_api_v7_inner(host: *const ZrHostApiV1) -> *const ZrRuntimeApiV7 {
    #[cfg(feature = "profiling-tracy")]
    let _ = crate::core::diagnostics::profiling::initialize_tracy_sink();

    if !host_abi_is_supported(host) {
        return core::ptr::null();
    }
    &RUNTIME_API_V7
}

fn host_abi_is_supported(host: *const ZrHostApiV1) -> bool {
    if host.is_null() {
        return true;
    }
    unsafe { (*host).abi_version == ZIRCON_RUNTIME_ABI_VERSION_V1 }
}

fn catch_ffi_panic(call: impl FnOnce() -> ZrStatus) -> ZrStatus {
    match catch_unwind(AssertUnwindSafe(call)) {
        Ok(status) => status,
        Err(_) => ZrStatus::new(
            ZrStatusCode::Panic,
            ZrByteSlice::from_static(b"runtime dynamic API panic caught at FFI boundary"),
        ),
    }
}

unsafe extern "C" fn create_session_ffi(
    config: ZrRuntimeSessionConfigV3,
    out_session: *mut ZrRuntimeSessionHandle,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { create_session(config, out_session) })
}

unsafe extern "C" fn destroy_session_ffi(handle: ZrRuntimeSessionHandle) -> ZrStatus {
    catch_ffi_panic(|| unsafe { destroy_session(handle) })
}

unsafe extern "C" fn release_allocation_ffi(
    session: ZrRuntimeSessionHandle,
    allocation: ZrRuntimeAllocationId,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { release_allocation(session, allocation) })
}

unsafe extern "C" fn handle_event_ffi(
    handle: ZrRuntimeSessionHandle,
    event: ZrRuntimeEventV1,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { handle_event(handle, event) })
}

unsafe extern "C" fn capture_frame_ffi(
    handle: ZrRuntimeSessionHandle,
    request: ZrRuntimeFrameRequestV1,
    out_frame: *mut ZrRuntimeFrameV2,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { capture_frame(handle, request, out_frame) })
}

unsafe extern "C" fn capture_accessibility_tree_ffi(
    handle: ZrRuntimeSessionHandle,
    request: ZrRuntimeAccessibilityTreeRequestV1,
    out_tree: *mut ZrOwnedResultV2,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { capture_accessibility_tree(handle, request, out_tree) })
}

unsafe extern "C" fn bind_viewport_surface_ffi(
    handle: ZrRuntimeSessionHandle,
    request: ZrRuntimeBindViewportSurfaceRequestV1,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { bind_viewport_surface(handle, request) })
}

unsafe extern "C" fn unbind_viewport_surface_ffi(
    handle: ZrRuntimeSessionHandle,
    viewport: ZrRuntimeViewportHandle,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { unbind_viewport_surface(handle, viewport) })
}

unsafe extern "C" fn present_viewport_ffi(
    handle: ZrRuntimeSessionHandle,
    request: ZrRuntimeFrameRequestV1,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { present_viewport(handle, request) })
}

unsafe extern "C" fn submit_highlight_set_ffi(
    handle: ZrRuntimeSessionHandle,
    request: ZrRuntimeHighlightSetV1,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { submit_highlight_set(handle, request) })
}

unsafe extern "C" fn profile_control_ffi(
    handle: ZrRuntimeSessionHandle,
    request_json: ZrByteSlice,
    out_json: *mut ZrOwnedResultV2,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { profile_control(handle, request_json, out_json) })
}

unsafe extern "C" fn tick_frame_ffi(
    handle: ZrRuntimeSessionHandle,
    out_demand: *mut ZrRuntimeFrameDemandV1,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { tick_frame(handle, out_demand) })
}

unsafe extern "C" fn drain_host_requests_ffi(
    handle: ZrRuntimeSessionHandle,
    out_requests: *mut ZrOwnedResultV2,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { drain_host_requests(handle, out_requests) })
}

unsafe extern "C" fn subscribe_plugin_event_ffi(
    handle: ZrRuntimeSessionHandle,
    request_json: ZrByteSlice,
    out_subscription: *mut ZrRuntimePluginEventSubscriptionHandle,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { subscribe_plugin_event(handle, request_json, out_subscription) })
}

unsafe extern "C" fn unsubscribe_plugin_event_ffi(
    handle: ZrRuntimeSessionHandle,
    subscription: ZrRuntimePluginEventSubscriptionHandle,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { unsubscribe_plugin_event(handle, subscription) })
}

unsafe extern "C" fn drain_plugin_events_ffi(
    handle: ZrRuntimeSessionHandle,
    subscription: ZrRuntimePluginEventSubscriptionHandle,
    out_deliveries: *mut ZrOwnedResultV2,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { drain_plugin_events(handle, subscription, out_deliveries) })
}

unsafe extern "C" fn submit_operation_ffi(
    handle: ZrRuntimeSessionHandle,
    request_json: ZrByteSlice,
    out_operation: *mut ZrRuntimeOperationHandle,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { submit_operation(handle, request_json, out_operation) })
}

unsafe extern "C" fn poll_operation_ffi(
    handle: ZrRuntimeSessionHandle,
    operation: ZrRuntimeOperationHandle,
    out_status: *mut zircon_runtime_interface::ZrRuntimeOperationStatusV2,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { poll_operation(handle, operation, out_status) })
}

unsafe extern "C" fn harvest_operation_ffi(
    handle: ZrRuntimeSessionHandle,
    operation: ZrRuntimeOperationHandle,
    out_result: *mut ZrOwnedResultV2,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { harvest_operation(handle, operation, out_result) })
}

unsafe extern "C" fn query_world_ffi(
    handle: ZrRuntimeSessionHandle,
    request_json: ZrByteSlice,
    out_result: *mut ZrOwnedResultV2,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { query_world(handle, request_json, out_result) })
}

unsafe extern "C" fn watch_world_ffi(
    handle: ZrRuntimeSessionHandle,
    registration_json: ZrByteSlice,
    out_token: *mut WatchToken,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { watch_world(handle, registration_json, out_token) })
}

unsafe extern "C" fn unwatch_world_ffi(
    handle: ZrRuntimeSessionHandle,
    token: WatchToken,
    out_removed: *mut u8,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { unwatch_world(handle, token, out_removed) })
}

unsafe extern "C" fn drain_world_invalidations_ffi(
    handle: ZrRuntimeSessionHandle,
    out_batches: *mut ZrOwnedResultV2,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { drain_world_invalidations(handle, out_batches) })
}
