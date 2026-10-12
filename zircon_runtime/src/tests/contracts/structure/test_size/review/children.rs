//! 约束审查目录结构的模块挂载、委托入口与既有检查保留；读取当前源码后按文本验证，不能替代被检查模块的行为测试。
use super::super::*;

#[path = "../../../../structure/test_size/review/children/budgets.rs"]
mod budgets;
#[path = "../../../../structure/test_size/review/children/delegation.rs"]
mod delegation;
#[path = "../../../../structure/test_size/review/children/folders.rs"]
mod folder_backed_summary;
#[path = "../../../../structure/test_size/review/children/plugin_importer.rs"]
mod plugin_importer;
#[path = "../../../../test_size/review/children/review_guard_groups.rs"]
mod review_guard_groups;
#[path = "../../../../structure/test_size/review/children/root_child_rows.rs"]
mod root_child_rows;
#[path = "../../../../structure/test_size/review/children/root_inventory.rs"]
mod root_inventory;
#[path = "../../../../structure/test_size/review/children/root_paths.rs"]
mod root_paths;
#[path = "../../../../structure/test_size/review/children/root_sources.rs"]
mod root_sources;
#[path = "../../../../structure/test_size/review/children/typed_error.rs"]
mod typed_error;

pub(super) use root_child_rows::*;
pub(super) use root_paths::*;
pub(super) use root_sources::*;

pub(super) fn assert_nested_structure_children_are_mounted() {
    folder_backed_summary::assert_folder_backed_summary_structure_children_are_mounted();
    typed_error::assert_typed_error_structure_children_are_mounted();
}
