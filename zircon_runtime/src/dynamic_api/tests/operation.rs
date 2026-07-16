use super::support::*;
use zircon_runtime_interface::{
    ZrRuntimeOperationHandle, ZrRuntimeOperationPhase, ZrRuntimeOperationProgressV1,
    ZrRuntimeOperationResultV1, ZrRuntimeOperationSubmitRequestV1,
};

#[test]
fn dynamic_api_submits_polls_and_harvests_runtime_operation() {
    let api = runtime_api();
    let session = create_test_session(api);
    let submit = api.submit_operation.expect("submit_operation");
    let poll = api.poll_operation.expect("poll_operation");
    let harvest = api.harvest_operation.expect("harvest_operation");
    let request = serde_json::to_vec(&ZrRuntimeOperationSubmitRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        zircon_runtime::core::framework::navigation::NAVIGATION_CLEAR_SURFACE_OPERATION,
        serde_json::json!({"surface_entity": 42}),
    ))
    .unwrap();
    let mut handle = ZrRuntimeOperationHandle::invalid();

    let status = unsafe {
        submit(
            session,
            ZrByteSlice {
                data: request.as_ptr(),
                len: request.len(),
            },
            &mut handle,
        )
    };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
    assert!(handle.is_valid());

    let running: ZrRuntimeOperationProgressV1 =
        decode_operation_output(call_operation_output(poll, session, handle));
    assert_eq!(running.phase, ZrRuntimeOperationPhase::Running);
    let completed: ZrRuntimeOperationProgressV1 =
        decode_operation_output(call_operation_output(poll, session, handle));
    assert_eq!(completed.phase, ZrRuntimeOperationPhase::Completed);
    let result: ZrRuntimeOperationResultV1 =
        decode_operation_output(call_operation_output(harvest, session, handle));
    assert!(result.succeeded_output().is_some());

    destroy_test_session(api, session);
}

fn call_operation_output(
    call: unsafe extern "C" fn(
        ZrRuntimeSessionHandle,
        ZrRuntimeOperationHandle,
        *mut ZrOwnedByteBuffer,
    ) -> ZrStatus,
    session: ZrRuntimeSessionHandle,
    handle: ZrRuntimeOperationHandle,
) -> ZrOwnedByteBuffer {
    let mut output = ZrOwnedByteBuffer::empty();
    let status = unsafe { call(session, handle, &mut output) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
    output
}

fn decode_operation_output<T: serde::de::DeserializeOwned>(output: ZrOwnedByteBuffer) -> T {
    let bytes = unsafe { core::slice::from_raw_parts(output.data.cast_const(), output.len) };
    let decoded = serde_json::from_slice(bytes).unwrap();
    free_output(output);
    decoded
}
