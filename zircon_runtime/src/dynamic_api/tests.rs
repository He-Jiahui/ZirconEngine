use zircon_runtime_interface::{
    ui::{
        accessibility::{
            UiAccessibilityAction, UiAccessibilityActionRequest, UiAccessibilityActionSource,
            UiAccessibilityTreeSnapshot,
        },
        event_ui::UiNodeId,
    },
    ZrByteSlice, ZrHostApiV1, ZrOwnedByteBuffer, ZrRuntimeAccessibilityTreeRequestV1,
    ZrRuntimeBindViewportSurfaceRequestV1, ZrRuntimeEventV1, ZrRuntimeFrameRequestV1,
    ZrRuntimeFrameV1, ZrRuntimeNativeSurfaceTargetV1, ZrRuntimeSessionConfigV1,
    ZrRuntimeSessionHandle, ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1, ZrStatus,
    ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use super::{frame::free_runtime_accessibility_bytes, zircon_runtime_get_api_v1};

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
}

#[test]
fn profile_control_rejects_invalid_json_before_session_lookup() {
    let api = runtime_api();
    let profile_control = api.profile_control.expect("profile_control");
    let bytes = b"not-json";
    let mut output = ZrOwnedByteBuffer::empty();

    let status = unsafe {
        profile_control(
            ZrRuntimeSessionHandle::new(99_999),
            ZrByteSlice {
                data: bytes.as_ptr(),
                len: bytes.len(),
            },
            &mut output,
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(status_message(status), "invalid profile control request");
    assert!(output.is_empty());
}

#[test]
fn profile_control_snapshot_returns_serialized_response() {
    let api = runtime_api();
    let profile_control = api.profile_control.expect("profile_control");
    let session = create_test_session(api);
    let request = zircon_runtime_interface::ProfileControlRequest {
        command: zircon_runtime_interface::ProfileControlCommand::Snapshot,
        config: None,
    };
    let bytes = serde_json::to_vec(&request).unwrap();
    let mut output = ZrOwnedByteBuffer::empty();

    let status = unsafe {
        profile_control(
            session,
            ZrByteSlice {
                data: bytes.as_ptr(),
                len: bytes.len(),
            },
            &mut output,
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::Ok);
    let response_bytes =
        unsafe { core::slice::from_raw_parts(output.data as *const u8, output.len) };
    let response: zircon_runtime_interface::ProfileControlResponse =
        serde_json::from_slice(response_bytes).unwrap();
    assert_eq!(response.status, "ok");
    assert!(response.snapshot.is_some());

    free_profile_output(output);
    destroy_test_session(api, session);
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

#[test]
fn create_session_requires_output_pointer() {
    let api = unsafe { &*zircon_runtime_get_api_v1(core::ptr::null()) };
    let create_session = api.create_session.expect("create_session");
    let status = unsafe {
        create_session(
            ZrRuntimeSessionConfigV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1),
            core::ptr::null_mut::<ZrRuntimeSessionHandle>(),
        )
    };

    assert!(!status.is_ok());
}

#[test]
fn capture_accessibility_tree_requires_output_pointer() {
    let api = runtime_api();
    let capture_accessibility_tree = api
        .capture_accessibility_tree
        .expect("capture_accessibility_tree");
    let session = create_test_session(api);

    let status = unsafe {
        capture_accessibility_tree(
            session,
            accessibility_tree_request(ZIRCON_RUNTIME_ABI_VERSION_V1, 1),
            core::ptr::null_mut::<ZrOwnedByteBuffer>(),
        )
    };

    destroy_test_session(api, session);
    assert_eq!(status.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(status_message(status), "missing accessibility tree output");
}

#[test]
fn capture_accessibility_tree_rejects_wrong_abi_before_session_lookup() {
    let api = runtime_api();
    let capture_accessibility_tree = api
        .capture_accessibility_tree
        .expect("capture_accessibility_tree");
    let mut output = ZrOwnedByteBuffer::empty();

    let status = unsafe {
        capture_accessibility_tree(
            ZrRuntimeSessionHandle::new(99_999),
            accessibility_tree_request(ZIRCON_RUNTIME_ABI_VERSION_V1 + 1, 1),
            &mut output,
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::UnsupportedVersion);
    assert!(output.is_empty());
}

#[test]
fn capture_accessibility_tree_rejects_unknown_viewport() {
    let api = runtime_api();
    let capture_accessibility_tree = api
        .capture_accessibility_tree
        .expect("capture_accessibility_tree");
    let mut output = ZrOwnedByteBuffer::empty();

    let status = unsafe {
        capture_accessibility_tree(
            ZrRuntimeSessionHandle::new(99_999),
            accessibility_tree_request(ZIRCON_RUNTIME_ABI_VERSION_V1, 44),
            &mut output,
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::NotFound);
    assert_eq!(status_message(status), "runtime viewport not found");
    assert!(output.is_empty());
}

#[test]
fn capture_accessibility_tree_returns_serialized_preview_snapshot() {
    let api = runtime_api();
    let capture_accessibility_tree = api
        .capture_accessibility_tree
        .expect("capture_accessibility_tree");
    let session = create_test_session(api);
    let mut output = ZrOwnedByteBuffer::empty();

    let status = unsafe {
        capture_accessibility_tree(
            session,
            accessibility_tree_request(ZIRCON_RUNTIME_ABI_VERSION_V1, 1),
            &mut output,
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::Ok);
    assert!(!output.is_empty());
    assert!(output.free.is_some());

    let bytes = unsafe { core::slice::from_raw_parts(output.data as *const u8, output.len) };
    let snapshot: UiAccessibilityTreeSnapshot = serde_json::from_slice(bytes).unwrap();
    assert_eq!(snapshot.roots, vec![UiNodeId::new(1)]);
    assert_eq!(snapshot.nodes.len(), 1);
    assert_eq!(
        snapshot.nodes[0].name.as_deref(),
        Some("Zircon Runtime Preview")
    );
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.message
            == "runtime UI surface accessibility extraction unavailable in dynamic preview"
    }));

    free_output(output);
    destroy_test_session(api, session);
}

#[test]
fn accessibility_free_rejects_wrong_owner_token() {
    let mut bytes = vec![1_u8, 2, 3];
    let buffer = ZrOwnedByteBuffer {
        data: bytes.as_mut_ptr(),
        len: bytes.len(),
        capacity: bytes.capacity(),
        owner_token: 0,
        free: Some(free_runtime_accessibility_bytes),
    };

    let status = unsafe { free_runtime_accessibility_bytes(buffer) };

    assert_eq!(status.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(
        status_message(status),
        "invalid runtime accessibility buffer"
    );
}

#[test]
fn accessibility_action_event_rejects_invalid_json_payload() {
    let api = runtime_api();
    let handle_event = api.handle_event.expect("handle_event");
    let session = create_test_session(api);
    let payload = b"not-json";
    let event = ZrRuntimeEventV1::accessibility_action(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        ZrByteSlice {
            data: payload.as_ptr(),
            len: payload.len(),
        },
    );

    let status = unsafe { handle_event(session, event) };

    destroy_test_session(api, session);
    assert_eq!(status.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(
        status_message(status),
        "invalid accessibility action payload"
    );
}

#[test]
fn accessibility_action_event_rejects_dynamic_preview_without_surface() {
    let api = runtime_api();
    let handle_event = api.handle_event.expect("handle_event");
    let session = create_test_session(api);
    let request = UiAccessibilityActionRequest {
        target: UiNodeId::new(1),
        action: UiAccessibilityAction::Focus,
        source: UiAccessibilityActionSource::AssistiveTechnology,
        value: None,
        numeric_value: None,
    };
    let bytes = serde_json::to_vec(&request).unwrap();
    let event = ZrRuntimeEventV1::accessibility_action(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        ZrByteSlice {
            data: bytes.as_ptr(),
            len: bytes.len(),
        },
    );

    let status = unsafe { handle_event(session, event) };

    destroy_test_session(api, session);
    assert_eq!(status.status_code(), ZrStatusCode::NotFound);
    assert_eq!(
        status_message(status),
        "runtime UI surface accessibility action dispatch unavailable in dynamic preview"
    );
}

fn runtime_api() -> &'static zircon_runtime_interface::ZrRuntimeApiV1 {
    unsafe { &*zircon_runtime_get_api_v1(core::ptr::null()) }
}

fn accessibility_tree_request(
    abi_version: u32,
    viewport: u64,
) -> ZrRuntimeAccessibilityTreeRequestV1 {
    ZrRuntimeAccessibilityTreeRequestV1::new(
        abi_version,
        ZrRuntimeViewportHandle::new(viewport),
        ZrRuntimeViewportSizeV1::new(64, 48),
        7,
    )
}

fn create_test_session(api: &zircon_runtime_interface::ZrRuntimeApiV1) -> ZrRuntimeSessionHandle {
    let create_session = api.create_session.expect("create_session");
    let mut session = ZrRuntimeSessionHandle::invalid();
    let status = unsafe {
        create_session(
            ZrRuntimeSessionConfigV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1),
            &mut session,
        )
    };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
    session
}

fn destroy_test_session(
    api: &zircon_runtime_interface::ZrRuntimeApiV1,
    session: ZrRuntimeSessionHandle,
) {
    let destroy_session = api.destroy_session.expect("destroy_session");
    let status = unsafe { destroy_session(session) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
}

fn status_message(status: ZrStatus) -> String {
    String::from_utf8(unsafe { status.diagnostics.as_slice() }.to_vec()).unwrap()
}

fn free_output(output: ZrOwnedByteBuffer) {
    let free = output.free.expect("free accessibility output");
    let status = unsafe { free(output) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
}

fn free_profile_output(output: ZrOwnedByteBuffer) {
    let free = output.free.expect("free profile output");
    let status = unsafe { free(output) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
}
