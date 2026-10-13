//! 固定 Runtime 自有结果的载体布局、空值形状和统一释放入口。
//!
//! 布局和源码断言不读取外来地址，也不代替同一会话中的真实分配与释放测试。
//! 源码守卫直接搜索声明区间，注释同样会参与这些文本检查。

use core::mem::{align_of, size_of};

use crate::{
    ZrOwnedResultV2, ZrRuntimeAllocationId, ZrRuntimeApiV8, ZrRuntimeFrameV2,
    ZIRCON_RUNTIME_ABI_VERSION_V2, ZIRCON_RUNTIME_API_VERSION_V8,
};

#[test]
/// 空结果只有只读指针、长度和不透明编号，并符合当前目标的布局断言。
fn owned_result_v2_exposes_only_an_immutable_view_and_opaque_allocation_id() {
    let result = ZrOwnedResultV2::empty();

    assert!(result.data.is_null());
    assert_eq!(result.len, 0);
    assert_eq!(result.allocation, ZrRuntimeAllocationId::invalid());
    assert!(result.is_empty());
    assert_eq!(size_of::<ZrOwnedResultV2>(), 24);
    assert_eq!(align_of::<ZrOwnedResultV2>(), align_of::<u64>());
}

#[test]
/// V2 帧的像素字段使用统一结果载体，其初始值无需分配或释放。
fn runtime_frame_v2_owns_the_new_result_carrier() {
    let frame = ZrRuntimeFrameV2::empty(ZIRCON_RUNTIME_ABI_VERSION_V2);

    assert_eq!(frame.abi_version, ZIRCON_RUNTIME_ABI_VERSION_V2);
    assert!(frame.rgba.is_empty());
    assert!(frame.is_empty());
}

#[test]
/// 单个零维度或缺少像素不能冒充规范空帧，宽高和载体须同时满足空值规则。
fn runtime_frame_v2_empty_state_requires_the_canonical_zero_sized_carrier() {
    let zero_width = ZrRuntimeFrameV2 {
        abi_version: ZIRCON_RUNTIME_ABI_VERSION_V2,
        width: 0,
        height: 1,
        generation: 0,
        rgba: ZrOwnedResultV2::empty(),
    };
    let missing_rgba = ZrRuntimeFrameV2 {
        abi_version: ZIRCON_RUNTIME_ABI_VERSION_V2,
        width: 1,
        height: 1,
        generation: 0,
        rgba: ZrOwnedResultV2::empty(),
    };

    assert!(!zero_width.is_empty());
    assert!(!missing_rgba.is_empty());
}

#[test]
/// V8 空表具有统一释放槽及冻结表头，但构造器不会安装实际释放函数。
fn runtime_api_v8_requires_one_allocation_release_entry_point() {
    let api = ZrRuntimeApiV8::empty();

    assert_eq!(api.abi_version, ZIRCON_RUNTIME_API_VERSION_V8);
    assert_eq!(api.size_bytes, size_of::<ZrRuntimeApiV8>());
    assert!(api.release_allocation.is_none());
}

#[test]
/// 声明只暴露三项结果字段，禁止泄露分配器元数据或派生隐式复制能力。
fn runtime_owned_result_source_forbids_allocator_metadata_and_copy_semantics() {
    let source = include_str!("../buffer.rs");
    let struct_start = source
        .find("pub struct ZrOwnedResultV2")
        .expect("owned result v2 declaration");
    let impl_start = source[struct_start..]
        .find("impl ZrOwnedResultV2")
        .map(|offset| struct_start + offset)
        .expect("owned result v2 implementation");
    // 只检查声明到实现前的区间；派生标记另看声明前的固定窗口，均属于文本约束。
    let declaration = &source[struct_start..impl_start];
    let derive_start = source.floor_char_boundary(struct_start.saturating_sub(96));
    let derive_window = &source[derive_start..struct_start];

    assert!(declaration.contains("pub data: *const u8"));
    assert!(declaration.contains("pub len: u64"));
    assert!(declaration.contains("pub allocation: ZrRuntimeAllocationId"));
    for forbidden in ["capacity", "owner_token", "free:"] {
        assert!(
            !declaration.contains(forbidden),
            "runtime-owned result must not expose {forbidden}"
        );
    }
    assert!(!derive_window.contains("Copy"));
    assert!(!derive_window.contains("Clone"));
}

#[test]
/// 结果载体迁移同时切断旧表、旧入口和旧版本常量的残留。
fn runtime_api_v6_cannot_survive_the_owned_result_hard_cut() {
    // 同时检查接口表、生成入口和版本常量，避免旧 V6 只从其中一层残留。
    let api_source = include_str!("../runtime_api/abi/api_table.rs");
    let interface_catalog_generator_source = include_str!("../../build.rs");
    let version_source = include_str!("../version.rs");

    assert!(api_source.contains("ZrRuntimeApiV8"));
    assert!(interface_catalog_generator_source.contains("zircon_runtime_get_api_v8"));
    assert!(!api_source.contains("ZrRuntimeApiV6"));
    assert!(!api_source.contains("zircon_runtime_get_api_v6"));
    assert!(version_source.contains("ZIRCON_RUNTIME_API_VERSION_V8"));
    assert!(!version_source.contains("ZIRCON_RUNTIME_API_VERSION_V6"));
}
