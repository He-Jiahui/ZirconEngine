use super::support::*;
use zircon_runtime_interface::{
    ZrRuntimeFrameDemandV1, ZrRuntimeOperationDetailKindV2, ZrRuntimeOperationHandle,
    ZrRuntimeOperationPhase, ZrRuntimeOperationResultV1, ZrRuntimeOperationStatusV2,
    ZrRuntimeOperationSubmitRequestV1,
};

#[test]
fn dynamic_api_submits_polls_and_harvests_runtime_operation() {
    let api = runtime_api();
    let session = create_test_session(api);
    let submit = api.submit_operation.expect("submit_operation");
    let poll = api.poll_operation.expect("poll_operation");
    let harvest = api.harvest_operation.expect("harvest_operation");
    let tick_frame = api.tick_frame.expect("tick_frame");
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

    let queued = call_operation_status(poll, session, handle);
    assert_eq!(queued.phase(), Some(ZrRuntimeOperationPhase::Queued));

    let mut demand = ZrRuntimeFrameDemandV1::idle();
    let completed = (0..256)
        .find_map(|_| {
            let status = unsafe { tick_frame(session, &mut demand) };
            assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
            let operation_status = call_operation_status(poll, session, handle);
            if operation_status
                .phase()
                .is_some_and(|phase| phase.is_terminal())
            {
                Some(operation_status)
            } else {
                std::thread::yield_now();
                None
            }
        })
        .expect("runtime operation must complete after bounded owner ticks");
    assert_eq!(completed.phase(), Some(ZrRuntimeOperationPhase::Completed));
    let result: ZrRuntimeOperationResultV1 =
        decode_operation_output(session, call_operation_output(harvest, session, handle));
    assert!(result.succeeded_output().is_some());

    destroy_test_session(api, session);
}

#[test]
fn dynamic_api_poll_leaves_status_output_untouched_for_invalid_or_unknown_handles() {
    let api = runtime_api();
    let session = create_test_session(api);
    let poll = api.poll_operation.expect("poll_operation");
    let sentinel = ZrRuntimeOperationStatusV2::new(
        ZrRuntimeOperationHandle::new(17),
        ZrRuntimeOperationPhase::Queued,
        0,
        1,
        ZrRuntimeOperationDetailKindV2::None,
        0,
    );
    let mut output = sentinel;

    let invalid = unsafe { poll(session, ZrRuntimeOperationHandle::invalid(), &mut output) };
    assert_eq!(invalid.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(output, sentinel);

    let unknown = unsafe { poll(session, ZrRuntimeOperationHandle::new(99), &mut output) };
    assert_eq!(unknown.status_code(), ZrStatusCode::NotFound);
    assert_eq!(output, sentinel);

    let null = unsafe {
        poll(
            session,
            ZrRuntimeOperationHandle::new(99),
            core::ptr::null_mut(),
        )
    };
    assert_eq!(null.status_code(), ZrStatusCode::InvalidArgument);

    destroy_test_session(api, session);
}

#[test]
fn dynamic_api_operation_poll_is_fixed_layout_and_allocation_free() {
    let source = include_str!("../session/operation.rs");
    let start = source
        .find("pub(crate) unsafe fn poll_operation(")
        .expect("operation poll owner");
    let end = source[start..]
        .find("pub(crate) unsafe fn harvest_operation(")
        .map(|offset| start + offset)
        .expect("operation harvest boundary");
    let poll_source = &source[start..end];

    assert!(poll_source.contains("*mut ZrRuntimeOperationStatusV2"));
    assert!(poll_source.contains("ptr::write(out_status, status)"));
    assert!(!poll_source.contains("ZrOwnedByteBuffer"));
    assert!(!poll_source.contains("serde_json"));
}

fn call_operation_status(
    call: unsafe extern "C" fn(
        ZrRuntimeSessionHandle,
        ZrRuntimeOperationHandle,
        *mut ZrRuntimeOperationStatusV2,
    ) -> ZrStatus,
    session: ZrRuntimeSessionHandle,
    handle: ZrRuntimeOperationHandle,
) -> ZrRuntimeOperationStatusV2 {
    let mut status = core::mem::MaybeUninit::<ZrRuntimeOperationStatusV2>::uninit();
    let result = unsafe { call(session, handle, status.as_mut_ptr()) };
    assert_eq!(result.status_code(), ZrStatusCode::Ok, "{result:?}");
    unsafe { status.assume_init() }
}

fn call_operation_output(
    call: unsafe extern "C" fn(
        ZrRuntimeSessionHandle,
        ZrRuntimeOperationHandle,
        *mut ZrOwnedResultV2,
    ) -> ZrStatus,
    session: ZrRuntimeSessionHandle,
    handle: ZrRuntimeOperationHandle,
) -> ZrOwnedResultV2 {
    let mut output = ZrOwnedResultV2::empty();
    let status = unsafe { call(session, handle, &mut output) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
    output
}

fn decode_operation_output<T: serde::de::DeserializeOwned>(
    session: ZrRuntimeSessionHandle,
    output: ZrOwnedResultV2,
) -> T {
    let bytes = output_bytes(&output);
    let decoded = serde_json::from_slice(bytes).unwrap();
    release_output(session, output);
    decoded
}
