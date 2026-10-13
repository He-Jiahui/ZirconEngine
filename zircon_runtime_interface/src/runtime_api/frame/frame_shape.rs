//! 定义 Runtime V2 RGBA 帧的无状态尺寸与报告字节数校验。
//!
//! 此处只检查 ABI 帧携带的宽、高和生产者报告的长度，不读取像素指针，也不证明缓冲区分配或
//! 可读性；App 与 Editor Gateway 在消费像素前调用同一规则，缓冲区所有权和指针前提仍由调用者管理。

use thiserror::Error;

use crate::{ZR_RUNTIME_FRAME_MAX_DIMENSION_V1, ZR_RUNTIME_FRAME_MAX_RGBA_BYTES_V1};

/// Reports why a V2 frame's owned RGBA payload cannot describe its dimensions.
///
/// `validate_runtime_frame_rgba_shape` first rejects zero or over-limit dimensions, then checks the
/// exact four-byte-per-pixel RGBA8 length against the shared byte budget. The App and Editor
/// validate their frame metadata with this helper before consuming the pixels; it never reads the
/// payload pointer.
///
/// 变体按尺寸、预算和精确长度的校验顺序报告错误。其 `actual` / `rgba_len` 都是生产者报告的
/// 字节计数；形状通过不等于底层像素分配存在、完整或可读。
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ZrRuntimeFrameRgbaShapeError {
    /// 宽或高至少一项为零；字段保留帧生产者报告的原始尺寸。
    #[error("runtime frame returned invalid dimensions {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },
    /// 非零宽或高超过共享维度上限。
    #[error("runtime frame dimensions {width}x{height} exceed maximum {maximum}")]
    DimensionsExceedMaximum {
        /// 帧生产者报告的宽度。
        width: u32,
        /// 帧生产者报告的高度。
        height: u32,
        /// 宽或高各自允许的最大值。
        maximum: u32,
    },
    /// 精确 RGBA8 输出大小超过共享字节预算。
    #[error("runtime frame RGBA length {expected} exceeds maximum {maximum}")]
    RgbaLengthExceedsMaximum {
        /// 帧生产者报告的宽度。
        width: u32,
        /// 帧生产者报告的高度。
        height: u32,
        /// 以 `u64` 计算的精确 RGBA8 字节数 `width * height * 4`。
        expected: u64,
        /// 本接口允许的最大 RGBA 字节数。
        maximum: u64,
    },
    /// 生产者报告的字节数与精确 RGBA8 长度不一致。
    #[error("runtime frame {width}x{height} returned {actual} RGBA bytes; expected {expected}")]
    RgbaLengthMismatch {
        /// 帧生产者报告的宽度。
        width: u32,
        /// 帧生产者报告的高度。
        height: u32,
        /// 帧值报告的实际字节数，不代表已验证分配范围。
        actual: u64,
        /// 以 `u64` 计算的精确 RGBA8 字节数 `width * height * 4`。
        expected: u64,
    },
}

/// Validates the exact RGBA8 byte shape reported by a V2 frame.
///
/// Callers must validate ABI version and foreign-output ownership before
/// supplying `rgba_len`; this function only evaluates owned metadata and never
/// dereferences foreign memory.
///
/// # 参数
///
/// `width` 和 `height` 是帧报告的像素尺寸；`rgba_len` 是帧生产者报告的 RGBA 字节数，不证明
/// 对应缓冲区已分配、完整或可读。RGBA8 每像素恰为四字节。
///
/// # 返回值
///
/// 尺寸非零、两个尺寸都不超过 `ZR_RUNTIME_FRAME_MAX_DIMENSION_V1`、精确长度不超过
/// `ZR_RUNTIME_FRAME_MAX_RGBA_BYTES_V1` 且 `rgba_len == width * height * 4` 时返回 `Ok(())`；
/// 否则按检查顺序返回对应错误变体。
///
/// # 使用时机与生命周期
///
/// App `validate_runtime_frame` 和 Editor `ensure_frame_rgba_shape` 用此函数检查跨边界帧的
/// 形状。调用者仍负责先验证 ABI 版本，并保证被检查的长度来自仍有效的帧输出所有者；本函数
/// 只读取三个整数参数，不访问帧缓冲区，也不延长其生命周期。
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
