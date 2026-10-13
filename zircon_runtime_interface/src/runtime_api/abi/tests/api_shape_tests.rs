//! 核对 Runtime V8 表头的精确匹配规则。
//!
//! 每个用例只改变一个元数据字段，确认错误携带期望值和实际值。
//! 空表通过形状校验并不表示槽位可调用；能力检查由调用方另行完成。

use core::mem::size_of;

use crate::ZIRCON_RUNTIME_API_VERSION_V8;

use super::{validate_runtime_api_v8_shape, ZrRuntimeApiV8, ZrRuntimeApiV8ShapeError};

#[test]
/// 空表的版本和字节数符合冻结形状，即使所有函数槽仍为空。
fn runtime_api_v8_shape_accepts_the_frozen_exact_table() {
    assert_eq!(
        validate_runtime_api_v8_shape(&ZrRuntimeApiV8::empty()),
        Ok(())
    );
}

#[test]
/// 改变版本族后返回版本错误，不把不同版本当成兼容前缀。
fn runtime_api_v8_shape_rejects_a_different_api_family() {
    let mut api = ZrRuntimeApiV8::empty();
    api.abi_version = ZIRCON_RUNTIME_API_VERSION_V8 + 1;

    assert_eq!(
        validate_runtime_api_v8_shape(&api),
        Err(ZrRuntimeApiV8ShapeError::ApiVersionMismatch {
            expected: ZIRCON_RUNTIME_API_VERSION_V8,
            actual: ZIRCON_RUNTIME_API_VERSION_V8 + 1,
        })
    );
}

#[test]
/// 少一个字节的表被拒绝，错误保留本地类型的完整尺寸。
fn runtime_api_v8_shape_rejects_a_shorter_table() {
    let mut api = ZrRuntimeApiV8::empty();
    api.size_bytes = size_of::<ZrRuntimeApiV8>() - 1;

    assert_eq!(
        validate_runtime_api_v8_shape(&api),
        Err(ZrRuntimeApiV8ShapeError::TableSizeMismatch {
            expected: size_of::<ZrRuntimeApiV8>(),
            actual: size_of::<ZrRuntimeApiV8>() - 1,
        })
    );
}

#[test]
/// 多一个字节的表也被拒绝，避免同版本静默追加尾部字段。
fn runtime_api_v8_shape_rejects_a_larger_table() {
    let mut api = ZrRuntimeApiV8::empty();
    api.size_bytes = size_of::<ZrRuntimeApiV8>() + 1;

    assert_eq!(
        validate_runtime_api_v8_shape(&api),
        Err(ZrRuntimeApiV8ShapeError::TableSizeMismatch {
            expected: size_of::<ZrRuntimeApiV8>(),
            actual: size_of::<ZrRuntimeApiV8>() + 1,
        })
    );
}
