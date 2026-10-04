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
    register_runtime_allocation_in_action, with_session_result_committed,
    with_session_result_finalized, RuntimeAllocationKind,
};
use super::status::{
    error_status, invalid_argument, invalid_or_limit_payload, limit_exceeded, not_found,
    output_payload_status, unsupported_version,
};

pub(crate) unsafe fn submit_operation(
    session: ZrRuntimeSessionHandle,
    request_json: ZrByteSlice,
    out_handle: *mut ZrRuntimeOperationHandle,
) -> ZrStatus {
    if out_handle.is_null() {
        return invalid_argument(b"missing runtime operation handle output");
    }
    if request_json.is_empty() {
        return invalid_argument(b"missing runtime operation request");
    }
    let request_bytes = match unsafe {
        request_json.checked_slice(ZR_RUNTIME_OPERATION_REQUEST_LIMIT_V1.max_encoded_bytes)
    } {
        Ok(bytes) => bytes.to_vec(),
        Err(error) if error.is_limit_exceeded() => {
            return limit_exceeded(b"runtime operation request exceeds limit");
        }
        Err(_) => return invalid_argument(b"invalid runtime operation request"),
    };
    let request_len = request_bytes.len();
    match with_session_result_finalized(
        session,
        move |runtime| {
            let mut limit = ZR_RUNTIME_OPERATION_REQUEST_LIMIT_V1;
            limit.max_encoded_bytes = limit
                .max_encoded_bytes
                .min(runtime.operations.max_retained_bytes());
            let request_slice = ZrByteSlice {
                data: request_bytes.as_ptr(),
                len: request_bytes.len(),
            };
            match runtime
                .operations
                .submit_with_raw_admission(request_len, || unsafe {
                    bounded_json::decode::<ZrRuntimeOperationSubmitRequestV1>(
                        request_slice,
                        limit,
                        |request| {
                            bounded_json::json_value_item_count(&request.payload).saturating_add(2)
                        },
                    )
                }) {
                Err(error) => Err(invalid_or_limit_payload(
                    &error,
                    b"invalid runtime operation request",
                    b"runtime operation request exceeds limit",
                )),
                Ok(Err(RuntimeOperationServiceError::RetainedBytesCapacityReached { maximum }))
                    if request_len > maximum =>
                {
                    Err(limit_exceeded(b"runtime operation request exceeds limit"))
                }
                Ok(Err(error)) => Err(operation_error_status(error)),
                Ok(Ok(handle)) => Ok(handle),
            }
        },
        |_session, handle| {
            unsafe { ptr::write(out_handle, handle) };
            Ok(ZrStatus::ok())
        },
    ) {
        Ok(status) | Err(status) => status,
    }
}

pub(crate) unsafe fn poll_operation(
    session: ZrRuntimeSessionHandle,
    handle: ZrRuntimeOperationHandle,
    out_status: *mut ZrRuntimeOperationStatusV2,
) -> ZrStatus {
    if out_status.is_null() {
        return invalid_argument(b"missing runtime operation status output");
    }
    if !handle.is_valid() {
        return invalid_argument(b"invalid runtime operation handle");
    }
    match with_session_result_finalized(
        session,
        |runtime| match runtime.operations.poll(handle) {
            Ok(status) => Ok(status),
            Err(error) => Err(operation_error_status(error)),
        },
        |_session, status| {
            unsafe { ptr::write(out_status, status) };
            Ok(ZrStatus::ok())
        },
    ) {
        Ok(status) | Err(status) => status,
    }
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
