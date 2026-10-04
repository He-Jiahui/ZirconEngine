//! 约束类型化错误审查的模块挂载、委托入口与既有检查保留；读取当前源码后按文本验证，不能替代被检查模块的行为测试。
use super::super::*;

#[path = "structure_guard/budgets.rs"]
mod budgets;
#[path = "structure_guard/route_ownership.rs"]
mod route_ownership;

pub(in super::super) const TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STRUCTURE_GUARD_CHILDREN:
    &[(&str, &str, &str)] = &[
    (
        "route_ownership",
        TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STRUCTURE_GUARD_ROUTE_CHILD,
        "pub(in super::super) fn assert_typed_error_source_inventory_child_sources_structure_guard_is_child_backed",
    ),
    (
        "budgets",
        TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STRUCTURE_GUARD_BUDGETS_CHILD,
        "pub(in super::super) fn assert_typed_error_source_inventory_child_sources_structure_guard_budgets_are_current",
    ),
];

pub(in super::super) fn assert_typed_error_source_inventory_child_sources_are_folder_backed() {
    route_ownership::assert_typed_error_source_inventory_child_sources_structure_guard_is_child_backed();
    budgets::assert_typed_error_source_inventory_child_sources_structure_guard_budgets_are_current(
    );
}

#[test]
fn runtime_15_typed_error_source_inventory_child_sources_structure_guard_is_child_backed() {
    assert_typed_error_source_inventory_child_sources_are_folder_backed();
}
