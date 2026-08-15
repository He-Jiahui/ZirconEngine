use super::super::*;

#[path = "f8_owners/budgets.rs"]
mod budgets;
#[path = "f8_owners/delegation.rs"]
mod delegation;
#[path = "f8_owners/root_child_rows.rs"]
mod root_child_rows;
#[path = "f8_owners/root_inventory.rs"]
mod root_inventory;
#[path = "f8_owners/root_paths.rs"]
mod root_paths;
#[path = "f8_owners/root_sources.rs"]
mod root_sources;
#[path = "f8_owners/route_ownership.rs"]
mod route_ownership;

pub(super) use root_child_rows::*;
pub(super) use root_paths::*;
pub(super) use root_sources::*;

pub(super) const GUARD: &str = "runtime_15_f8_api_convergence_review_guards_are_child_owners";
pub(super) const FOLDER_BACKED_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        F8_DELEGATION_CHILD,
        "runtime_15_f8_child_owner_structure_guard_is_folder_backed",
    ),
    ("route_ownership", F8_ROUTE_OWNERSHIP_CHILD, GUARD),
    (
        "budgets",
        F8_BUDGETS_CHILD,
        "runtime_15_f8_child_owner_structure_guard_budgets_are_focused",
    ),
];
