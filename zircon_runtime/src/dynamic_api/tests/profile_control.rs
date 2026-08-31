use super::support::*;

#[test]
fn profile_control_rejects_invalid_json_after_session_action_admission() {
    let api = runtime_api();
    let profile_control = api.profile_control.expect("profile_control");
    let bytes = b"not-json";
    let mut output = ZrOwnedResultV2::empty();
    let session = create_test_session(api);

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

    destroy_test_session(api, session);
    assert_eq!(status.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(status_message(status), "invalid profile control request");
    assert!(output.is_empty());
}

#[test]
fn profile_control_rejects_malformed_and_oversized_slices_before_decode() {
    let api = runtime_api();
    let profile_control = api.profile_control.expect("profile_control");
    let session = create_test_session(api);
    let mut output = ZrOwnedResultV2::empty();

    let malformed = unsafe {
        profile_control(
            session,
            ZrByteSlice {
                data: core::ptr::null(),
                len: 1,
            },
            &mut output,
        )
    };
    assert_session_status(
        malformed,
        ZrStatusCode::InvalidArgument,
        "invalid profile control request byte slice",
    );
    assert!(output.is_empty());

    let byte = b'{';
    let oversized = unsafe {
        profile_control(
            session,
            ZrByteSlice {
                data: &byte,
                len: zircon_runtime_interface::ZR_RUNTIME_PROFILE_REQUEST_LIMIT_V1
                    .max_encoded_bytes
                    + 1,
            },
            &mut output,
        )
    };
    assert_session_status(
        oversized,
        ZrStatusCode::LimitExceeded,
        "profile control request exceeds byte limit",
    );
    assert!(output.is_empty());

    destroy_test_session(api, session);
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
    let mut output = ZrOwnedResultV2::empty();

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
    let response_bytes = output_bytes(&output);
    let response: zircon_runtime_interface::ProfileControlResponse =
        serde_json::from_slice(response_bytes).unwrap();
    assert_eq!(response.status, "ok");
    assert!(response.snapshot.is_some());

    release_output(session, output);
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
    let mut output = ZrOwnedResultV2::empty();

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
    let response_bytes = output_bytes(&output);
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

    release_output(session, output);
    destroy_test_session(api, session);
}

#[test]
fn profile_control_returns_the_frozen_module_composition_receipt() {
    use zircon_runtime_interface::runtime_build_set::{
        ZrRuntimeModuleCompositionTargetV1, ZrRuntimeSessionProfileV1,
        ZR_RUNTIME_MODULE_COMPOSITION_RECEIPT_SCHEMA_V1,
    };

    let api = runtime_api();
    let profile_control = api.profile_control.expect("profile_control");
    let session = create_test_session(api);
    let request = zircon_runtime_interface::ProfileControlRequest {
        command: zircon_runtime_interface::ProfileControlCommand::RuntimeModuleCompositionReceipt,
        config: None,
    };
    let bytes = serde_json::to_vec(&request).unwrap();
    let mut output = ZrOwnedResultV2::empty();

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
    let response: zircon_runtime_interface::ProfileControlResponse =
        serde_json::from_slice(output_bytes(&output)).unwrap();
    let receipt = response
        .module_composition_receipt
        .expect("module composition receipt");

    assert_eq!(response.status, "ok");
    assert_eq!(
        response.message,
        "runtime module composition receipt captured"
    );
    assert_eq!(
        receipt.schema_version,
        ZR_RUNTIME_MODULE_COMPOSITION_RECEIPT_SCHEMA_V1
    );
    assert!(receipt.catalog_generation > 0);
    assert_eq!(
        receipt.target_mode,
        ZrRuntimeModuleCompositionTargetV1::ClientRuntime
    );
    assert_eq!(receipt.module_profile, None);
    assert_eq!(receipt.session_profile, ZrRuntimeSessionProfileV1::Headless);
    assert_eq!(receipt.composition_hash.as_str().len(), 64);

    release_output(session, output);
    destroy_test_session(api, session);
}
