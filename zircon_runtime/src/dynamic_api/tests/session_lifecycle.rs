use super::support::*;

#[test]
fn tick_frame_rejects_unknown_session() {
    let api = runtime_api();
    let tick_frame = api.tick_frame.expect("tick_frame");

    let status = unsafe { tick_frame(ZrRuntimeSessionHandle::new(99_999)) };

    assert_eq!(status.status_code(), ZrStatusCode::NotFound);
    assert_eq!(status_message(status), "runtime session not found");
}

#[test]
fn tick_frame_accepts_valid_session() {
    let api = runtime_api();
    let tick_frame = api.tick_frame.expect("tick_frame");
    let session = create_test_session(api);

    let status = unsafe { tick_frame(session) };

    destroy_test_session(api, session);
    assert_eq!(status.status_code(), ZrStatusCode::Ok);
}

#[test]
fn destroy_session_reports_explicit_not_found_for_missing_nonzero_handle() {
    let api = runtime_api();
    let destroy_session = api.destroy_session.expect("destroy_session");
    let missing_session = ZrRuntimeSessionHandle::new(99_999);

    assert_session_status(
        unsafe { destroy_session(missing_session) },
        ZrStatusCode::NotFound,
        "runtime session not found",
    );
}

#[test]
fn destroy_session_removes_registry_entry_so_destroyed_handles_become_missing() {
    let session_source = include_str!("../session/ffi.rs");
    let exports_source = include_str!("../exports.rs");
    let destroy_start = session_source
        .find("pub(in crate::dynamic_api) unsafe fn destroy_session")
        .expect("destroy_session rust owner");
    let next_entry = session_source[destroy_start..]
        .find("\npub(in crate::dynamic_api) unsafe fn handle_event")
        .map(|offset| destroy_start + offset)
        .expect("entry point after destroy_session");
    let destroy_body = &session_source[destroy_start..next_entry];
    let export_start = exports_source
        .find("unsafe extern \"C\" fn destroy_session_ffi")
        .expect("destroy_session FFI wrapper");
    let next_export = exports_source[export_start..]
        .find("\nunsafe extern \"C\" fn handle_event_ffi")
        .map(|offset| export_start + offset)
        .expect("FFI entry point after destroy_session");
    let export_body = &exports_source[export_start..next_export];

    assert!(destroy_body.contains("if !handle.is_valid()"));
    assert!(destroy_body.contains("registry.sessions.remove(&handle.raw()).is_none()"));
    assert!(destroy_body.contains("return not_found(b\"runtime session not found\")"));
    assert!(export_body.contains("catch_ffi_panic(|| unsafe { destroy_session(handle) })"));
}

#[test]
fn session_destroy_reports_explicit_not_found_after_headless_destroy() {
    let api = runtime_api();
    let destroy_session = api.destroy_session.expect("destroy_session");
    let session = create_test_session(api);

    let first_destroy = unsafe { destroy_session(session) };
    assert_eq!(first_destroy.status_code(), ZrStatusCode::Ok);

    assert_session_status(
        unsafe { destroy_session(session) },
        ZrStatusCode::NotFound,
        "runtime session not found",
    );
}

#[test]
fn create_session_requires_output_pointer() {
    let api = unsafe { &*zircon_runtime_get_api_v2(core::ptr::null()) };
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
fn create_session_rejects_unknown_profile_before_runtime_bootstrap() {
    let api = runtime_api();
    let create_session = api.create_session.expect("create_session");
    let mut session = ZrRuntimeSessionHandle::invalid();
    let profile = b"unknown-profile";

    let status = unsafe {
        create_session(
            ZrRuntimeSessionConfigV1 {
                abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
                profile: ZrByteSlice {
                    data: profile.as_ptr(),
                    len: profile.len(),
                },
                project_manifest: ZrByteSlice::empty(),
            },
            &mut session,
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(status_message(status), "unknown runtime session profile");
    assert!(!session.is_valid());
}

#[test]
fn create_session_rejects_invalid_project_root_before_runtime_bootstrap() {
    let api = runtime_api();
    let create_session = api.create_session.expect("create_session");
    let mut session = ZrRuntimeSessionHandle::invalid();
    let invalid_project_root = [0xff, 0xfe];

    let status = unsafe {
        create_session(
            ZrRuntimeSessionConfigV1 {
                abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
                profile: ZrByteSlice::from_static(b"runtime"),
                project_manifest: ZrByteSlice {
                    data: invalid_project_root.as_ptr(),
                    len: invalid_project_root.len(),
                },
            },
            &mut session,
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(status_message(status), "invalid runtime project root");
    assert!(!session.is_valid());
}
