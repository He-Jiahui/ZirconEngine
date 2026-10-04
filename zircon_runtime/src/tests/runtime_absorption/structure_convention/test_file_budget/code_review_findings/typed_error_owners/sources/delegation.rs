//! 约束类型化错误审查的模块挂载、委托入口与既有检查保留；读取当前源码后按文本验证，不能替代被检查模块的行为测试。
use super::super::super::super::*;
use super::*;

#[path = "delegation/folder_backed.rs"]
mod folder_backed;
#[path = "delegation/parent_delegation.rs"]
mod parent_delegation;
#[path = "delegation/source_inventory_mounts.rs"]
mod source_inventory_mounts;
#[path = "delegation/source_ownership.rs"]
mod source_ownership;

pub(super) fn assert_typed_error_source_inventory_is_child_owner(
    sources: &TypedErrorSourceInventorySources,
) {
    parent_delegation::assert_typed_error_structure_delegates_source_inventory(sources);
    source_inventory_mounts::assert_typed_error_source_inventory_parent_mounts_focused_owners(
        sources,
    );
    source_ownership::assert_typed_error_source_inventory_paths_and_reads_are_child_owned(sources);
}

#[test]
fn runtime_15_typed_error_source_inventory_guard_is_folder_backed() {
    folder_backed::assert_typed_error_source_inventory_guard_is_folder_backed();
}
