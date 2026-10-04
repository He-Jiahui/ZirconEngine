use super::*;

fn request() -> RenderViewportPickRequest {
    RenderViewportPickRequest::new(
        RenderViewportHandle::new(7),
        UVec2::new(1280, 720),
        UVec2::new(640, 360),
        19,
        23,
        RenderViewportPickPurpose::Press,
        RenderViewportPickPolicy::default(),
    )
}

#[test]
fn request_and_result_retain_presented_frame_identity() {
    let request = request();
    let ticket = RenderViewportPickTicket::new(3);
    let result = RenderViewportPickResult::hit(
        ticket,
        request,
        29,
        31,
        37,
        41,
        2.0,
        [1.0, 2.0, 3.0],
        [0.0, 1.0, 0.0],
    );

    assert!(request.is_valid());
    assert!(result.matches_request(request));
    assert_eq!(result.entity, 31);
    assert_eq!(result.instance, 37);
    assert_eq!(result.subobject, 41);
}

#[test]
fn result_rejects_cross_frame_and_non_finite_geometry() {
    let request = request();
    let mut result = RenderViewportPickResult::hit(
        RenderViewportPickTicket::new(3),
        request,
        29,
        31,
        0,
        0,
        2.0,
        [1.0, 2.0, 3.0],
        [0.0, 1.0, 0.0],
    );
    let mut another_frame = request;
    another_frame.frame_generation += 1;
    assert!(!result.matches_request(another_frame));

    let mut another_ticket = result;
    another_ticket.ticket = RenderViewportPickTicket::new(5);
    assert!(!another_ticket.matches_ticketed_request(RenderViewportPickTicket::new(3), request));

    let mut another_pixel = request;
    another_pixel.pixel.x += 1;
    assert!(!result.matches_request(another_pixel));

    result.depth = f32::NAN;
    assert!(!result.is_valid());
}

#[test]
fn pick_policy_exposes_typed_translucent_and_backface_decisions() {
    let translucent =
        RenderViewportPickPolicy::from_bits(RenderViewportPickPolicy::INCLUDE_TRANSLUCENT).unwrap();
    let both = RenderViewportPickPolicy::from_bits(
        RenderViewportPickPolicy::INCLUDE_TRANSLUCENT | RenderViewportPickPolicy::INCLUDE_BACKFACES,
    )
    .unwrap();

    assert!(translucent.includes_translucent());
    assert!(!translucent.includes_backfaces());
    assert!(both.includes_translucent());
    assert!(both.includes_backfaces());
}
