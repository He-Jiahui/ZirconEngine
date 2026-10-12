//! 约束类型化错误审查的模块挂载、委托入口与既有检查保留；读取当前源码后按文本验证，不能替代被检查模块的行为测试。
use super::super::super::super::*;

#[path = "../../../../../test_size/review/errors/structure/loader/budgets.rs"]
mod budgets;
#[path = "../../../../../review/errors/structure/loader/child_inventory.rs"]
mod child_inventory;
#[path = "../../../../../test_size/review/errors/structure/loader/delegation.rs"]
mod delegation;
#[path = "../../../../../test_size/review/errors/structure/loader/metadata.rs"]
mod metadata;
#[path = "../../../../../test_size/review/errors/structure/loader/routes.rs"]
mod routes;
#[path = "../../../../../review/errors/loader/source_helper_ownership.rs"]
mod source_helper_ownership;
#[path = "../../../../../test_size/review/errors/structure/loader/sources.rs"]
mod sources;

pub(super) use child_inventory::*;
pub(super) use metadata::*;
pub(super) use sources::*;

pub(super) fn assert_typed_error_native_plugin_loader_children_are_folder_backed() {
    let sources = typed_error_native_plugin_loader_sources();
    routes::assert_typed_error_native_plugin_loader_routes_are_folder_backed(&sources);
}

#[test]
fn runtime_15_typed_error_native_plugin_loader_structure_is_child_owner() {
    let sources = typed_error_native_plugin_loader_sources();

    delegation::assert_typed_error_native_plugin_loader_structure_is_child_owner(&sources);
    routes::assert_typed_error_native_plugin_loader_routes_are_folder_backed(&sources);
    budgets::assert_typed_error_native_plugin_loader_structure_budgets_are_focused(&sources);
}
