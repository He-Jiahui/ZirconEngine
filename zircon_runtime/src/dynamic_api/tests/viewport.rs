use super::support::*;

#[test]
fn bind_viewport_surface_rejects_wrong_abi_before_session_lookup() {
    let api = runtime_api();
    let bind = api.bind_viewport_surface.expect("bind_viewport_surface");
    let request = ZrRuntimeBindViewportSurfaceRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1 + 1,
        ZrRuntimeViewportHandle::new(1),
        ZrRuntimeViewportSizeV1::new(64, 48),
        ZrRuntimeNativeSurfaceTargetV1::none(ZIRCON_RUNTIME_ABI_VERSION_V1),
    );

    let status = unsafe { bind(ZrRuntimeSessionHandle::new(99_999), request) };

    assert_eq!(status.status_code(), ZrStatusCode::UnsupportedVersion);
}

#[test]
fn bind_viewport_surface_rejects_wrong_target_abi_before_session_lookup() {
    let api = runtime_api();
    let bind = api.bind_viewport_surface.expect("bind_viewport_surface");
    let request = ZrRuntimeBindViewportSurfaceRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        ZrRuntimeViewportSizeV1::new(64, 48),
        ZrRuntimeNativeSurfaceTargetV1::win32(ZIRCON_RUNTIME_ABI_VERSION_V1 + 1, 1, 0),
    );

    let status = unsafe { bind(ZrRuntimeSessionHandle::new(99_999), request) };

    assert_eq!(status.status_code(), ZrStatusCode::UnsupportedVersion);
}

#[test]
fn bind_viewport_surface_rejects_unknown_viewport_before_session_lookup() {
    let api = runtime_api();
    let bind = api.bind_viewport_surface.expect("bind_viewport_surface");
    let request = ZrRuntimeBindViewportSurfaceRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(44),
        ZrRuntimeViewportSizeV1::new(64, 48),
        ZrRuntimeNativeSurfaceTargetV1::none(ZIRCON_RUNTIME_ABI_VERSION_V1),
    );

    let status = unsafe { bind(ZrRuntimeSessionHandle::new(99_999), request) };

    assert_eq!(status.status_code(), ZrStatusCode::NotFound);
    assert_eq!(status_message(status), "runtime viewport not found");
}

#[test]
fn bind_viewport_surface_with_valid_descriptor_rejects_invalid_session() {
    let api = runtime_api();
    let bind = api.bind_viewport_surface.expect("bind_viewport_surface");
    let request = ZrRuntimeBindViewportSurfaceRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        ZrRuntimeViewportSizeV1::new(64, 48),
        ZrRuntimeNativeSurfaceTargetV1::win32(ZIRCON_RUNTIME_ABI_VERSION_V1, 1, 0),
    );

    let status = unsafe { bind(ZrRuntimeSessionHandle::new(99_999), request) };

    assert_eq!(status.status_code(), ZrStatusCode::NotFound);
    assert_eq!(status_message(status), "runtime session not found");
}

#[test]
fn bind_viewport_surface_rejects_unsupported_surface_target_before_session_lookup() {
    let api = runtime_api();
    let bind = api.bind_viewport_surface.expect("bind_viewport_surface");
    let request = ZrRuntimeBindViewportSurfaceRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        ZrRuntimeViewportSizeV1::new(64, 48),
        ZrRuntimeNativeSurfaceTargetV1::none(ZIRCON_RUNTIME_ABI_VERSION_V1),
    );

    let status = unsafe { bind(ZrRuntimeSessionHandle::new(99_999), request) };

    assert_eq!(status.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(
        status_message(status),
        "unsupported runtime native surface target"
    );
}

#[test]
fn capture_frame_rejects_wrong_abi_before_session_lookup() {
    let api = runtime_api();
    let capture_frame = api.capture_frame.expect("capture_frame");
    let request = ZrRuntimeFrameRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1 + 1,
        ZrRuntimeViewportHandle::new(1),
        ZrRuntimeViewportSizeV1::new(64, 48),
    );
    let mut output = ZrRuntimeFrameV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);

    let status =
        unsafe { capture_frame(ZrRuntimeSessionHandle::new(99_999), request, &mut output) };

    assert_eq!(status.status_code(), ZrStatusCode::UnsupportedVersion);
    assert!(output.is_empty());
}

#[test]
fn capture_frame_rejects_unknown_viewport_before_session_lookup() {
    let api = runtime_api();
    let capture_frame = api.capture_frame.expect("capture_frame");
    let request = ZrRuntimeFrameRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(44),
        ZrRuntimeViewportSizeV1::new(64, 48),
    );
    let mut output = ZrRuntimeFrameV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);

    let status =
        unsafe { capture_frame(ZrRuntimeSessionHandle::new(99_999), request, &mut output) };

    assert_eq!(status.status_code(), ZrStatusCode::NotFound);
    assert_eq!(status_message(status), "runtime viewport not found");
    assert!(output.is_empty());
}

#[test]
fn present_viewport_rejects_unknown_viewport_before_session_lookup() {
    let api = runtime_api();
    let present = api.present_viewport.expect("present_viewport");
    let request = ZrRuntimeFrameRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(44),
        ZrRuntimeViewportSizeV1::new(64, 48),
    );

    let status = unsafe { present(ZrRuntimeSessionHandle::new(99_999), request) };

    assert_eq!(status.status_code(), ZrStatusCode::NotFound);
    assert_eq!(status_message(status), "runtime viewport not found");
}

#[test]
fn unbind_viewport_surface_rejects_unknown_viewport_before_session_lookup() {
    let api = runtime_api();
    let unbind = api
        .unbind_viewport_surface
        .expect("unbind_viewport_surface");

    let status = unsafe {
        unbind(
            ZrRuntimeSessionHandle::new(99_999),
            ZrRuntimeViewportHandle::new(44),
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::NotFound);
    assert_eq!(status_message(status), "runtime viewport not found");
}
