use super::super::super::super::*;

#[path = "moved_guard_absence/budgets.rs"]
mod budgets;
#[path = "moved_guard_absence/child_ownership.rs"]
mod child_ownership;
#[path = "moved_guard_absence/child_ownership_status.rs"]
mod child_ownership_status;
#[path = "moved_guard_absence/parent_backflow.rs"]
mod parent_backflow;
#[path = "moved_guard_absence/path_anchors.rs"]
mod path_anchors;
#[path = "moved_guard_absence/preserved_guards.rs"]
mod preserved_guards;
#[path = "moved_guard_absence/root_child_rows.rs"]
mod root_child_rows;
#[path = "moved_guard_absence/root_inventory.rs"]
mod root_inventory;
#[path = "moved_guard_absence/root_paths.rs"]
mod root_paths;
#[path = "moved_guard_absence/root_sources.rs"]
mod root_sources;
#[path = "moved_guard_absence/root_statuses.rs"]
mod root_statuses;
#[path = "moved_guard_absence/status_mirrors.rs"]
mod status_mirrors;

pub(super) use root_child_rows::*;
pub(super) use root_paths::*;
pub(super) use root_sources::*;
pub(super) use root_statuses::*;

pub(super) fn assert_typed_error_moved_guards_stay_child_owned() {
    preserved_guards::assert_typed_error_preserved_review_guards_are_current();
    parent_backflow::assert_typed_error_parent_backflow_guards_are_absent();
}
