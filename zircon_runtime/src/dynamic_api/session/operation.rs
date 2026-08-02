use std::ptr;

use zircon_runtime_interface::{
    ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimeOperationHandle, ZrRuntimeOperationStatusV2,
    ZrRuntimeSessionHandle, ZrStatus, ZrStatusCode,
};

use crate::operation::RuntimeOperationServiceError;

use super::registry::with_session;
use super::status::{error_status, invalid_argument, not_found, unsupported_version};

const RUNTIME_OPERATION_BUFFER_OWNER_TOKEN: u64 = 0x5a52_4f50_4552_0001;

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
    out_result: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    with_session(session, |runtime| {
        if out_result.is_null() {
            return invalid_argument(b"missing runtime operation result output");
        }
        if !handle.is_valid() {
            return invalid_argument(b"invalid runtime operation handle");
        }
        write_harvest_json_result(out_result, runtime.operations.harvest(handle))
    })
}

fn write_harvest_json_result<T: serde::Serialize>(
    destination: *mut ZrOwnedByteBuffer,
    result: Result<T, RuntimeOperationServiceError>,
) -> ZrStatus {
    let value = match result {
        Ok(value) => value,
        Err(error) => return operation_error_status(error),
    };
    let bytes = match serde_json::to_vec(&value) {
        Ok(bytes) => bytes,
        Err(error) => return error_status(error),
    };
    unsafe { ptr::write(destination, owned_operation_buffer(bytes)) };
    ZrStatus::ok()
}

fn owned_operation_buffer(mut bytes: Vec<u8>) -> ZrOwnedByteBuffer {
    if bytes.is_empty() {
        return ZrOwnedByteBuffer::empty();
    }
    let buffer = ZrOwnedByteBuffer {
        data: bytes.as_mut_ptr(),
        len: bytes.len(),
        capacity: bytes.capacity(),
        owner_token: RUNTIME_OPERATION_BUFFER_OWNER_TOKEN,
        free: Some(free_runtime_operation_bytes),
    };
    std::mem::forget(bytes);
    buffer
}

unsafe extern "C" fn free_runtime_operation_bytes(buffer: ZrOwnedByteBuffer) -> ZrStatus {
    if buffer.is_empty() {
        return ZrStatus::ok();
    }
    if buffer.owner_token != RUNTIME_OPERATION_BUFFER_OWNER_TOKEN || buffer.data.is_null() {
        return invalid_argument(b"invalid runtime operation buffer");
    }
    if buffer.len > buffer.capacity {
        return invalid_argument(b"invalid runtime operation buffer");
    }
    let _ = unsafe { Vec::from_raw_parts(buffer.data, buffer.len, buffer.capacity) };
    ZrStatus::ok()
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
