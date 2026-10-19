//! 动态入口收到的可选 Host V1 回调表检查。
//! 空表指针表示没有回调；非空表的可读性仍是 unsafe 调用方必须建立的前提。

use core::mem::{align_of, size_of};

use thiserror::Error;

use crate::ZIRCON_RUNTIME_ABI_VERSION_V1;

use super::api_table::ZrHostApiV1;

/// Reports why a copied V1 host callback table does not match the frozen shape.
/// 表引用已合法建立后，分别报告历史 V1 版本与完整布局大小不符。
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ZrRuntimeHostApiV1ShapeError {
    /// `expected` 为 Host V1 编号，`actual` 为收到的表头编号。
    #[error("runtime host API V1 requires version {expected}, received version {actual}")]
    ApiVersionMismatch { expected: u32, actual: u32 },
    /// `expected` 为本地完整布局大小，`actual` 为收到的大小字段。
    #[error("runtime host API V1 requires table size {expected}, received {actual}")]
    TableSizeMismatch { expected: usize, actual: usize },
}

/// Reports why an optional host-table pointer cannot be inspected safely enough to read metadata.
/// 原始指针检查可识别未对齐地址，或转交合法表引用的形状错误；无法检测悬垂地址。
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ZrRuntimeHostApiV1PointerError {
    #[error(
        "runtime host API V1 pointer {address:#x} is not aligned to required alignment {expected_alignment}"
    )]
    MisalignedPointer {
        /// 收到的非空地址，仅用于诊断，不会在此错误分支解引用。
        address: usize,
        /// 创建 Host 表引用所需的本地对齐值。
        expected_alignment: usize,
    },
    /// 地址检查后，表头不符合冻结 V1 布局。
    #[error(transparent)]
    Shape(#[from] ZrRuntimeHostApiV1ShapeError),
}

/// Validates the frozen V1 host-table version and exact table-size metadata.
///
/// The callback slots are optional in V1. Capability requirements and callback
/// dependencies belong to the later handshake receipt, not this shape check.
///
/// # 参数与返回值
/// `host` 是已安全持有的完整表。返回精确版本/大小检查结果，不调用或保存其中回调。
/// 检查版本早于大小，两个 V1 回调均可为空。
pub fn validate_runtime_host_api_v1_shape(
    host: &ZrHostApiV1,
) -> Result<(), ZrRuntimeHostApiV1ShapeError> {
    if host.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return Err(ZrRuntimeHostApiV1ShapeError::ApiVersionMismatch {
            expected: ZIRCON_RUNTIME_ABI_VERSION_V1,
            actual: host.abi_version,
        });
    }

    let expected = size_of::<ZrHostApiV1>();
    if host.size_bytes != expected {
        return Err(ZrRuntimeHostApiV1ShapeError::TableSizeMismatch {
            expected,
            actual: host.size_bytes,
        });
    }

    Ok(())
}

/// Validates an optional raw V1 host-table pointer before reading its metadata.
///
/// A null pointer is the V8 entry point's established "no host callbacks"
/// form. A non-null pointer must be aligned and point to a readable
/// `ZrHostApiV1` for this call. Address readability cannot be established
/// safely in-process; untrusted callers require the planned process-isolated
/// ABI boundary.
///
/// # Safety
///
/// For a non-null pointer, `host` must be valid to read a `ZrHostApiV1` for
/// the duration of this call. The function checks alignment before creating a
/// reference but cannot make an unmapped or dangling address safe to read.
///
/// # 参数与返回值
/// `host` 是动态入口收到的可选表地址；空值直接成功，未对齐值在读取前返回错误。
/// 对齐的非空值继续核对完整表头，成功不表示回调身份、能力或可调用性已经验证。
///
/// # 使用时机与生命周期
/// Runtime 的 `zircon_runtime_get_api_v8` 在返回静态函数表前使用本函数。
/// App 加载器传入本次调用内存活的局部 Host 表；这里不保存表引用。
pub unsafe fn validate_runtime_host_api_v1_pointer(
    host: *const ZrHostApiV1,
) -> Result<(), ZrRuntimeHostApiV1PointerError> {
    if host.is_null() {
        return Ok(());
    }

    let address = host as usize;
    let expected_alignment = align_of::<ZrHostApiV1>();
    if !address.is_multiple_of(expected_alignment) {
        return Err(ZrRuntimeHostApiV1PointerError::MisalignedPointer {
            address,
            expected_alignment,
        });
    }

    // 此处只证明了非空与对齐；可读且已初始化的完整表由调用方的 unsafe 契约提供。
    // SAFETY: 非空和对齐已检查；完整可读且已初始化的表及借用期间有效性由本 unsafe 入口的契约和调用方保证。
    let host = unsafe { &*host };
    validate_runtime_host_api_v1_shape(host)?;
    Ok(())
}
