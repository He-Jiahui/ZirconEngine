use super::support::*;
use zircon_runtime_interface::{
    ZrRuntimeViewportPickDispositionV1, ZrRuntimeViewportPickPurposeV1,
    ZrRuntimeViewportPickRequestV1, ZrRuntimeViewportPickResultV1, ZrRuntimeViewportPickTicket,
    ZrRuntimeViewportPixelV1,
};

fn valid_pick_request() -> ZrRuntimeViewportPickRequestV1 {
    ZrRuntimeViewportPickRequestV1::new(
        default_viewport(),
        valid_viewport_size(),
        ZrRuntimeViewportPixelV1::new(32, 24),
        19,
        23,
        ZrRuntimeViewportPickPurposeV1::Press,
        0,
    )
}

#[test]
fn headless_viewport_pick_closes_one_exact_unavailable_ticket() {
    let api = runtime_api();
    let session = create_test_session(api);
    let request_pick = api.request_viewport_pick.expect("request_viewport_pick");
    let poll_pick = api.poll_viewport_pick.expect("poll_viewport_pick");
    let request = valid_pick_request();
    let mut ticket = ZrRuntimeViewportPickTicket::invalid();

    let status = unsafe { request_pick(session, request, &mut ticket) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
    assert!(ticket.is_valid());

    let mut result = ZrRuntimeViewportPickResultV1::invalid();
    let status = unsafe { poll_pick(session, ticket, &mut result) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
    assert!(result.matches_request(request));
    assert_eq!(
        result.disposition(),
        Some(ZrRuntimeViewportPickDispositionV1::Unavailable)
    );

    let status = unsafe { poll_pick(session, ticket, &mut result) };
    assert_session_status(
        status,
        ZrStatusCode::NotFound,
        "runtime viewport-pick ticket not found",
    );
    assert_eq!(result, ZrRuntimeViewportPickResultV1::invalid());
    destroy_test_session(api, session);
}

#[test]
fn invalid_viewport_pick_request_never_allocates_a_ticket() {
    let api = runtime_api();
    let session = create_test_session(api);
    let request_pick = api.request_viewport_pick.expect("request_viewport_pick");
    let mut request = valid_pick_request();
    request.pixel.x = request.viewport_size.width;
    let mut ticket = ZrRuntimeViewportPickTicket::new(99);

    let status = unsafe { request_pick(session, request, &mut ticket) };

    assert_session_status(
        status,
        ZrStatusCode::InvalidArgument,
        "invalid runtime viewport-pick request",
    );
    assert_eq!(ticket, ZrRuntimeViewportPickTicket::invalid());
    destroy_test_session(api, session);
}
