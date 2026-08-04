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
pub(super) use root_statuses::*;

pub(super) fn assert_p0_robustness_child_owners_are_folder_backed() {
    route_ownership::assert_p0_robustness_child_owners_are_folder_backed();
}
