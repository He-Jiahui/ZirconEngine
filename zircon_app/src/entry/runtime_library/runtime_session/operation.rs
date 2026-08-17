use std::mem::MaybeUninit;

use zircon_runtime_host::foreign_output::operation_result_item_count;
use zircon_runtime_interface::{
    ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimeOperationHandle, ZrRuntimeOperationResultV1,
    ZrRuntimeOperationStatusV2, ZrRuntimeOperationSubmitRequestV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZIRCON_RUNTIME_ABI_VERSION_V2,
};

use super::{
    ensure_status,
    foreign_output::{ForeignOutputKind, ForeignOutputState, OPERATION_RESULT_OUTPUT_BUDGET},
    RuntimeLibraryError, RuntimeSession,
};

impl RuntimeSession {
    pub(super) fn submit_operation(
        &self,
        request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, RuntimeLibraryError> {
        self.foreign_output
            .ensure_available(ForeignOutputKind::OperationResult)?;
        let submit = self.runtime().submit_operation();
        let request = serde_json::to_vec(&request)
            .map_err(|error| RuntimeLibraryError::new(error.to_string()))?;
        let mut handle = ZrRuntimeOperationHandle::invalid();
        ensure_status(
            unsafe {
                submit(
                    self.handle,
                    ZrByteSlice {
                        data: request.as_ptr(),
                        len: request.len(),
                    },
                    &mut handle,
                )
            },
            "submit runtime operation",
        )?;
        if !handle.is_valid() {
            return self
                .foreign_output
                .reject_protocol(
                    ForeignOutputKind::OperationResult,
                    RuntimeLibraryError::protocol_violation(
                        "runtime returned an invalid operation handle",
                    ),
                )
                .map_err(Into::into);
        }
        Ok(handle)
    }

    pub(super) fn poll_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationStatusV2, RuntimeLibraryError> {
        self.foreign_output
            .ensure_available(ForeignOutputKind::OperationResult)?;
        let poll = self.runtime().poll_operation();
        let mut status = MaybeUninit::<ZrRuntimeOperationStatusV2>::uninit();
        ensure_status(
            unsafe { poll(self.handle, handle, status.as_mut_ptr()) },
            "poll runtime operation",
        )?;
        let status = unsafe { status.assume_init() };
        if let Err(error) = ensure_operation_status(&status, handle) {
            return self
                .foreign_output
                .reject_protocol(ForeignOutputKind::OperationResult, error)
                .map_err(Into::into);
        }
        Ok(status)
    }

    pub(super) fn harvest_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, RuntimeLibraryError> {
        self.foreign_output
            .ensure_available(ForeignOutputKind::OperationResult)?;
        let harvest = self.runtime().harvest_operation();
        decode_operation_output(
            &self.foreign_output,
            |output| unsafe { harvest(self.handle, handle, output) },
            "harvest runtime operation",
            |result: &ZrRuntimeOperationResultV1| {
                ensure_operation_result_abi(result.abi_version, "runtime operation result")?;
                ensure_operation_output_handle(result.handle, handle, "runtime operation result")?;
                Ok(operation_result_item_count(result))
            },
        )
    }
}

fn ensure_operation_result_abi(
    abi_version: u32,
    output_kind: &'static str,
) -> Result<(), RuntimeLibraryError> {
    if abi_version == ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return Ok(());
    }
    Err(RuntimeLibraryError::protocol_violation(format!(
        "{output_kind} used unsupported ABI version {abi_version}"
    )))
}

fn ensure_operation_status(
    status: &ZrRuntimeOperationStatusV2,
    requested: ZrRuntimeOperationHandle,
) -> Result<(), RuntimeLibraryError> {
    if status.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V2 {
        return Err(RuntimeLibraryError::protocol_violation(format!(
            "runtime operation status used unsupported ABI version {}",
            status.abi_version
        )));
    }
    if status.reserved != 0 {
        return Err(RuntimeLibraryError::protocol_violation(format!(
            "runtime operation status reserved field must be zero, got {}",
            status.reserved
        )));
    }
    ensure_operation_output_handle(status.handle, requested, "runtime operation status")?;
    if status.phase().is_none() {
        return Err(RuntimeLibraryError::protocol_violation(format!(
            "runtime operation status used unknown phase {}",
            status.phase
        )));
    }
    if status.detail_kind().is_none() {
        return Err(RuntimeLibraryError::protocol_violation(format!(
            "runtime operation status used unknown detail kind {}",
            status.detail_kind
        )));
    }
    Ok(())
}

fn ensure_operation_output_handle(
    response: ZrRuntimeOperationHandle,
    requested: ZrRuntimeOperationHandle,
    output_kind: &'static str,
) -> Result<(), RuntimeLibraryError> {
    if response == requested {
        return Ok(());
    }
    Err(RuntimeLibraryError::protocol_violation(format!(
        "{output_kind} handle {} did not match requested handle {}",
        response.raw(),
        requested.raw()
    )))
}

#[cfg(test)]
mod tests {
    use super::{
        decode_operation_output, ensure_operation_output_handle, ensure_operation_result_abi,
        ensure_operation_status, ForeignOutputState,
    };
    use crate::entry::runtime_library::runtime_library_error::RuntimeLibraryErrorKind;
    use zircon_runtime_interface::{
        ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimeOperationDetailKindV2, ZrRuntimeOperationHandle,
        ZrRuntimeOperationPhase, ZrRuntimeOperationStatusV2, ZrStatus, ZrStatusCode,
        ZIRCON_RUNTIME_ABI_VERSION_V1,
    };

    const OPERATION_RELEASE_DIAGNOSTIC: &[u8] = b"operation allocation still in use";

    #[derive(Debug, serde::Deserialize)]
    struct TestOperationOutput {
        abi_version: u32,
    }

    unsafe extern "C" fn return_foreign_operation_output(
        output: *mut ZrOwnedByteBuffer,
    ) -> ZrStatus {
        let mut bytes = br#"{"abi_version":2}"#.to_vec();
        let owned = ZrOwnedByteBuffer {
            data: bytes.as_mut_ptr(),
            len: bytes.len(),
            capacity: bytes.capacity(),
            owner_token: 1,
            free: Some(reject_operation_output_release),
        };
        std::mem::forget(bytes);
        unsafe {
            output.write(owned);
        }
        ZrStatus::ok()
    }

    unsafe extern "C" fn reject_operation_output_release(output: ZrOwnedByteBuffer) -> ZrStatus {
        unsafe {
            drop(Vec::from_raw_parts(
                output.data,
                output.len,
                output.capacity,
            ));
        }
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
        let error = ensure_operation_status(&status, handle)
            .expect_err("reserved wire field must be rejected");
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
}

fn decode_operation_output<T: serde::de::DeserializeOwned>(
    foreign_output: &ForeignOutputState,
    call: impl FnOnce(*mut ZrOwnedByteBuffer) -> zircon_runtime_interface::ZrStatus,
    operation: &'static str,
    validate: impl FnOnce(&T) -> Result<usize, RuntimeLibraryError>,
) -> Result<T, RuntimeLibraryError> {
    let mut output = ZrOwnedByteBuffer::empty();
    let status = call(&mut output);
    foreign_output.ensure_call_succeeded(
        status,
        output,
        ForeignOutputKind::OperationResult,
        operation,
        "free runtime operation output",
    )?;
    foreign_output
        .decode_json(
            output,
            ForeignOutputKind::OperationResult,
            OPERATION_RESULT_OUTPUT_BUDGET,
            operation,
            "free runtime operation output",
            validate,
        )?
        .ok_or_else(|| {
            RuntimeLibraryError::protocol_violation(format!(
                "{operation} returned an empty payload"
            ))
        })
}
