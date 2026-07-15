use std::ptr;

use zircon_runtime_interface::{
    ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimeOperationHandle, ZrRuntimeOperationSubmitRequestV1,
    ZrRuntimeSessionHandle, ZrStatus, ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use crate::operation::{RuntimeOperationContext, RuntimeOperationServiceError};

use super::registry::with_session;
use super::status::{error_status, invalid_argument, not_found, unsupported_version};

const RUNTIME_OPERATION_BUFFER_OWNER_TOKEN: u64 = 0x5a52_4f50_4552_0001;

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
    let request = match serde_json::from_slice::<ZrRuntimeOperationSubmitRequestV1>(unsafe {
        request_json.as_slice()
    }) {
        Ok(request) => request,
        Err(_) => return invalid_argument(b"invalid runtime operation request"),
    };
    if request.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return unsupported_version();
    }
    with_session(session, |runtime| {
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
    out_progress: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    if out_progress.is_null() {
        return invalid_argument(b"missing runtime operation progress output");
    }
    if !handle.is_valid() {
        return invalid_argument(b"invalid runtime operation handle");
    }
    with_session(session, |runtime| {
        let core = runtime.runtime.handle();
        let progress = runtime.level.with_world_mut(|world| {
            runtime
                .operations
                .poll(RuntimeOperationContext::new(&core, world), handle)
        });
        write_json_result(out_progress, progress)
    })
}

pub(crate) unsafe fn harvest_operation(
    session: ZrRuntimeSessionHandle,
    handle: ZrRuntimeOperationHandle,
    out_result: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    if out_result.is_null() {
        return invalid_argument(b"missing runtime operation result output");
    }
    if !handle.is_valid() {
        return invalid_argument(b"invalid runtime operation handle");
    }
    with_session(session, |runtime| {
        write_json_result(out_result, runtime.operations.harvest(handle))
    })
}

fn write_json_result<T: serde::Serialize>(
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
        RuntimeOperationServiceError::EmptyOperationId => {
            invalid_argument(b"runtime operation id cannot be empty")
        }
        RuntimeOperationServiceError::UnknownOperation { .. }
        | RuntimeOperationServiceError::UnknownHandle { .. } => {
            not_found(b"runtime operation not found")
        }
        other => error_status(other),
    }
}
