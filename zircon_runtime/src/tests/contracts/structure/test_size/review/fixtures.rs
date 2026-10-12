//! 为原生夹具审查挂载专属检查与共享清单；此入口只划分职责，实际断言由子模块及测试框架承担。
use super::super::*;

#[path = "../../../../structure/test_size/review/fixtures/budgets.rs"]
mod budgets;
#[path = "../../../../structure/test_size/review/fixtures/delegation.rs"]
mod delegation;
#[path = "../../../../structure/test_size/review/fixtures/leaf_ownership.rs"]
mod leaf_ownership;
#[path = "../../../../structure/test_size/review/fixtures/root_child_rows.rs"]
mod root_child_rows;
#[path = "../../../../structure/test_size/review/fixtures/root_inventory.rs"]
mod root_inventory;
#[path = "../../../../structure/test_size/review/fixtures/root_paths.rs"]
mod root_paths;
#[path = "../../../../structure/test_size/review/fixtures/root_sources.rs"]
mod root_sources;

pub(super) use root_child_rows::*;
pub(super) use root_paths::*;
pub(super) use root_sources::*;

pub(super) const GUARD: &str = "runtime_15_p0_native_fixture_review_guards_are_leaf_owners";
pub(super) const FOLDER_BACKED_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        P0_NATIVE_FIXTURE_DELEGATION_CHILD,
        "runtime_15_p0_native_fixture_leaf_owner_guard_is_folder_backed",
    ),
    (
        "leaf_ownership",
        P0_NATIVE_FIXTURE_LEAF_OWNERSHIP_CHILD,
        GUARD,
    ),
    (
        "budgets",
        P0_NATIVE_FIXTURE_BUDGETS_CHILD,
        "runtime_15_p0_native_fixture_leaf_owner_guard_budgets_are_focused",
    ),
];
