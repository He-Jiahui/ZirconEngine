use zircon_runtime_interface::{ZrHostApiV1, ZrRuntimeApiV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

use super::session::{
    bind_viewport_surface, capture_accessibility_tree, capture_frame, create_session,
    destroy_session, drain_host_requests, handle_event, present_viewport, profile_control,
    tick_frame, unbind_viewport_surface,
};

static RUNTIME_API_V1: ZrRuntimeApiV1 = ZrRuntimeApiV1 {
    abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
    size_bytes: core::mem::size_of::<ZrRuntimeApiV1>(),
    create_session: Some(create_session),
    destroy_session: Some(destroy_session),
    handle_event: Some(handle_event),
    capture_frame: Some(capture_frame),
    capture_accessibility_tree: Some(capture_accessibility_tree),
    bind_viewport_surface: Some(bind_viewport_surface),
    unbind_viewport_surface: Some(unbind_viewport_surface),
    present_viewport: Some(present_viewport),
    profile_control: Some(profile_control),
    tick_frame: Some(tick_frame),
    drain_host_requests: Some(drain_host_requests),
};

#[no_mangle]
pub unsafe extern "C" fn zircon_runtime_get_api_v1(
    host: *const ZrHostApiV1,
) -> *const ZrRuntimeApiV1 {
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
