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
fn create_session_accepts_named_dev_profile() {
    let api = runtime_api();
    let create_session = api.create_session.expect("create_session");
    let mut session = ZrRuntimeSessionHandle::invalid();

    let status = unsafe {
        create_session(
            ZrRuntimeSessionConfigV1 {
                abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
                profile: ZrByteSlice::from_static(b"dev"),
                project_manifest: ZrByteSlice::empty(),
            },
            &mut session,
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
    assert!(session.is_valid());
    destroy_test_session(api, session);
}

#[test]
fn dev_profile_ticks_runtime_diagnostic_store_log_schedule() {
    let source = include_str!("../session.rs");

    assert!(source.contains("DiagnosticStoreLogSchedule::repeating"));
    assert!(source.contains("DEFAULT_DIAGNOSTIC_STORE_LOG_WAIT"));
    assert!(source.contains("collect_runtime_diagnostics(&self.runtime.handle()).store"));
    assert!(source.contains("write_diagnostic_store_snapshot"));
}
