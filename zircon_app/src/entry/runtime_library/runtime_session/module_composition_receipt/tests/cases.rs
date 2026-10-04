use super::{require_response, validate_requested_profile};
use zircon_runtime_interface::runtime_build_set::{
    ZrRuntimeDigestV1, ZrRuntimeModuleCompositionReceiptV1, ZrRuntimeModuleCompositionTargetV1,
    ZrRuntimeSessionProfileV1, ZR_RUNTIME_MODULE_COMPOSITION_RECEIPT_SCHEMA_V1,
};
use zircon_runtime_interface::ProfileControlResponse;

#[test]
fn runtime_session_accepts_only_a_current_typed_module_composition_receipt() {
    let receipt = ZrRuntimeModuleCompositionReceiptV1::new(
        1,
        7,
        ZrRuntimeModuleCompositionTargetV1::ClientRuntime,
        None,
        ZrRuntimeSessionProfileV1::Runtime,
        ZrRuntimeDigestV1::parse(
            "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff",
        )
        .unwrap(),
    );
    let mut response = ProfileControlResponse::ok("composition receipt");
    response.module_composition_receipt = Some(receipt.clone());

    assert_eq!(require_response(Some(response)).unwrap(), receipt);
    validate_requested_profile(&receipt, b"runtime").unwrap();

    let mut wrong_target = receipt.clone();
    wrong_target.target_mode = ZrRuntimeModuleCompositionTargetV1::EditorHost;
    assert!(validate_requested_profile(&wrong_target, b"runtime")
        .unwrap_err()
        .to_string()
        .contains("does not match requested runtime profile"));

    let mut wrong_session_profile = receipt.clone();
    wrong_session_profile.session_profile = ZrRuntimeSessionProfileV1::Editor;
    assert!(
        validate_requested_profile(&wrong_session_profile, b"runtime")
            .unwrap_err()
            .to_string()
            .contains("does not match requested runtime profile")
    );

    assert!(require_response(None)
        .unwrap_err()
        .to_string()
        .contains("does not provide required module composition receipt control"));

    assert!(
        require_response(Some(ProfileControlResponse::error("identity unavailable")))
            .unwrap_err()
            .to_string()
            .contains("runtime rejected module composition receipt request")
    );
    assert!(
        require_response(Some(ProfileControlResponse::ok("receipt omitted")))
            .unwrap_err()
            .to_string()
            .contains("runtime omitted the required module composition receipt")
    );

    let mut stale = receipt;
    stale.schema_version = ZR_RUNTIME_MODULE_COMPOSITION_RECEIPT_SCHEMA_V1 + 1;
    let mut response = ProfileControlResponse::ok("stale composition receipt");
    response.module_composition_receipt = Some(stale);
    assert!(require_response(Some(response))
        .unwrap_err()
        .to_string()
        .contains("unsupported module composition receipt schema"));
}
