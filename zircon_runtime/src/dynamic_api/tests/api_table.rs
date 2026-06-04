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
