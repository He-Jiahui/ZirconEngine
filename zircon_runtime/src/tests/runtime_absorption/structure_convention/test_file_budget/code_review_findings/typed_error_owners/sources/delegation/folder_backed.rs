//! 约束类型化错误审查的模块挂载、委托入口与既有检查保留；读取当前源码后按文本验证，不能替代被检查模块的行为测试。
use super::super::super::super::super::*;
use super::super::*;

#[path = "folder_backed/child_ownership.rs"]
mod child_ownership;
#[path = "folder_backed/guard_body.rs"]
mod guard_body;

pub(super) fn assert_typed_error_source_inventory_guard_is_folder_backed() {
    guard_body::assert_typed_error_source_inventory_guard_is_folder_backed();
}

#[test]
fn runtime_15_typed_error_source_inventory_delegation_is_child_backed() {
    child_ownership::assert_typed_error_source_inventory_delegation_is_child_backed();
}
