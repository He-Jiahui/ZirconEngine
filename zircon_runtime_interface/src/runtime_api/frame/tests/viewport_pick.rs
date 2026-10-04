use super::*;

fn request() -> ZrRuntimeViewportPickRequestV1 {
    ZrRuntimeViewportPickRequestV1::new(
        ZrRuntimeViewportHandle::new(7),
        ZrRuntimeViewportSizeV1::new(1280, 720),
        ZrRuntimeViewportPixelV1::new(640, 360),
        19,
        23,
        ZrRuntimeViewportPickPurposeV1::Press,
        ZR_RUNTIME_VIEWPORT_PICK_POLICY_INCLUDE_TRANSLUCENT_V1,
    )
}

#[test]
fn request_requires_exact_view_frame_input_and_physical_pixel_identity() {
    let request = request();
    assert!(request.validate_viewport_pick());

    let mut out_of_bounds = request;
    out_of_bounds.pixel.x = out_of_bounds.viewport_size.width;
    assert!(!out_of_bounds.validate_viewport_pick());

    let mut unknown_policy = request;
    unknown_policy.policy_flags = 1 << 31;
    assert!(!unknown_policy.validate_viewport_pick());

    let mut stale_identity = request;
    stale_identity.frame_generation = 0;
    assert!(!stale_identity.validate_viewport_pick());
}

#[test]
fn hit_result_preserves_the_request_identity_and_target_detail() {
    let request = request();
    let result = ZrRuntimeViewportPickResultV1::hit(
        ZrRuntimeViewportPickTicket::new(29),
        request,
        31,
        37,
        41,
        43,
        0.25,
        [1.0, 2.0, 3.0],
        [0.0, 1.0, 0.0],
    );

    assert!(result.matches_request(request));
    assert_eq!(result.entity, 37);
    assert_eq!(result.instance, 41);
    assert_eq!(result.subobject, 43);
    assert!(result.disposition().unwrap().is_terminal());
}

#[test]
fn stale_or_non_hit_completion_cannot_smuggle_a_target() {
    let request = request();
    let mut result = ZrRuntimeViewportPickResultV1::empty(
        ZrRuntimeViewportPickDispositionV1::StaleFrame,
        ZrRuntimeViewportPickTicket::new(29),
        request,
        31,
    );
    assert!(result.matches_request(request));

    result.entity = 37;
    assert!(!result.validate_viewport_pick());
}

#[test]
fn completion_rejects_non_finite_geometry_and_cross_frame_reuse() {
    let request = request();
    let mut result = ZrRuntimeViewportPickResultV1::hit(
        ZrRuntimeViewportPickTicket::new(29),
        request,
        31,
        37,
        0,
        0,
        0.25,
        [1.0, 2.0, 3.0],
        [0.0, 1.0, 0.0],
    );
    result.world_normal[1] = f32::NAN;
    assert!(!result.validate_viewport_pick());

    let mut another_frame = request;
    another_frame.frame_generation += 1;
    assert!(!result.matches_request(another_frame));

    let mut another_pixel = request;
    another_pixel.pixel.x += 1;
    assert!(!result.matches_request(another_pixel));
}

#[test]
fn contract_is_fixed_layout_and_allocation_free() {
    assert!(!core::mem::needs_drop::<ZrRuntimeViewportPickRequestV1>());
    assert!(!core::mem::needs_drop::<ZrRuntimeViewportPickResultV1>());
    assert!(core::mem::size_of::<ZrRuntimeViewportPickRequestV1>() <= 64);
    assert!(core::mem::size_of::<ZrRuntimeViewportPickResultV1>() <= 160);
}
