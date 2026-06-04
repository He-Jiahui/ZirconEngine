use super::support::*;

#[test]
fn mouse_wheel_events_reject_invalid_unit_and_delta() {
    let api = runtime_api();
    let handle_event = api.handle_event.expect("handle_event");
    let session = create_test_session(api);

    let status = unsafe {
        handle_event(
            session,
            ZrRuntimeEventV1::mouse_wheel_delta(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                ZrRuntimeViewportHandle::new(1),
                99,
                1.0,
                2.0,
            ),
        )
    };
    assert_eq!(status.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(status_message(status), "unknown runtime mouse wheel unit");

    let status = unsafe {
        handle_event(
            session,
            ZrRuntimeEventV1::mouse_wheel_delta(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                ZrRuntimeViewportHandle::new(1),
                ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1,
                f32::NAN,
                2.0,
            ),
        )
    };

    destroy_test_session(api, session);
    assert_eq!(status.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(status_message(status), "invalid runtime mouse wheel delta");
}

#[test]
fn window_scale_factor_events_reject_non_positive_factor() {
    let api = runtime_api();
    let handle_event = api.handle_event.expect("handle_event");
    let session = create_test_session(api);

    let status = unsafe {
        handle_event(
            session,
            ZrRuntimeEventV1::window_scale_factor_changed(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                ZrRuntimeViewportHandle::new(1),
                0.0,
            ),
        )
    };
    assert_eq!(status.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(
        status_message(status),
        "invalid runtime window scale factor"
    );

    let status = unsafe {
        handle_event(
            session,
            ZrRuntimeEventV1::window_backend_scale_factor_changed(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                ZrRuntimeViewportHandle::new(1),
                -1.0,
            ),
        )
    };

    destroy_test_session(api, session);
    assert_eq!(status.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(
        status_message(status),
        "invalid runtime window scale factor"
    );
}

#[test]
fn ime_host_requests_reject_invalid_cursor_payloads() {
    let api = runtime_api();
    let handle_event = api.handle_event.expect("handle_event");
    let session = create_test_session(api);
    let payload = "你".as_bytes();

    let status = unsafe {
        handle_event(
            session,
            ZrRuntimeEventV1::ime_surrounding_text(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                ZrRuntimeViewportHandle::new(1),
                ZrByteSlice {
                    data: payload.as_ptr(),
                    len: payload.len(),
                },
                1,
                0,
            ),
        )
    };
    assert_eq!(status.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(
        status_message(status),
        "invalid runtime ime surrounding text"
    );

    let status = unsafe {
        handle_event(
            session,
            ZrRuntimeEventV1::ime_cursor_area(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                ZrRuntimeViewportHandle::new(1),
                16.0,
                24.0,
                0,
                18,
            ),
        )
    };

    destroy_test_session(api, session);
    assert_eq!(status.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(status_message(status), "invalid runtime ime cursor area");
}
