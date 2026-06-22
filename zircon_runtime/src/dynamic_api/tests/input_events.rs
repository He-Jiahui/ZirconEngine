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
fn mouse_wheel_at_events_decode_delta_bits_for_dynamic_session() {
    let source = concat!(
        include_str!("../session.rs"),
        include_str!("../session/events.rs"),
    );
    let wheel_handler = source
        .split("fn handle_mouse_wheel(&mut self, event: ZrRuntimeEventV1) -> ZrStatus")
        .nth(1)
        .expect("runtime dynamic session mouse wheel handler");
    let next_handler = wheel_handler
        .find("fn handle_lifecycle")
        .expect("mouse wheel handler should end before lifecycle handler");
    let wheel_handler = &wheel_handler[..next_handler];

    assert!(wheel_handler.contains("ZR_RUNTIME_MOUSE_WHEEL_COORDS_PRESENT_V1"));
    assert!(wheel_handler.contains("f32::from_bits(event.key_code)"));
    assert!(wheel_handler.contains("f32::from_bits(event.scan_code)"));

    let decode = wheel_handler
        .find("f32::from_bits(event.key_code)")
        .expect("wheel-at-point delta bits should decode before validation");
    let finite_check = wheel_handler
        .find("if !delta_x.is_finite() || !delta_y.is_finite()")
        .expect("decoded wheel deltas should be validated");
    let submit = wheel_handler
        .find("MouseWheelEvent::new(unit, delta_x, delta_y)")
        .expect("decoded wheel deltas should feed runtime input state");

    assert!(decode < finite_check);
    assert!(finite_check < submit);
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
