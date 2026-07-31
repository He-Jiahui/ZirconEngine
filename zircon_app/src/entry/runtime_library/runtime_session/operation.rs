use std::slice;

use zircon_runtime_interface::{
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimeOperationHandle,
    ZrRuntimeOperationProgressV1, ZrRuntimeOperationResultV1, ZrRuntimeOperationSubmitRequestV1,
};

use super::{RuntimeLibraryError, RuntimeSession, ensure_status};

impl RuntimeSession {
    pub(super) fn submit_operation(
        &self,
        request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, RuntimeLibraryError> {
        let submit = self.runtime.submit_operation();
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
            return Err(RuntimeLibraryError::new(
                "runtime returned an invalid operation handle",
            ));
        }
        Ok(handle)
    }

    pub(super) fn poll_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationProgressV1, RuntimeLibraryError> {
        let poll = self.runtime.poll_operation();
        let progress: ZrRuntimeOperationProgressV1 = decode_operation_output(
            |output| unsafe { poll(self.handle, handle, output) },
            "poll runtime operation",
        )?;
        ensure_operation_output_abi(progress.abi_version, "runtime operation progress")?;
        Ok(progress)
    }

    pub(super) fn harvest_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, RuntimeLibraryError> {
        let harvest = self.runtime.harvest_operation();
        let result: ZrRuntimeOperationResultV1 = decode_operation_output(
            |output| unsafe { harvest(self.handle, handle, output) },
            "harvest runtime operation",
        )?;
        ensure_operation_output_abi(result.abi_version, "runtime operation result")?;
        Ok(result)
    }
}

fn ensure_operation_output_abi(
    abi_version: u32,
    output_kind: &'static str,
) -> Result<(), RuntimeLibraryError> {
    if abi_version == ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return Ok(());
    }
    Err(RuntimeLibraryError::new(format!(
        "{output_kind} used unsupported ABI version {abi_version}"
    )))
}

#[cfg(test)]
mod tests {
    use super::ensure_operation_output_abi;
    use zircon_runtime_interface::ZIRCON_RUNTIME_ABI_VERSION_V1;

    #[test]
    fn operation_output_abi_rejects_foreign_versions() {
        let error = ensure_operation_output_abi(
            ZIRCON_RUNTIME_ABI_VERSION_V1 + 1,
            "runtime operation progress",
        )
        .expect_err("foreign operation DTO ABI should be rejected");

        assert_eq!(
            error.to_string(),
            "runtime operation progress used unsupported ABI version 2"
        );
    }
}

fn decode_operation_output<T: serde::de::DeserializeOwned>(
    call: impl FnOnce(*mut ZrOwnedByteBuffer) -> zircon_runtime_interface::ZrStatus,
    operation: &'static str,
) -> Result<T, RuntimeLibraryError> {
    let mut output = ZrOwnedByteBuffer::empty();
    ensure_status(call(&mut output), operation)?;
    let decoded = if output.is_empty() {
        Err(RuntimeLibraryError::new(format!(
            "{operation} returned an empty payload"
        )))
    } else {
        let bytes = unsafe { slice::from_raw_parts(output.data.cast_const(), output.len) };
        serde_json::from_slice(bytes).map_err(|error| RuntimeLibraryError::new(error.to_string()))
    };
    if let Some(free) = output.free {
        ensure_status(unsafe { free(output) }, "free runtime operation output")?;
    }
    decoded
}
