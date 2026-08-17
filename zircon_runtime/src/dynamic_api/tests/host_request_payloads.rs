use super::support::*;

#[test]
fn host_request_batch_encodes_runtime_ime_requests() {
    let viewport = default_viewport();
    let batch = ZrRuntimeHostRequestBatchV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        vec![
            ZrRuntimeHostRequestV1::ime(runtime_ime_host_request(ImeHostRequest::Enable, viewport)),
            ZrRuntimeHostRequestV1::ime(runtime_ime_host_request(
                ImeHostRequest::SetCursorArea(ImeCursorArea::new(16.0, 24.0, 8.0, 18.0)),
                viewport,
            )),
            ZrRuntimeHostRequestV1::ime(runtime_ime_host_request(
                ImeHostRequest::SetSurroundingText(
                    ImeSurroundingText::new("search", 6, 0)
                        .with_composition_range(Some(ImeCursorRange::new(1, 6))),
                ),
                viewport,
            )),
        ],
    );

    let output = encode_host_request_batch(&batch).unwrap();
    let batch = host_request_batch_from_bytes(&output);

    assert_eq!(batch.abi_version, ZIRCON_RUNTIME_ABI_VERSION_V1);
    assert_eq!(batch.requests.len(), 3);
    assert!(matches!(
        batch.requests[0],
        ZrRuntimeHostRequestV1::Ime(ref request)
            if request.kind == ZrRuntimeImeHostRequestKindV1::Enable
                && request.target_viewport == Some(viewport)
    ));
    assert!(matches!(
        batch.requests[1],
        ZrRuntimeHostRequestV1::Ime(ref request)
            if request.kind == ZrRuntimeImeHostRequestKindV1::SetCursorArea
                && request.cursor_area.as_ref().map(|area| area.width) == Some(8.0)
                && request.target_viewport == Some(viewport)
    ));
    assert!(matches!(
        batch.requests[2],
        ZrRuntimeHostRequestV1::Ime(ref request)
            if request.kind == ZrRuntimeImeHostRequestKindV1::SetSurroundingText
                && request
                    .surrounding_text
                    .as_ref()
                    .map(|text| (text.value.as_str(), text.composition_range))
                    == Some(("search", Some(ZrRuntimeImeTextRangeV1::new(1, 6))))
                && request.target_viewport == Some(viewport)
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
    let batch = host_request_batch_from_bytes(&output);

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
    let batch = host_request_batch_from_bytes(&output);

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
