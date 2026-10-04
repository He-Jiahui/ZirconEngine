use super::*;
use zircon_runtime_interface::ZR_RUNTIME_FRAME_MAX_DIMENSION_V1;

#[test]
fn frame_shape_rejects_shared_dimension_and_rgba_limits() {
    let dimension_error = ensure_frame_rgba_shape(ZR_RUNTIME_FRAME_MAX_DIMENSION_V1 + 1, 1, &[])
        .expect_err("oversized foreign frame dimensions must be rejected");
    assert!(matches!(
        dimension_error,
        GatewayError::Protocol { message }
            if message.contains("exceed maximum 16384")
    ));

    let rgba_error = ensure_frame_rgba_shape(ZR_RUNTIME_FRAME_MAX_DIMENSION_V1, 4_097, &[])
        .expect_err("oversized foreign RGBA lengths must be rejected");
    assert!(matches!(
        rgba_error,
        GatewayError::Protocol { message }
            if message.contains("exceeds maximum 268435456")
    ));

    let empty_payload_error = ensure_frame_rgba_shape(1, 1, &[])
        .expect_err("nonempty frame dimensions require an exact RGBA payload");
    assert!(matches!(
        empty_payload_error,
        GatewayError::Protocol { message }
            if message.contains("returned 0 RGBA bytes; expected 4")
    ));
}
