//! 已经安全持有的 Runtime V8 表的精确表头验证。
//! 这里不读取外来原始指针，也不接受同版本的前缀或尾部扩展。

use core::mem::size_of;

use thiserror::Error;

use crate::ZIRCON_RUNTIME_API_VERSION_V8;

use super::api_table::ZrRuntimeApiV8;

/// Reports why a copied runtime API table does not match the frozen V8 shape.
/// 供宿主加载器和编辑器 gateway 报告表族或大小不符的原因。
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ZrRuntimeApiV8ShapeError {
    /// `expected` 为冻结表族版本，`actual` 为已持有表的版本字段。
    #[error("runtime API V8 requires version {expected}, received version {actual}")]
    ApiVersionMismatch { expected: u32, actual: u32 },
    /// `expected` 为本地完整布局大小，`actual` 为提供方声明的字节数。
    #[error("runtime API V8 requires table size {expected}, received {actual}")]
    TableSizeMismatch { expected: usize, actual: usize },
}

/// Validates the frozen V8 family and exact table-size metadata.
///
/// The caller must already own a safely copied table or have independently
/// validated its foreign pointer. This does not validate provider identity,
/// BuildSet membership, required slots, or any function pointer.
///
/// # 参数与返回值
/// `api` 是调用方已经合法持有的完整表引用。成功只表示版本与精确大小匹配；
/// 失败先报告版本不符，再报告大小不符，不改变表或会话状态。
///
/// # 约束与参考
/// Host 加载器须先证明外来表可读取，再核对槽；编辑器 gateway 对复制的表调用此函数。
/// 空槽表 [`ZrRuntimeApiV8::empty`] 可通过本检查，因此通过检查不授予调用能力。
pub fn validate_runtime_api_v8_shape(api: &ZrRuntimeApiV8) -> Result<(), ZrRuntimeApiV8ShapeError> {
    if api.abi_version != ZIRCON_RUNTIME_API_VERSION_V8 {
        return Err(ZrRuntimeApiV8ShapeError::ApiVersionMismatch {
            expected: ZIRCON_RUNTIME_API_VERSION_V8,
            actual: api.abi_version,
        });
    }

    let expected = size_of::<ZrRuntimeApiV8>();
    if api.size_bytes != expected {
        return Err(ZrRuntimeApiV8ShapeError::TableSizeMismatch {
            expected,
            actual: api.size_bytes,
        });
    }

    Ok(())
}
