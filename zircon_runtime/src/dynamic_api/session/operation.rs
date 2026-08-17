use std::ptr;

use zircon_runtime_interface::{
    ZrByteSlice, ZrOwnedResultV2, ZrRuntimeOperationHandle, ZrRuntimeOperationResultV1,
    ZrRuntimeOperationStatusV2, ZrRuntimeOperationSubmitRequestV1, ZrRuntimeSessionHandle,
    ZrStatus, ZrStatusCode, ZR_RUNTIME_OPERATION_REQUEST_LIMIT_V1,
    ZR_RUNTIME_OPERATION_RESULT_OUTPUT_LIMIT_V1,
};

use crate::operation::RuntimeOperationServiceError;

use super::super::bounded_json;
use super::registry::{
    register_runtime_allocation_in_action, with_session, with_session_result_committed,
    RuntimeAllocationKind,
};
use super::status::{
    error_status, invalid_argument, invalid_or_limit_payload, not_found, output_payload_status,
    unsupported_version,
};

pub(crate) unsafe fn submit_operation(
    session: ZrRuntimeSessionHandle,
    request_json: ZrByteSlice,
    out_handle: *mut ZrRuntimeOperationHandle,
) -> ZrStatus {
    with_session(session, |runtime| {
        if out_handle.is_null() {
            return invalid_argument(b"missing runtime operation handle output");
        }
        if request_json.is_empty() {
            return invalid_argument(b"missing runtime operation request");
        }
        let mut limit = ZR_RUNTIME_OPERATION_REQUEST_LIMIT_V1;
        limit.max_encoded_bytes = limit
            .max_encoded_bytes
            .min(runtime.operations.max_retained_bytes());
        let request = match unsafe {
            bounded_json::decode::<ZrRuntimeOperationSubmitRequestV1>(
                request_json,
                limit,
                |request| bounded_json::json_value_item_count(&request.payload).saturating_add(2),
            )
        } {
            Ok(request) => request,
            Err(error) => {
                return invalid_or_limit_payload(
                    &error,
                    b"invalid runtime operation request",
                    b"runtime operation request exceeds limit",
                );
            }
        };
        match runtime.operations.submit(request) {
            Ok(handle) => {
                unsafe { ptr::write(out_handle, handle) };
                ZrStatus::ok()
            }
            Err(error) => operation_error_status(error),
        }
    })
}

pub(crate) unsafe fn poll_operation(
    session: ZrRuntimeSessionHandle,
    handle: ZrRuntimeOperationHandle,
    out_status: *mut ZrRuntimeOperationStatusV2,
) -> ZrStatus {
    with_session(session, |runtime| {
        if out_status.is_null() {
            return invalid_argument(b"missing runtime operation status output");
        }
        if !handle.is_valid() {
            return invalid_argument(b"invalid runtime operation handle");
        }
        match runtime.operations.poll(handle) {
            Ok(status) => {
                unsafe { ptr::write(out_status, status) };
                ZrStatus::ok()
            }
            Err(error) => operation_error_status(error),
        }
    })
}

pub(crate) unsafe fn harvest_operation(
    session: ZrRuntimeSessionHandle,
    handle: ZrRuntimeOperationHandle,
    out_result: *mut ZrOwnedResultV2,
) -> ZrStatus {
    if out_result.is_null() {
        return invalid_argument(b"missing runtime operation result output");
    }
    if !handle.is_valid() {
        return invalid_argument(b"invalid runtime operation handle");
    }
    match with_session_result_committed(
        session,
        |runtime| {
            runtime
                .operations
                .prepare_harvest(handle, encode_harvest_json_result)
                .map_err(operation_error_status)?
        },
        |active_session, bytes| {
            let output = register_runtime_allocation_in_action(
                active_session,
                RuntimeAllocationKind::Operation,
                bytes,
            )?;
            unsafe { ptr::write(out_result, output) };
            Ok(ZrStatus::ok())
        },
        |runtime| {
            let _ = runtime.operations.commit_harvest(handle);
        },
        |runtime| runtime.operations.rollback_harvest(handle),
    ) {
        Ok(status) | Err(status) => status,
    }
}

fn encode_harvest_json_result(value: &ZrRuntimeOperationResultV1) -> Result<Vec<u8>, ZrStatus> {
    bounded_json::encode(value, ZR_RUNTIME_OPERATION_RESULT_OUTPUT_LIMIT_V1, || {
        value
            .succeeded_output()
            .map(bounded_json::json_value_item_count)
            .unwrap_or(1)
            .saturating_add(1)
    })
    .map_err(|error| output_payload_status(error, b"runtime operation result exceeds limit"))
}

fn operation_error_status(error: RuntimeOperationServiceError) -> ZrStatus {
    match error {
        RuntimeOperationServiceError::UnsupportedAbiVersion { .. } => unsupported_version(),
        RuntimeOperationServiceError::InvalidRequest => {
            invalid_argument(b"invalid runtime operation request")
        }
        RuntimeOperationServiceError::EmptyOperationId => {
            invalid_argument(b"runtime operation id cannot be empty")
        }
        RuntimeOperationServiceError::UnknownOperation { .. }
        | RuntimeOperationServiceError::UnknownHandle { .. } => {
            not_found(b"runtime operation not found")
        }
        RuntimeOperationServiceError::OperationCancelled { .. } => ZrStatus::new(
            ZrStatusCode::Error,
            ZrByteSlice::from_static(b"operation cancelled"),
        ),
        RuntimeOperationServiceError::OperationExpired { .. } => ZrStatus::new(
            ZrStatusCode::Error,
            ZrByteSlice::from_static(b"operation result expired"),
        ),
        other => error_status(other),
    }
}
