use super::support::*;

#[test]
fn dynamic_api_export_returns_versioned_function_table() {
    let host = ZrHostApiV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
    let api = unsafe { zircon_runtime_get_api_v1(&host) };

    assert!(!api.is_null());
    let api = unsafe { &*api };
    assert_eq!(api.abi_version, ZIRCON_RUNTIME_ABI_VERSION_V1);
    assert!(api.create_session.is_some());
    assert!(api.destroy_session.is_some());
    assert!(api.handle_event.is_some());
    assert!(api.capture_frame.is_some());
    assert!(api.capture_accessibility_tree.is_some());
    assert!(api.bind_viewport_surface.is_some());
    assert!(api.unbind_viewport_surface.is_some());
    assert!(api.present_viewport.is_some());
    assert!(api.profile_control.is_some());
    assert!(api.tick_frame.is_some());
    assert!(api.drain_host_requests.is_some());
}

#[test]
fn dynamic_api_rejects_unsupported_host_version() {
    let host = ZrHostApiV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1 + 1);
    let api = unsafe { zircon_runtime_get_api_v1(&host) };

    assert!(api.is_null());
}

#[test]
fn runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary() {
    let source = include_str!("../exports.rs");
    let session_source = include_str!("../session.rs");

    assert!(source.contains("fn catch_ffi_panic("));
    assert!(source.contains("catch_unwind(AssertUnwindSafe"));
    assert!(source.contains("ZrStatusCode::Panic"));
    assert!(source.contains("runtime dynamic API panic caught at FFI boundary"));
    assert!(source.contains("zircon_runtime_get_api_v1_inner"));
    assert!(source.contains("Err(_) => core::ptr::null()"));
    assert!(!session_source.contains("pub(super) unsafe extern \"C\" fn"));

    for (inner, wrapper) in [
        ("create_session", "create_session_ffi"),
        ("destroy_session", "destroy_session_ffi"),
        ("handle_event", "handle_event_ffi"),
        ("capture_frame", "capture_frame_ffi"),
        (
            "capture_accessibility_tree",
            "capture_accessibility_tree_ffi",
        ),
        ("bind_viewport_surface", "bind_viewport_surface_ffi"),
        ("unbind_viewport_surface", "unbind_viewport_surface_ffi"),
        ("present_viewport", "present_viewport_ffi"),
        ("profile_control", "profile_control_ffi"),
        ("tick_frame", "tick_frame_ffi"),
        ("drain_host_requests", "drain_host_requests_ffi"),
    ] {
        assert!(source.contains(&format!("Some({wrapper})")));
        assert!(source.contains(&format!("fn {wrapper}(")));
        assert!(!source.contains(&format!("Some({inner}),")));
        assert!(session_source.contains(&format!("pub(super) unsafe fn {inner}(")));
    }
}

#[test]
fn runtime_frame_request_defaults_to_viewport_handle_payload() {
    let request = ZrRuntimeFrameRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        ZrRuntimeViewportSizeV1::new(10, 20),
    );
    let frame = ZrRuntimeFrameV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);

    assert_eq!(request.viewport.raw(), 1);
    assert_eq!(request.size.width, 10);
    assert!(frame.is_empty());
}
