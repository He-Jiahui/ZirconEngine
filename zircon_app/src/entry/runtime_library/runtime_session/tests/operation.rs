use super::{
    decode_operation_output, ensure_operation_output_handle, ensure_operation_result_abi,
    ensure_operation_status, ForeignOutputState, RuntimeOwnedOutputReleaser,
};
use crate::entry::runtime_library::runtime_library_error::RuntimeLibraryErrorKind;
use zircon_runtime_interface::{
    ZrByteSlice, ZrOwnedResultV2, ZrRuntimeAllocationId, ZrRuntimeOperationDetailKindV2,
    ZrRuntimeOperationHandle, ZrRuntimeOperationPhase, ZrRuntimeOperationStatusV2,
    ZrRuntimeSessionHandle, ZrStatus, ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

const OPERATION_RELEASE_DIAGNOSTIC: &[u8] = b"operation allocation still in use";

#[derive(Debug, serde::Deserialize)]
struct TestOperationOutput {
    abi_version: u32,
}

unsafe extern "C" fn return_foreign_operation_output(output: *mut ZrOwnedResultV2) -> ZrStatus {
    static BYTES: &[u8] = br#"{"abi_version":2}"#;
    let owned = ZrOwnedResultV2 {
        data: BYTES.as_ptr(),
        len: BYTES.len() as u64,
        allocation: ZrRuntimeAllocationId::new(1),
    };
    unsafe {
        output.write(owned);
    }
    ZrStatus::ok()
}

unsafe extern "C" fn reject_operation_output_release(
    _session: ZrRuntimeSessionHandle,
    _allocation: ZrRuntimeAllocationId,
) -> ZrStatus {
    ZrStatus::new(
        ZrStatusCode::Error,
        ZrByteSlice::from_static(OPERATION_RELEASE_DIAGNOSTIC),
    )
}

#[test]
fn operation_result_abi_rejects_foreign_versions() {
    let error = ensure_operation_result_abi(
        ZIRCON_RUNTIME_ABI_VERSION_V1 + 1,
        "runtime operation result",
    )
    .expect_err("foreign operation DTO ABI should be rejected");

    assert_eq!(
        error.to_string(),
        "runtime operation result used unsupported ABI version 2"
    );
}

#[test]
fn operation_output_handle_rejects_crossed_responses() {
    let error = ensure_operation_output_handle(
        ZrRuntimeOperationHandle::new(8),
        ZrRuntimeOperationHandle::new(7),
        "runtime operation result",
    )
    .expect_err("a response for another operation must be rejected");

    assert_eq!(
        error.to_string(),
        "runtime operation result handle 8 did not match requested handle 7"
    );
}

#[test]
fn operation_abi_and_release_failures_preserve_both_diagnostics() {
    let error = decode_operation_output(
        &ForeignOutputState::default(),
        unsafe {
            RuntimeOwnedOutputReleaser::new(
                ZrRuntimeSessionHandle::new(1),
                reject_operation_output_release,
            )
        },
        |output| unsafe { return_foreign_operation_output(output) },
        "poll runtime operation",
        |output: &TestOperationOutput| {
            ensure_operation_result_abi(output.abi_version, "runtime operation result")?;
            Ok(1)
        },
    )
    .expect_err("operation ABI and cleanup failures must both remain visible");

    assert_eq!(
        error.to_string(),
        "runtime operation result used unsupported ABI version 2; cleanup also failed: failed to free runtime operation output: error: operation allocation still in use"
    );
}

#[test]
fn operation_status_rejects_reserved_and_unknown_wire_values() {
    let handle = ZrRuntimeOperationHandle::new(7);
    let mut status = ZrRuntimeOperationStatusV2::new(
        handle,
        ZrRuntimeOperationPhase::Queued,
        0,
        1,
        ZrRuntimeOperationDetailKindV2::None,
        0,
    );
    status.reserved = 1;
    let error =
        ensure_operation_status(&status, handle).expect_err("reserved wire field must be rejected");
    assert_eq!(error.kind(), RuntimeLibraryErrorKind::ProtocolViolation);
    assert_eq!(
        error.to_string(),
        "runtime operation status reserved field must be zero, got 1"
    );

    status.reserved = 0;
    status.phase = 99;
    let error =
        ensure_operation_status(&status, handle).expect_err("unknown phase must be rejected");
    assert_eq!(error.kind(), RuntimeLibraryErrorKind::ProtocolViolation);
    assert_eq!(
        error.to_string(),
        "runtime operation status used unknown phase 99"
    );
}
