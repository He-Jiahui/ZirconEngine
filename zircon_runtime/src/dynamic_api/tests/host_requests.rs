use super::support::*;

#[test]
fn drain_host_requests_requires_output_pointer() {
    let api = runtime_api();
    let drain_host_requests = api.drain_host_requests.expect("drain_host_requests");

    let status = unsafe {
        drain_host_requests(
            ZrRuntimeSessionHandle::new(99_999),
            core::ptr::null_mut::<ZrOwnedByteBuffer>(),
        )
    };

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
fn host_request_batch_encodes_runtime_ime_requests() {
    let batch = ZrRuntimeHostRequestBatchV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        vec![
            ZrRuntimeHostRequestV1::ime(runtime_ime_host_request(ImeHostRequest::Enable)),
            ZrRuntimeHostRequestV1::ime(runtime_ime_host_request(ImeHostRequest::SetCursorArea(
                ImeCursorArea::new(16.0, 24.0, 8.0, 18.0),
            ))),
            ZrRuntimeHostRequestV1::ime(runtime_ime_host_request(
                ImeHostRequest::SetSurroundingText(ImeSurroundingText::new("search", 6, 0)),
            )),
        ],
    );

    let output = encode_host_request_batch(&batch).unwrap();
    let batch = host_request_batch_from_output(output);

    assert_eq!(batch.abi_version, ZIRCON_RUNTIME_ABI_VERSION_V1);
    assert_eq!(batch.requests.len(), 3);
    assert!(matches!(
        batch.requests[0],
        ZrRuntimeHostRequestV1::Ime(ref request)
            if request.kind == ZrRuntimeImeHostRequestKindV1::Enable
    ));
    assert!(matches!(
        batch.requests[1],
        ZrRuntimeHostRequestV1::Ime(ref request)
            if request.kind == ZrRuntimeImeHostRequestKindV1::SetCursorArea
                && request.cursor_area.as_ref().map(|area| area.width) == Some(8.0)
    ));
    assert!(matches!(
        batch.requests[2],
        ZrRuntimeHostRequestV1::Ime(ref request)
            if request.kind == ZrRuntimeImeHostRequestKindV1::SetSurroundingText
                && request
                    .surrounding_text
                    .as_ref()
                    .map(|text| text.value.as_str())
                    == Some("search")
    ));
}

#[test]
fn host_request_batch_encodes_gamepad_rumble_requests() {
    let batch = ZrRuntimeHostRequestBatchV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        vec![
            ZrRuntimeHostRequestV1::gamepad_rumble(runtime_gamepad_rumble_request(
                GamepadRumbleRequest::add(
                    GamepadId(7),
                    GamepadRumbleIntensity::new(1.25, -0.5),
                    250,
                ),
            )),
            ZrRuntimeHostRequestV1::gamepad_rumble(runtime_gamepad_rumble_request(
                GamepadRumbleRequest::stop(GamepadId(7)),
            )),
        ],
    );

    let output = encode_host_request_batch(&batch).unwrap();
    let batch = host_request_batch_from_output(output);

    assert_eq!(batch.abi_version, ZIRCON_RUNTIME_ABI_VERSION_V1);
    assert_eq!(batch.requests.len(), 2);
    assert!(matches!(
        batch.requests[0],
        ZrRuntimeHostRequestV1::GamepadRumble(ZrRuntimeGamepadRumbleRequestV1 {
            gamepad_id: 7,
            kind: ZrRuntimeGamepadRumbleRequestKindV1::Add,
            strong_motor: 1.0,
            weak_motor: 0.0,
            duration_millis: 250,
        })
    ));
    assert!(matches!(
        batch.requests[1],
        ZrRuntimeHostRequestV1::GamepadRumble(ZrRuntimeGamepadRumbleRequestV1 {
            gamepad_id: 7,
            kind: ZrRuntimeGamepadRumbleRequestKindV1::Stop,
            strong_motor: 0.0,
            weak_motor: 0.0,
            duration_millis: 0,
        })
    ));
}

#[test]
fn host_request_batch_encodes_cursor_requests() {
    let batch = ZrRuntimeHostRequestBatchV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        vec![
            ZrRuntimeHostRequestV1::cursor(runtime_cursor_host_request(
                CursorHostRequest::set_visible(false),
            )),
            ZrRuntimeHostRequestV1::cursor(runtime_cursor_host_request(
                CursorHostRequest::set_grab_mode(CursorGrabMode::Locked),
            )),
            ZrRuntimeHostRequestV1::cursor(runtime_cursor_host_request(
                CursorHostRequest::set_hit_test(false),
            )),
            ZrRuntimeHostRequestV1::cursor(runtime_cursor_host_request(
                CursorHostRequest::set_position(320.0, 180.0),
            )),
        ],
    );

    let output = encode_host_request_batch(&batch).unwrap();
    let batch = host_request_batch_from_output(output);

    assert_eq!(batch.abi_version, ZIRCON_RUNTIME_ABI_VERSION_V1);
    assert_eq!(batch.requests.len(), 4);
    assert!(matches!(
        batch.requests[0],
        ZrRuntimeHostRequestV1::Cursor(ZrRuntimeCursorHostRequestV1 {
            kind: ZrRuntimeCursorHostRequestKindV1::SetVisible,
            value: false,
            ..
        })
    ));
    assert!(matches!(
        batch.requests[1],
        ZrRuntimeHostRequestV1::Cursor(ZrRuntimeCursorHostRequestV1 {
            kind: ZrRuntimeCursorHostRequestKindV1::SetGrabMode,
            grab_mode: Some(ZrRuntimeCursorGrabModeV1::Locked),
            ..
        })
    ));
    assert!(matches!(
        batch.requests[2],
        ZrRuntimeHostRequestV1::Cursor(ZrRuntimeCursorHostRequestV1 {
            kind: ZrRuntimeCursorHostRequestKindV1::SetHitTest,
            value: false,
            ..
        })
    ));
    assert!(matches!(
        batch.requests[3],
        ZrRuntimeHostRequestV1::Cursor(ZrRuntimeCursorHostRequestV1 {
            kind: ZrRuntimeCursorHostRequestKindV1::SetPosition,
            position: Some(position),
            ..
        }) if position.x == 320.0 && position.y == 180.0
    ));
}

#[test]
fn host_request_free_rejects_wrong_owner_token() {
    let mut bytes = vec![1_u8, 2, 3];
    let buffer = ZrOwnedByteBuffer {
        data: bytes.as_mut_ptr(),
        len: bytes.len(),
        capacity: bytes.capacity(),
        owner_token: 0,
        free: Some(free_runtime_host_request_bytes),
    };

    let status = unsafe { free_runtime_host_request_bytes(buffer) };

    assert_eq!(status.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(
        status_message(status),
        "invalid runtime host request buffer"
    );
}
