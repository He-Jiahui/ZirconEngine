use super::super::*;

#[path = "p0_native_fixture_leaf_owners/budgets.rs"]
mod budgets;
#[path = "p0_native_fixture_leaf_owners/delegation.rs"]
mod delegation;
#[path = "p0_native_fixture_leaf_owners/leaf_ownership.rs"]
mod leaf_ownership;
#[path = "p0_native_fixture_leaf_owners/root_child_rows.rs"]
mod root_child_rows;
#[path = "p0_native_fixture_leaf_owners/root_inventory.rs"]
mod root_inventory;
#[path = "p0_native_fixture_leaf_owners/root_paths.rs"]
mod root_paths;
#[path = "p0_native_fixture_leaf_owners/root_sources.rs"]
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
