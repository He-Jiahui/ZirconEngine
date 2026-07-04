use super::super::super::super::super::*;
use super::super::*;

#[path = "folder_backed/child_ownership.rs"]
mod child_ownership;
#[path = "folder_backed/guard_body.rs"]
mod guard_body;
#[path = "folder_backed/status_current.rs"]
mod status_current;

pub(super) fn assert_typed_error_source_inventory_guard_is_folder_backed() {
    guard_body::assert_typed_error_source_inventory_guard_is_folder_backed();
}

#[test]
fn runtime_15_typed_error_source_inventory_delegation_is_child_backed() {
    child_ownership::assert_typed_error_source_inventory_delegation_is_child_backed();
}
