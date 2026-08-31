use super::{validate_runtime_frame_rgba_shape, ZrRuntimeFrameRgbaShapeError};
use crate::{ZR_RUNTIME_FRAME_MAX_DIMENSION_V1, ZR_RUNTIME_FRAME_MAX_RGBA_BYTES_V1};

#[test]
fn frame_rgba_shape_accepts_exact_nonempty_payloads() {
    assert_eq!(validate_runtime_frame_rgba_shape(1, 1, 4), Ok(()));
    assert_eq!(
        validate_runtime_frame_rgba_shape(8_192, 8_192, ZR_RUNTIME_FRAME_MAX_RGBA_BYTES_V1 as u64),
        Ok(())
    );
}

#[test]
fn frame_rgba_shape_rejects_zero_dimensions_even_with_an_empty_payload() {
    assert_eq!(
        validate_runtime_frame_rgba_shape(0, 0, 0),
        Err(ZrRuntimeFrameRgbaShapeError::InvalidDimensions {
            width: 0,
            height: 0,
        })
    );
}

#[test]
fn frame_rgba_shape_rejects_dimensions_above_the_shared_limit() {
    assert_eq!(
        validate_runtime_frame_rgba_shape(ZR_RUNTIME_FRAME_MAX_DIMENSION_V1 + 1, 1, 0),
        Err(ZrRuntimeFrameRgbaShapeError::DimensionsExceedMaximum {
            width: ZR_RUNTIME_FRAME_MAX_DIMENSION_V1 + 1,
            height: 1,
            maximum: ZR_RUNTIME_FRAME_MAX_DIMENSION_V1,
        })
    );
}

#[test]
fn frame_rgba_shape_rejects_dimensions_whose_exact_output_exceeds_the_budget() {
    let width = ZR_RUNTIME_FRAME_MAX_DIMENSION_V1;
    let height = 4_097;
    let expected = u64::from(width) * u64::from(height) * 4;

    assert_eq!(
        validate_runtime_frame_rgba_shape(width, height, 0),
        Err(ZrRuntimeFrameRgbaShapeError::RgbaLengthExceedsMaximum {
            width,
            height,
            expected,
            maximum: ZR_RUNTIME_FRAME_MAX_RGBA_BYTES_V1 as u64,
        })
    );
}

#[test]
fn frame_rgba_shape_rejects_a_payload_that_is_not_the_exact_rgba_length() {
    assert_eq!(
        validate_runtime_frame_rgba_shape(1, 1, 3),
        Err(ZrRuntimeFrameRgbaShapeError::RgbaLengthMismatch {
            width: 1,
            height: 1,
            actual: 3,
            expected: 4,
        })
    );
}
