use super::support::*;

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
fn profile_control_runtime_diagnostics_snapshot_returns_store_and_scene_reload_report() {
    let api = runtime_api();
    let profile_control = api.profile_control.expect("profile_control");
    let session = create_test_session(api);
    let request = zircon_runtime_interface::ProfileControlRequest {
        command: zircon_runtime_interface::ProfileControlCommand::RuntimeDiagnosticsSnapshot,
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
    let runtime_diagnostics = response
        .runtime_diagnostics
        .expect("runtime diagnostics snapshot");

    assert_eq!(response.status, "ok");
    assert_eq!(response.message, "runtime diagnostics snapshot captured");
    assert!(runtime_diagnostics.scene_asset_reload.is_some());
    assert!(
        !runtime_diagnostics
            .scene_asset_reload
            .as_ref()
            .expect("scene asset reload diagnostics")
            .enabled
    );
    assert_eq!(runtime_diagnostics.profile.session_id, "local");

    free_profile_output(output);
    destroy_test_session(api, session);
}
