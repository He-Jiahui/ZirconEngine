use super::support::*;

#[test]
fn dynamic_api_export_returns_versioned_function_table() {
    let host = ZrHostApiV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
    let api = unsafe { zircon_runtime_get_api_v6(&host) };

    assert!(!api.is_null());
    let api = unsafe { &*api };
    assert_eq!(api.abi_version, ZIRCON_RUNTIME_API_VERSION_V6);
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
    assert!(api.subscribe_plugin_event.is_some());
    assert!(api.unsubscribe_plugin_event.is_some());
    assert!(api.drain_plugin_events.is_some());
    assert!(api.submit_operation.is_some());
    assert!(api.poll_operation.is_some());
    assert!(api.harvest_operation.is_some());
    assert!(api.query_world.is_some());
    assert!(api.watch_world.is_some());
    assert!(api.unwatch_world.is_some());
    assert!(api.drain_world_invalidations.is_some());
}

#[test]
fn dynamic_api_exports_only_the_v6_runtime_table() {
    let source = include_str!("../exports.rs");

    assert!(!source.contains("ZrRuntimeApiV1"));
    assert!(!source.contains("zircon_runtime_get_api_v1"));
    assert!(!source.contains("ZrRuntimeApiV2"));
    assert!(!source.contains("zircon_runtime_get_api_v2"));
    assert!(!source.contains("ZrRuntimeApiV4"));
    assert!(!source.contains("zircon_runtime_get_api_v4"));
    assert!(source.contains("ZrRuntimeApiV6"));
    assert!(source.contains("zircon_runtime_get_api_v6"));
}

#[test]
fn dynamic_api_rejects_unsupported_host_version() {
    let host = ZrHostApiV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1 + 1);
    let api = unsafe { zircon_runtime_get_api_v6(&host) };

    assert!(api.is_null());
}

#[test]
fn runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary() {
    let source = include_str!("../exports.rs");
    let session_source = include_str!("../session/ffi.rs");
    let operation_source = include_str!("../session/operation.rs");

    assert!(source.contains("fn catch_ffi_panic("));
    assert!(source.contains("catch_unwind(AssertUnwindSafe"));
    assert!(source.contains("ZrStatusCode::Panic"));
    assert!(source.contains("runtime dynamic API panic caught at FFI boundary"));
    assert!(source.contains("zircon_runtime_get_api_v6_inner"));
    assert!(source.contains("Err(_) => core::ptr::null()"));
    assert!(!session_source.contains("pub(super) unsafe extern \"C\" fn"));
    assert!(!operation_source.contains("pub(crate) unsafe extern \"C\" fn"));

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
        ("subscribe_plugin_event", "subscribe_plugin_event_ffi"),
        ("unsubscribe_plugin_event", "unsubscribe_plugin_event_ffi"),
        ("drain_plugin_events", "drain_plugin_events_ffi"),
        ("submit_operation", "submit_operation_ffi"),
        ("poll_operation", "poll_operation_ffi"),
        ("harvest_operation", "harvest_operation_ffi"),
        ("query_world", "query_world_ffi"),
        ("watch_world", "watch_world_ffi"),
        ("unwatch_world", "unwatch_world_ffi"),
        ("drain_world_invalidations", "drain_world_invalidations_ffi"),
    ] {
        assert!(source.contains(&format!("Some({wrapper})")));
        assert!(source.contains(&format!("fn {wrapper}(")));
        assert!(!source.contains(&format!("Some({inner}),")));
        let inner_source = if inner.ends_with("_operation") {
            operation_source
        } else {
            session_source
        };
        let expected_visibility = if inner.ends_with("_operation") {
            "pub(crate)"
        } else {
            "pub(in crate::dynamic_api)"
        };
        assert!(inner_source.contains(&format!("{expected_visibility} unsafe fn {inner}(")));
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
