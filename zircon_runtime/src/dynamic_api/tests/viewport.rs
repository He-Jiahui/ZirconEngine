use super::support::*;

#[test]
fn bind_viewport_surface_rejects_wrong_abi_after_session_action_admission() {
    let api = runtime_api();
    let bind = api.bind_viewport_surface.expect("bind_viewport_surface");
    let request = ZrRuntimeBindViewportSurfaceRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1 + 1,
        ZrRuntimeViewportHandle::new(1),
        ZrRuntimeViewportSizeV1::new(64, 48),
        ZrRuntimeNativeSurfaceTargetV1::none(ZIRCON_RUNTIME_ABI_VERSION_V1),
    );
    let session = create_test_session(api);

    let status = unsafe { bind(session, request) };

    destroy_test_session(api, session);
    assert_eq!(status.status_code(), ZrStatusCode::UnsupportedVersion);
}

#[test]
fn bind_viewport_surface_rejects_wrong_target_abi_after_session_action_admission() {
    let api = runtime_api();
    let bind = api.bind_viewport_surface.expect("bind_viewport_surface");
    let request = ZrRuntimeBindViewportSurfaceRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        ZrRuntimeViewportSizeV1::new(64, 48),
        ZrRuntimeNativeSurfaceTargetV1::win32(ZIRCON_RUNTIME_ABI_VERSION_V1 + 1, 1, 0),
    );
    let session = create_test_session(api);

    let status = unsafe { bind(session, request) };

    destroy_test_session(api, session);
    assert_eq!(status.status_code(), ZrStatusCode::UnsupportedVersion);
}

#[test]
fn bind_viewport_surface_rejects_unknown_viewport_after_session_action_admission() {
    let api = runtime_api();
    let bind = api.bind_viewport_surface.expect("bind_viewport_surface");
    let request = ZrRuntimeBindViewportSurfaceRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(44),
        ZrRuntimeViewportSizeV1::new(64, 48),
        ZrRuntimeNativeSurfaceTargetV1::none(ZIRCON_RUNTIME_ABI_VERSION_V1),
    );
    let session = create_test_session(api);

    let status = unsafe { bind(session, request) };

    destroy_test_session(api, session);
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
fn bind_viewport_surface_rejects_unsupported_surface_target_after_session_action_admission() {
    let api = runtime_api();
    let bind = api.bind_viewport_surface.expect("bind_viewport_surface");
    let request = ZrRuntimeBindViewportSurfaceRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        ZrRuntimeViewportSizeV1::new(64, 48),
        ZrRuntimeNativeSurfaceTargetV1::none(ZIRCON_RUNTIME_ABI_VERSION_V1),
    );
    let session = create_test_session(api);

    let status = unsafe { bind(session, request) };

    destroy_test_session(api, session);
    assert_eq!(status.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(
        status_message(status),
        "unsupported runtime native surface target"
    );
}

#[test]
fn capture_frame_rejects_wrong_abi_after_session_action_admission() {
    let api = runtime_api();
    let capture_frame = api.capture_frame.expect("capture_frame");
    let request = ZrRuntimeFrameRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1 + 1,
        ZrRuntimeViewportHandle::new(1),
        ZrRuntimeViewportSizeV1::new(64, 48),
    );
    let mut output = ZrRuntimeFrameV2::empty(ZIRCON_RUNTIME_ABI_VERSION_V2);
    let session = create_test_session(api);

    let status = unsafe { capture_frame(session, request, &mut output) };

    destroy_test_session(api, session);
    assert_eq!(status.status_code(), ZrStatusCode::UnsupportedVersion);
    assert!(output.is_empty());
}

#[test]
fn capture_frame_rejects_unknown_viewport_after_session_action_admission() {
    let api = runtime_api();
    let capture_frame = api.capture_frame.expect("capture_frame");
    let request = ZrRuntimeFrameRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(44),
        ZrRuntimeViewportSizeV1::new(64, 48),
    );
    let mut output = ZrRuntimeFrameV2::empty(ZIRCON_RUNTIME_ABI_VERSION_V2);
    let session = create_test_session(api);

    let status = unsafe { capture_frame(session, request, &mut output) };

    destroy_test_session(api, session);
    assert_eq!(status.status_code(), ZrStatusCode::NotFound);
    assert_eq!(status_message(status), "runtime viewport not found");
    assert!(output.is_empty());
}

#[test]
fn capture_frame_rejects_missing_output_before_rendering() {
    let source = include_str!("../session/ffi.rs");
    let start = source
        .find("pub(in crate::dynamic_api) unsafe fn capture_frame(")
        .expect("capture_frame FFI owner");
    let end = source[start..]
        .find("pub(in crate::dynamic_api) unsafe fn capture_accessibility_tree(")
        .map(|offset| start + offset)
        .expect("capture_frame FFI owner end");
    let capture_source = &source[start..end];
    let missing_output_guard = capture_source
        .find("if out_frame.is_null()")
        .expect("capture_frame must reject a missing output before allocating a captured frame");
    let render_call = capture_source
        .find("session.capture_frame(request)")
        .expect("capture_frame render call");
    assert!(
        missing_output_guard < render_call,
        "missing frame output must be rejected before rendering and owning the RGBA payload"
    );

    let api = runtime_api();
    let capture_frame = api.capture_frame.expect("capture_frame");
    let session = create_test_session(api);
    let status = unsafe { capture_frame(session, valid_frame_request(), core::ptr::null_mut()) };

    destroy_test_session(api, session);
    assert_session_status(
        status,
        ZrStatusCode::InvalidArgument,
        "missing frame output",
    );
}

#[test]
fn present_viewport_rejects_unknown_viewport_after_session_action_admission() {
    let api = runtime_api();
    let present = api.present_viewport.expect("present_viewport");
    let request = ZrRuntimeFrameRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(44),
        ZrRuntimeViewportSizeV1::new(64, 48),
    );
    let session = create_test_session(api);

    let status = unsafe { present(session, request) };

    destroy_test_session(api, session);
    assert_eq!(status.status_code(), ZrStatusCode::NotFound);
    assert_eq!(status_message(status), "runtime viewport not found");
}

#[test]
fn unbind_viewport_surface_rejects_unknown_viewport_after_session_action_admission() {
    let api = runtime_api();
    let unbind = api
        .unbind_viewport_surface
        .expect("unbind_viewport_surface");

    let session = create_test_session(api);
    let status = unsafe { unbind(session, ZrRuntimeViewportHandle::new(44)) };

    destroy_test_session(api, session);
    assert_eq!(status.status_code(), ZrStatusCode::NotFound);
    assert_eq!(status_message(status), "runtime viewport not found");
}
