use super::super::*;

#[path = "late_api_cleanup_owners/budgets.rs"]
mod budgets;
#[path = "late_api_cleanup_owners/delegation.rs"]
mod delegation;
#[path = "late_api_cleanup_owners/root_child_rows.rs"]
mod root_child_rows;
#[path = "late_api_cleanup_owners/root_inventory.rs"]
mod root_inventory;
#[path = "late_api_cleanup_owners/root_paths.rs"]
mod root_paths;
#[path = "late_api_cleanup_owners/root_sources.rs"]
mod root_sources;
#[path = "late_api_cleanup_owners/route_ownership.rs"]
mod route_ownership;

pub(super) use root_child_rows::*;
pub(super) use root_paths::*;
pub(super) use root_sources::*;

pub(super) const GUARD: &str = "runtime_15_late_api_cleanup_review_guards_are_child_owners";
pub(super) const FOLDER_BACKED_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        LATE_API_CLEANUP_DELEGATION_CHILD,
        "runtime_15_late_api_cleanup_structure_guard_is_folder_backed",
    ),
    (
        "route_ownership",
        LATE_API_CLEANUP_ROUTE_OWNERSHIP_CHILD,
        GUARD,
    ),
    (
        "budgets",
        LATE_API_CLEANUP_BUDGETS_CHILD,
        "runtime_15_late_api_cleanup_structure_guard_budgets_are_focused",
    ),
];

pub(super) fn assert_late_api_cleanup_child_owners_are_folder_backed() {
    route_ownership::assert_late_api_cleanup_child_owners_are_folder_backed();
}
