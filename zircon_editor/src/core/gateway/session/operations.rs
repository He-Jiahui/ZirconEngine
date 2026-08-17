use std::mem::MaybeUninit;

use zircon_runtime_host::foreign_output::{
    operation_result_item_count, RuntimeForeignOutputKind, OPERATION_RESULT_OUTPUT_BUDGET,
};
use zircon_runtime_interface::{
    ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimeOperationHandle, ZrRuntimeOperationResultV1,
    ZrRuntimeOperationStatusV2, ZrRuntimeOperationSubmitRequestV1,
};

use super::super::GatewayError;
use super::gateway::SessionGateway;
use super::protocol::{
    ensure_operation_handle, ensure_operation_status, ensure_output_abi, ensure_status,
};

impl SessionGateway {
    pub(super) fn submit_operation(
        &self,
        request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
        self.ensure_output_available(RuntimeForeignOutputKind::OperationResult)?;
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
            return self.reject_protocol(
                RuntimeForeignOutputKind::OperationResult,
                GatewayError::Protocol {
                    message: "runtime returned an invalid operation handle".to_string(),
                },
            );
        }
        Ok(handle)
    }

    pub(super) fn poll_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationStatusV2, GatewayError> {
        self.ensure_output_available(RuntimeForeignOutputKind::OperationResult)?;
        let poll = Self::required(self.api.poll_operation, "runtime.operation.poll")?;
        let mut status = MaybeUninit::<ZrRuntimeOperationStatusV2>::uninit();
        ensure_status(
            unsafe { poll(self.session, handle, status.as_mut_ptr()) },
            "poll runtime operation",
        )?;
        let status = unsafe { status.assume_init() };
        if let Err(error) = ensure_operation_status(&status, handle) {
            return self.reject_protocol(RuntimeForeignOutputKind::OperationResult, error);
        }
        Ok(status)
    }

    pub(super) fn harvest_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        self.ensure_output_available(RuntimeForeignOutputKind::OperationResult)?;
        let harvest = Self::required(self.api.harvest_operation, "runtime.operation.harvest")?;
        let mut output = ZrOwnedByteBuffer::empty();
        let status = unsafe { harvest(self.session, handle, &mut output) };
        let result = self.decode_output(
            status,
            output,
            RuntimeForeignOutputKind::OperationResult,
            OPERATION_RESULT_OUTPUT_BUDGET,
            "harvest runtime operation",
            "free runtime operation output",
            |result: &ZrRuntimeOperationResultV1| {
                ensure_output_abi(result.abi_version, "runtime operation result")?;
                ensure_operation_handle(result.handle, handle, "runtime operation result")?;
                Ok::<usize, GatewayError>(operation_result_item_count(result))
            },
        )?;
        result.ok_or_else(|| GatewayError::Protocol {
            message: "harvest runtime operation returned an empty payload".to_owned(),
        })
    }
}
