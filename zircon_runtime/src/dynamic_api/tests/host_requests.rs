use super::support::*;

#[test]
fn drain_host_requests_requires_output_pointer() {
    let api = runtime_api();
    let drain_host_requests = api.drain_host_requests.expect("drain_host_requests");
    let session = create_test_session(api);

    let status =
        unsafe { drain_host_requests(session, core::ptr::null_mut::<ZrOwnedByteBuffer>()) };

    destroy_test_session(api, session);
    assert_eq!(status.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(status_message(status), "missing host request output");
}

#[test]
fn drain_host_requests_rejects_unknown_session() {
    let api = runtime_api();
    let drain_host_requests = api.drain_host_requests.expect("drain_host_requests");
    let mut output = ZrOwnedByteBuffer::empty();

    let status = unsafe { drain_host_requests(ZrRuntimeSessionHandle::new(99_999), &mut output) };

    assert_eq!(status.status_code(), ZrStatusCode::NotFound);
    assert_eq!(status_message(status), "runtime session not found");
    assert!(output.is_empty());
}

#[test]
fn dynamic_session_drains_runtime_ime_cursor_area_and_surrounding_text_requests_once() {
    let api = runtime_api();
    let handle_event = api.handle_event.expect("handle_event");
    let drain_host_requests = api.drain_host_requests.expect("drain_host_requests");
    let session = create_test_session(api);

    assert_session_status(
        unsafe {
            handle_event(
                session,
                ZrRuntimeEventV1::ime_cursor_area(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    default_viewport(),
                    12.0,
                    34.0,
                    2,
                    18,
                ),
            )
        },
        ZrStatusCode::Ok,
        "",
    );
    assert_session_status(
        unsafe {
            handle_event(
                session,
                ZrRuntimeEventV1::ime_surrounding_text(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    default_viewport(),
                    ZrByteSlice::from_static(b"abcdef"),
                    5,
                    1,
                ),
            )
        },
        ZrStatusCode::Ok,
        "",
    );

    let mut output = ZrOwnedByteBuffer::empty();
    assert_session_status(
        unsafe { drain_host_requests(session, &mut output) },
        ZrStatusCode::Ok,
        "",
    );
    let batch = host_request_batch_from_output(output);
    assert_eq!(batch.abi_version, ZIRCON_RUNTIME_ABI_VERSION_V1);
    assert_eq!(batch.requests.len(), 2);
    assert!(matches!(
        batch.requests[0],
        ZrRuntimeHostRequestV1::Ime(ref request)
            if request.kind == ZrRuntimeImeHostRequestKindV1::SetCursorArea
                && request.cursor_area == Some(ZrRuntimeImeCursorAreaV1::new(12.0, 34.0, 2.0, 18.0))
                && request.target_viewport == Some(default_viewport())
    ));
    assert!(matches!(
        batch.requests[1],
        ZrRuntimeHostRequestV1::Ime(ref request)
            if request.kind == ZrRuntimeImeHostRequestKindV1::SetSurroundingText
                && request
                    .surrounding_text
                    .as_ref()
                    .map(|text| (text.value.as_str(), text.cursor, text.anchor))
                    == Some(("abcdef", 5, 1))
                && request.target_viewport == Some(default_viewport())
    ));

    let mut second_output = ZrOwnedByteBuffer::empty();
    assert_session_status(
        unsafe { drain_host_requests(session, &mut second_output) },
        ZrStatusCode::Ok,
        "",
    );
    assert!(
        second_output.is_empty(),
        "an empty host-request drain must not allocate or serialize an ABI payload"
    );

    destroy_test_session(api, session);
}
