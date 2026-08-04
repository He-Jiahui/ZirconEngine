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
pub(super) use root_statuses::*;

pub(super) fn assert_late_api_cleanup_child_owners_are_folder_backed() {
    route_ownership::assert_late_api_cleanup_child_owners_are_folder_backed();
}
