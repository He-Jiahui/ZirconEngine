use zircon_runtime_interface::runtime_build_set::{
    ZrRuntimeModuleCompositionReceiptV1, ZrRuntimeModuleCompositionTargetV1,
    ZrRuntimeSessionProfileV1, ZR_RUNTIME_MODULE_COMPOSITION_RECEIPT_SCHEMA_V1,
};
use zircon_runtime_interface::{
    ProfileControlCommand, ProfileControlRequest, ProfileControlResponse,
};

use super::{RuntimeLibraryError, RuntimeSession};

pub(super) fn query(
    session: &RuntimeSession,
    requested_profile: &[u8],
) -> Result<ZrRuntimeModuleCompositionReceiptV1, RuntimeLibraryError> {
    let receipt = require_response(session.profile_control(&ProfileControlRequest {
        command: ProfileControlCommand::RuntimeModuleCompositionReceipt,
        config: None,
    })?)?;
    validate_requested_profile(&receipt, requested_profile)?;
    Ok(receipt)
}

pub(super) fn require_response(
    response: Option<ProfileControlResponse>,
) -> Result<ZrRuntimeModuleCompositionReceiptV1, RuntimeLibraryError> {
    let response = response.ok_or_else(|| {
        RuntimeLibraryError::capability_unavailable(
            "runtime does not provide required module composition receipt control",
        )
    })?;
    if response.status != "ok" {
        return Err(RuntimeLibraryError::new(format!(
            "runtime rejected module composition receipt request: {}",
            response.message
        )));
    }
    let receipt = response.module_composition_receipt.ok_or_else(|| {
        RuntimeLibraryError::protocol_violation(
            "runtime omitted the required module composition receipt",
        )
    })?;
    if receipt.schema_version != ZR_RUNTIME_MODULE_COMPOSITION_RECEIPT_SCHEMA_V1 {
        return Err(RuntimeLibraryError::protocol_violation(format!(
            "runtime returned unsupported module composition receipt schema {}; expected {}",
            receipt.schema_version, ZR_RUNTIME_MODULE_COMPOSITION_RECEIPT_SCHEMA_V1
        )));
    }
    Ok(receipt)
}

pub(super) fn validate_requested_profile(
    receipt: &ZrRuntimeModuleCompositionReceiptV1,
    requested_profile: &[u8],
) -> Result<(), RuntimeLibraryError> {
    let expected = match requested_profile {
        [] | b"runtime" => (
            ZrRuntimeSessionProfileV1::Runtime,
            ZrRuntimeModuleCompositionTargetV1::ClientRuntime,
        ),
        b"runtime-pipelined" => (
            ZrRuntimeSessionProfileV1::RuntimePipelined,
            ZrRuntimeModuleCompositionTargetV1::ClientRuntime,
        ),
        b"editor" => (
            ZrRuntimeSessionProfileV1::Editor,
            ZrRuntimeModuleCompositionTargetV1::EditorHost,
        ),
        b"dev" => (
            ZrRuntimeSessionProfileV1::Dev,
            ZrRuntimeModuleCompositionTargetV1::ClientRuntime,
        ),
        b"minimal" => (
            ZrRuntimeSessionProfileV1::Minimal,
            ZrRuntimeModuleCompositionTargetV1::ClientRuntime,
        ),
        b"headless" => (
            ZrRuntimeSessionProfileV1::Headless,
            ZrRuntimeModuleCompositionTargetV1::ClientRuntime,
        ),
        _ => {
            return Err(RuntimeLibraryError::protocol_violation(
                "runtime accepted an unknown requested session profile",
            ));
        }
    };
    if (receipt.session_profile, receipt.target_mode) != expected {
        return Err(RuntimeLibraryError::protocol_violation(format!(
            "runtime module composition receipt does not match requested runtime profile: expected {:?}/{:?}, received {:?}/{:?}",
            expected.0,
            expected.1,
            receipt.session_profile,
            receipt.target_mode
        )));
    }
    Ok(())
}
