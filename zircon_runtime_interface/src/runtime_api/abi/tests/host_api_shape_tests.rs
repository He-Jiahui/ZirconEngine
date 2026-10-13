//! 核对 Host V1 表头、可选空指针以及解引用前的对齐拒绝。
//!
//! 非空指针用例只走提前返回的错误分支，不把字节数组解释为已初始化的 Host 表。
//! 元数据正确的空表同样不能证明回调存在或调用生命周期成立。

use core::mem::{align_of, size_of};

use crate::ZIRCON_RUNTIME_ABI_VERSION_V1;

use super::{
    validate_runtime_host_api_v1_pointer, validate_runtime_host_api_v1_shape, ZrHostApiV1,
    ZrRuntimeHostApiV1PointerError, ZrRuntimeHostApiV1ShapeError,
};

#[test]
/// 本地 V1 空表保留冻结版本和尺寸，因而通过元数据检查。
fn runtime_host_api_v1_shape_accepts_the_frozen_exact_table() {
    assert_eq!(
        validate_runtime_host_api_v1_shape(&ZrHostApiV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1)),
        Ok(())
    );
}

#[test]
/// 不同 ABI 版本被拒绝，并返回双方版本以供调用方诊断。
fn runtime_host_api_v1_shape_rejects_a_different_api_family() {
    let host = ZrHostApiV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1 + 1);

    assert_eq!(
        validate_runtime_host_api_v1_shape(&host),
        Err(ZrRuntimeHostApiV1ShapeError::ApiVersionMismatch {
            expected: ZIRCON_RUNTIME_ABI_VERSION_V1,
            actual: ZIRCON_RUNTIME_ABI_VERSION_V1 + 1,
        })
    );
}

#[test]
/// 尺寸缩短触发精确尺寸错误；不会据此构造较短表的引用。
fn runtime_host_api_v1_shape_rejects_a_non_exact_table_size() {
    let mut host = ZrHostApiV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
    host.size_bytes = size_of::<ZrHostApiV1>() - 1;

    assert_eq!(
        validate_runtime_host_api_v1_shape(&host),
        Err(ZrRuntimeHostApiV1ShapeError::TableSizeMismatch {
            expected: size_of::<ZrHostApiV1>(),
            actual: size_of::<ZrHostApiV1>() - 1,
        })
    );
}

#[test]
/// 空指针表示可选 Host 表缺席，校验直接返回成功。
fn runtime_host_api_v1_pointer_accepts_null_as_an_absent_optional_table() {
    assert_eq!(
        // SAFETY: 传入空指针；校验器的空值分支不读取任何表内存。
        unsafe { validate_runtime_host_api_v1_pointer(core::ptr::null()) },
        Ok(())
    );
}

#[test]
/// 已断言未对齐的地址必须在创建 Host 表引用之前被拒绝。
fn runtime_host_api_v1_pointer_rejects_misalignment_before_dereference() {
    // TODO: [ZCA-ABI-001] 字节数组只保证一字节对齐，偏移一不能普遍保证目标地址未对齐。
    // 当前未复现该夹具断言失败；下一步用显式对齐的存储或按实际基址计算偏移验证此用例。
    let storage = [0_u8; size_of::<ZrHostApiV1>() + align_of::<ZrHostApiV1>()];
    // SAFETY: 数组长度大于一，偏移一仍位于同一数组内；这里只形成指针，不解引用。
    let pointer = unsafe { storage.as_ptr().add(1) }.cast::<ZrHostApiV1>();

    assert_ne!((pointer as usize) % align_of::<ZrHostApiV1>(), 0);
    assert_eq!(
        // SAFETY: 前一断言已证明地址未对齐，校验器在解引用前返回对应错误。
        unsafe { validate_runtime_host_api_v1_pointer(pointer) },
        Err(ZrRuntimeHostApiV1PointerError::MisalignedPointer {
            address: pointer as usize,
            expected_alignment: align_of::<ZrHostApiV1>(),
        })
    );
}
