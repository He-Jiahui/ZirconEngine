//! 约束基础稳健性审查的模块挂载、委托入口与既有检查保留；读取当前源码后按文本验证，不能替代被检查模块的行为测试。
use super::super::*;

#[path = "p0_owners/budgets.rs"]
mod budgets;
#[path = "p0_owners/delegation.rs"]
mod delegation;
#[path = "p0_owners/root_child_rows.rs"]
mod root_child_rows;
#[path = "p0_owners/root_inventory.rs"]
mod root_inventory;
#[path = "p0_owners/root_paths.rs"]
mod root_paths;
#[path = "p0_owners/root_sources.rs"]
mod root_sources;
#[path = "p0_owners/route_ownership.rs"]
mod route_ownership;

pub(super) use root_child_rows::*;
pub(super) use root_paths::*;
pub(super) use root_sources::*;

pub(super) const GUARD: &str = "runtime_15_p0_robustness_review_guards_are_child_owners";
pub(super) const FOLDER_BACKED_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        P0_DELEGATION_CHILD,
        "runtime_15_p0_robustness_structure_guard_is_folder_backed",
    ),
    ("route_ownership", P0_ROUTE_OWNERSHIP_CHILD, GUARD),
    (
        "budgets",
        P0_BUDGETS_CHILD,
        "runtime_15_p0_robustness_structure_guard_budgets_are_focused",
    ),
];

pub(super) fn assert_p0_robustness_child_owners_are_folder_backed() {
    route_ownership::assert_p0_robustness_child_owners_are_folder_backed();
}
