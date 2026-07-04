use super::super::super::super::super::*;
use super::super::*;

#[path = "status_current/budgets.rs"]
mod budgets;
#[path = "status_current/route_ownership.rs"]
mod route_ownership;
#[path = "status_current/status_mirrors.rs"]
mod status_mirrors;

pub(super) const TYPED_ERROR_SOURCE_INVENTORY_CHILD_INVENTORY_STATUS_CURRENT_CHILDREN: &[(
    &str,
    &str,
    &str,
)] = &[
    (
        "budgets",
        TYPED_ERROR_SOURCE_INVENTORY_CHILD_INVENTORY_STATUS_CURRENT_BUDGETS_CHILD,
        "pub(super) fn assert_typed_error_source_inventory_child_inventory_status_current_child_budgets_are_current",
    ),
    (
        "route_ownership",
        TYPED_ERROR_SOURCE_INVENTORY_CHILD_INVENTORY_STATUS_CURRENT_ROUTE_CHILD,
        "pub(super) fn assert_typed_error_source_inventory_child_inventory_status_current_is_child_backed",
    ),
    (
        "status_mirrors",
        TYPED_ERROR_SOURCE_INVENTORY_CHILD_INVENTORY_STATUS_CURRENT_MIRRORS_CHILD,
        "pub(super) fn assert_typed_error_source_inventory_child_inventory_status_current_mirrors_are_current",
    ),
];

pub(super) fn assert_typed_error_source_inventory_child_inventory_is_folder_backed() {
    route_ownership::assert_typed_error_source_inventory_child_inventory_status_current_is_child_backed();
    status_mirrors::assert_typed_error_source_inventory_child_inventory_status_current_mirrors_are_current();
    budgets::assert_typed_error_source_inventory_child_inventory_status_current_child_budgets_are_current();
}

#[test]
fn runtime_15_typed_error_source_inventory_child_inventory_status_current_is_child_backed() {
    assert_typed_error_source_inventory_child_inventory_is_folder_backed();
}
