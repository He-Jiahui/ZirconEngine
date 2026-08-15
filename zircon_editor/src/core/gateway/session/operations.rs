use std::mem::MaybeUninit;

use zircon_runtime_interface::{
    ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimeOperationHandle, ZrRuntimeOperationResultV1,
    ZrRuntimeOperationStatusV2, ZrRuntimeOperationSubmitRequestV1,
};

use super::super::GatewayError;
use super::gateway::SessionGateway;
use super::output::{decode_owned_output, validate_output_status};
use super::protocol::{
    ensure_operation_handle, ensure_operation_status, ensure_output_abi, ensure_status,
};

impl SessionGateway {
    pub(super) fn submit_operation(
        &self,
        request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
        let submit = Self::required(self.api.submit_operation, "runtime.operation.submit")?;
        let request = serde_json::to_vec(&request).map_err(|error| GatewayError::Protocol {
            message: format!("encode runtime operation request: {error}"),
        })?;
        let mut handle = ZrRuntimeOperationHandle::invalid();
        ensure_status(
            unsafe {
                submit(
                    self.session,
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
            return Err(GatewayError::Protocol {
                message: "runtime returned an invalid operation handle".to_string(),
            });
        }
        Ok(handle)
    }

    pub(super) fn poll_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationStatusV2, GatewayError> {
        let poll = Self::required(self.api.poll_operation, "runtime.operation.poll")?;
        let mut status = MaybeUninit::<ZrRuntimeOperationStatusV2>::uninit();
        ensure_status(
            unsafe { poll(self.session, handle, status.as_mut_ptr()) },
            "poll runtime operation",
        )?;
        let status = unsafe { status.assume_init() };
        ensure_operation_status(&status, handle)?;
        Ok(status)
    }

    pub(super) fn harvest_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        let harvest = Self::required(self.api.harvest_operation, "runtime.operation.harvest")?;
        let mut output = ZrOwnedByteBuffer::empty();
        let status = unsafe { harvest(self.session, handle, &mut output) };
        let output = validate_output_status(status, output, "harvest runtime operation")?;
        let result: ZrRuntimeOperationResultV1 =
            decode_owned_output(output, "harvest runtime operation")?;
        ensure_output_abi(result.abi_version, "runtime operation result")?;
        ensure_operation_handle(result.handle, handle, "runtime operation result")?;
        Ok(result)
    }
}
