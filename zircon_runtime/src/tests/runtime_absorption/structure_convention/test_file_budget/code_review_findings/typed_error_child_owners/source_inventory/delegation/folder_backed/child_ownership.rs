use super::super::super::super::super::super::*;
use super::super::super::*;

#[path = "child_ownership/budgets.rs"]
mod budgets;
#[path = "child_ownership/delegation_parent.rs"]
mod delegation_parent;
#[path = "child_ownership/folder_backed_parent.rs"]
mod folder_backed_parent;
#[path = "child_ownership/route_ownership.rs"]
mod route_ownership;
#[path = "child_ownership/status_current.rs"]
mod status_current;

pub(super) fn assert_typed_error_source_inventory_delegation_is_child_backed() {
    delegation_parent::assert_typed_error_source_inventory_delegation_is_child_backed();
}

#[test]
fn runtime_15_typed_error_source_inventory_delegation_folder_backed_is_child_owned() {
    folder_backed_parent::assert_typed_error_source_inventory_delegation_folder_backed_is_child_owned();
}

#[test]
fn runtime_15_typed_error_source_inventory_delegation_folder_backed_ownership_is_child_backed() {
    route_ownership::assert_typed_error_source_inventory_delegation_folder_backed_ownership_is_child_backed();
    status_current::assert_typed_error_source_inventory_delegation_folder_backed_ownership_status_is_current();
    budgets::assert_typed_error_source_inventory_delegation_folder_backed_ownership_child_budgets_are_current();
}
