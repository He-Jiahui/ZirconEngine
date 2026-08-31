use thiserror::Error;

use crate::{ZR_RUNTIME_FRAME_MAX_DIMENSION_V1, ZR_RUNTIME_FRAME_MAX_RGBA_BYTES_V1};

/// Reports why a V2 frame's owned RGBA payload cannot describe its dimensions.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ZrRuntimeFrameRgbaShapeError {
    #[error("runtime frame returned invalid dimensions {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },
    #[error("runtime frame dimensions {width}x{height} exceed maximum {maximum}")]
    DimensionsExceedMaximum {
        width: u32,
        height: u32,
        maximum: u32,
    },
    #[error("runtime frame RGBA length {expected} exceeds maximum {maximum}")]
    RgbaLengthExceedsMaximum {
        width: u32,
        height: u32,
        expected: u64,
        maximum: u64,
    },
    #[error("runtime frame {width}x{height} returned {actual} RGBA bytes; expected {expected}")]
    RgbaLengthMismatch {
        width: u32,
        height: u32,
        actual: u64,
        expected: u64,
    },
}

/// Validates the exact RGBA8 byte shape reported by a V2 frame.
///
/// Callers must validate ABI version and foreign-output ownership before
/// supplying `rgba_len`; this function only evaluates owned metadata and never
/// dereferences foreign memory.
pub fn validate_runtime_frame_rgba_shape(
    width: u32,
    height: u32,
    rgba_len: u64,
) -> Result<(), ZrRuntimeFrameRgbaShapeError> {
    if width == 0 || height == 0 {
        return Err(ZrRuntimeFrameRgbaShapeError::InvalidDimensions { width, height });
    }
    if width > ZR_RUNTIME_FRAME_MAX_DIMENSION_V1 || height > ZR_RUNTIME_FRAME_MAX_DIMENSION_V1 {
        return Err(ZrRuntimeFrameRgbaShapeError::DimensionsExceedMaximum {
            width,
            height,
            maximum: ZR_RUNTIME_FRAME_MAX_DIMENSION_V1,
        });
    }
    let expected = u64::from(width) * u64::from(height) * 4;
    let maximum = ZR_RUNTIME_FRAME_MAX_RGBA_BYTES_V1 as u64;
    if expected > maximum {
        return Err(ZrRuntimeFrameRgbaShapeError::RgbaLengthExceedsMaximum {
            width,
            height,
            expected,
            maximum,
        });
    }
    if rgba_len != expected {
        return Err(ZrRuntimeFrameRgbaShapeError::RgbaLengthMismatch {
            width,
            height,
            actual: rgba_len,
            expected,
        });
    }
    Ok(())
}
