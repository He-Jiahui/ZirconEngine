use std::panic::{catch_unwind, AssertUnwindSafe};

use zircon_runtime_interface::{
    ZrByteSlice, ZrHostApiV1, ZrOwnedByteBuffer, ZrRuntimeAccessibilityTreeRequestV1,
    ZrRuntimeApiV1, ZrRuntimeBindViewportSurfaceRequestV1, ZrRuntimeEventV1,
    ZrRuntimeFrameRequestV1, ZrRuntimeFrameV1, ZrRuntimeSessionConfigV1, ZrRuntimeSessionHandle,
    ZrRuntimeViewportHandle, ZrStatus, ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use super::session::{
    bind_viewport_surface, capture_accessibility_tree, capture_frame, create_session,
    destroy_session, drain_host_requests, handle_event, present_viewport, profile_control,
    tick_frame, unbind_viewport_surface,
};

static RUNTIME_API_V1: ZrRuntimeApiV1 = ZrRuntimeApiV1 {
    abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
    size_bytes: core::mem::size_of::<ZrRuntimeApiV1>(),
    create_session: Some(create_session_ffi),
    destroy_session: Some(destroy_session_ffi),
    handle_event: Some(handle_event_ffi),
    capture_frame: Some(capture_frame_ffi),
    capture_accessibility_tree: Some(capture_accessibility_tree_ffi),
    bind_viewport_surface: Some(bind_viewport_surface_ffi),
    unbind_viewport_surface: Some(unbind_viewport_surface_ffi),
    present_viewport: Some(present_viewport_ffi),
    profile_control: Some(profile_control_ffi),
    tick_frame: Some(tick_frame_ffi),
    drain_host_requests: Some(drain_host_requests_ffi),
};

#[no_mangle]
pub unsafe extern "C" fn zircon_runtime_get_api_v1(
    host: *const ZrHostApiV1,
) -> *const ZrRuntimeApiV1 {
    match catch_unwind(AssertUnwindSafe(|| unsafe {
        zircon_runtime_get_api_v1_inner(host)
    })) {
        Ok(api) => api,
        Err(_) => core::ptr::null(),
    }
}

unsafe fn zircon_runtime_get_api_v1_inner(host: *const ZrHostApiV1) -> *const ZrRuntimeApiV1 {
    #[cfg(feature = "profiling-tracy")]
    let _ = crate::core::diagnostics::profiling::initialize_tracy_sink();

    if !host.is_null() {
        let host = unsafe { &*host };
        if host.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
            return core::ptr::null();
        }
    }
    &RUNTIME_API_V1
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
    config: ZrRuntimeSessionConfigV1,
    out_session: *mut ZrRuntimeSessionHandle,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { create_session(config, out_session) })
}

unsafe extern "C" fn destroy_session_ffi(handle: ZrRuntimeSessionHandle) -> ZrStatus {
    catch_ffi_panic(|| unsafe { destroy_session(handle) })
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
    out_frame: *mut ZrRuntimeFrameV1,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { capture_frame(handle, request, out_frame) })
}

unsafe extern "C" fn capture_accessibility_tree_ffi(
    handle: ZrRuntimeSessionHandle,
    request: ZrRuntimeAccessibilityTreeRequestV1,
    out_tree: *mut ZrOwnedByteBuffer,
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

unsafe extern "C" fn profile_control_ffi(
    handle: ZrRuntimeSessionHandle,
    request_json: ZrByteSlice,
    out_json: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { profile_control(handle, request_json, out_json) })
}

unsafe extern "C" fn tick_frame_ffi(handle: ZrRuntimeSessionHandle) -> ZrStatus {
    catch_ffi_panic(|| unsafe { tick_frame(handle) })
}

unsafe extern "C" fn drain_host_requests_ffi(
    handle: ZrRuntimeSessionHandle,
    out_requests: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    catch_ffi_panic(|| unsafe { drain_host_requests(handle, out_requests) })
}
