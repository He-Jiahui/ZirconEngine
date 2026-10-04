use std::mem::MaybeUninit;

use zircon_runtime_host::foreign_output::operation_result_item_count;
use zircon_runtime_host::foreign_output::RuntimeOwnedOutputReleaser;
use zircon_runtime_interface::{
    ZrByteSlice, ZrOwnedResultV2, ZrRuntimeOperationHandle, ZrRuntimeOperationResultV1,
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
            self.output_releaser(),
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
#[path = "tests/operation.rs"]
mod tests;

fn decode_operation_output<T: serde::de::DeserializeOwned>(
    foreign_output: &ForeignOutputState,
    releaser: RuntimeOwnedOutputReleaser,
    call: impl FnOnce(*mut ZrOwnedResultV2) -> zircon_runtime_interface::ZrStatus,
    operation: &'static str,
    validate: impl FnOnce(&T) -> Result<usize, RuntimeLibraryError>,
) -> Result<T, RuntimeLibraryError> {
    let mut output = ZrOwnedResultV2::empty();
    let status = call(&mut output);
    // The closure returns output from the live provider bound to this releaser.
    output = unsafe {
        foreign_output.ensure_call_succeeded(
            status,
            output,
            releaser,
            ForeignOutputKind::OperationResult,
            operation,
            "free runtime operation output",
        )?
    };
    unsafe {
        foreign_output
            .decode_json(
                output,
                releaser,
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
}
