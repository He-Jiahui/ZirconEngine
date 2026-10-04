//! 约束类型化错误审查的模块挂载、委托入口与既有检查保留；读取当前源码后按文本验证，不能替代被检查模块的行为测试。
use super::super::*;

#[path = "typed_error_owners/budgets.rs"]
mod budgets;
#[path = "typed_error_owners/child_ownership.rs"]
mod child_ownership;
#[path = "typed_error_owners/delegation.rs"]
mod delegation;
#[path = "typed_error_owners/root_child_rows.rs"]
mod root_child_rows;
#[path = "typed_error_owners/root_inventory.rs"]
mod root_inventory;
#[path = "typed_error_owners/root_paths.rs"]
mod root_paths;
#[path = "typed_error_owners/root_sources.rs"]
mod root_sources;
#[path = "typed_error_owners/source_inventory.rs"]
mod source_inventory;
#[path = "typed_error_owners/structure_assertions.rs"]
mod structure_assertions;

pub(super) use root_child_rows::*;
pub(super) use root_paths::*;
pub(super) use root_sources::*;

pub(super) const GUARD: &str =
    "runtime_15_code_review_findings_typed_error_structure_guard_is_child_owner";
pub(super) const TYPED_ERROR_CHILD_OWNER_LINE_BUDGET: usize = 800;

pub(super) fn assert_typed_error_child_owners_are_folder_backed() {
    structure_assertions::assert_typed_error_child_owners_are_folder_backed();
}

pub(super) fn typed_error_children_source() -> String {
    source_inventory::typed_error_children_source()
}

pub(super) fn assert_typed_error_line_budgets() {
    source_inventory::assert_typed_error_line_budgets();
}

pub(super) fn typed_error_review_guard_count() -> usize {
    source_inventory::typed_error_review_guard_count()
}
