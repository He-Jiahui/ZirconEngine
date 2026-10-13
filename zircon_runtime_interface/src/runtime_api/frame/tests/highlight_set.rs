use super::{ZrRuntimeHighlightRenderAttributesV1, ZrRuntimeHighlightSetV1};
use crate::handles::ZrRuntimeViewportHandle;

#[test]
/// 验证安全构造的高亮请求能在调用方数组仍存活时借读原始实体顺序。
fn validates_a_borrowed_entity_slice_without_owning_it() {
    let entities = [7, 2, 7];
    let request = ZrRuntimeHighlightSetV1::new(
        ZrRuntimeViewportHandle::new(5),
        3,
        &entities,
        ZrRuntimeHighlightRenderAttributesV1::outlined([0.4, 0.6, 0.8, 1.0]),
    );

    // SAFETY: `entities` 是本作用域中的已初始化栈数组；请求借用它，且该数组在同步验证期间仍存活。
    assert!(unsafe { request.validate() });
    // SAFETY: 同一栈数组仍存活、对齐且可读，读取范围正是 `from_slice` 记录的完整长度。
    assert_eq!(
        unsafe { request.entities.as_slice() },
        Some(entities.as_slice())
    );
}

#[test]
/// 拒绝不能组成合法 `u64` 切片的未对齐指针和超出 Rust 切片上限的长度。
fn rejects_misaligned_or_oversized_borrowed_slices() {
    let misaligned = super::ZrRuntimeEntityIdSliceV1 {
        data: 1_usize as *const u64,
        len: 1,
    };
    let oversized = super::ZrRuntimeEntityIdSliceV1 {
        data: core::ptr::NonNull::<u64>::dangling().as_ptr(),
        len: (isize::MAX as usize / core::mem::size_of::<u64>()) + 1,
    };

    // SAFETY: 地址 1 未按 u64 对齐，as_slice 在构造切片前由对齐检查返回 None。
    assert!(unsafe { misaligned.as_slice() }.is_none());
    // SAFETY: 长度超过 Rust 切片上限，as_slice 在构造切片前由长度检查返回 None。
    assert!(unsafe { oversized.as_slice() }.is_none());
}
