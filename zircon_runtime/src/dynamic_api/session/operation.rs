use std::ptr;

use zircon_runtime_interface::{
    ZrByteSlice, ZrOwnedResultV2, ZrRuntimeOperationHandle, ZrRuntimeOperationStatusV2,
    ZrRuntimeSessionHandle, ZrStatus, ZrStatusCode,
};

use crate::operation::RuntimeOperationServiceError;

use super::registry::{
    register_runtime_allocation_in_action, with_session, with_session_result_finalized,
    RuntimeAllocationKind,
};
use super::status::{error_status, invalid_argument, not_found, unsupported_version};

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
        if request_json.len > runtime.operations.max_retained_bytes() {
            return invalid_argument(b"runtime operation request exceeds retained byte capacity");
        }
        let request_json = unsafe { request_json.as_slice() };
        match runtime.operations.submit_json(request_json) {
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
    match with_session_result_finalized(
        session,
        |runtime| encode_harvest_json_result(runtime.operations.harvest(handle)),
        |active_session, bytes| {
            let output = register_runtime_allocation_in_action(
                active_session,
                RuntimeAllocationKind::Operation,
                bytes,
            )?;
            unsafe { ptr::write(out_result, output) };
            Ok(ZrStatus::ok())
        },
    ) {
        Ok(status) | Err(status) => status,
    }
}

fn encode_harvest_json_result<T: serde::Serialize>(
    result: Result<T, RuntimeOperationServiceError>,
) -> Result<Vec<u8>, ZrStatus> {
    let value = match result {
        Ok(value) => value,
        Err(error) => return Err(operation_error_status(error)),
    };
    serde_json::to_vec(&value).map_err(error_status)
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
