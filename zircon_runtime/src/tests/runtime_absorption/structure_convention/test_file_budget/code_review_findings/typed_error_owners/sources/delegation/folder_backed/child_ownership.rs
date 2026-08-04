use super::super::super::super::super::super::*;
use super::super::super::*;

#[path = "ownership/budgets.rs"]
mod budgets;
#[path = "ownership/delegation_parent.rs"]
mod delegation_parent;
#[path = "ownership/folder_backed_parent.rs"]
mod folder_backed_parent;
#[path = "ownership/route_ownership.rs"]
mod route_ownership;

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
    budgets::assert_typed_error_source_inventory_delegation_folder_backed_ownership_child_budgets_are_current();
}
