use super::super::*;

#[path = "structure_guard/budgets.rs"]
mod budgets;
#[path = "structure_guard/route_ownership.rs"]
mod route_ownership;
#[path = "structure_guard/status_mirrors.rs"]
mod status_mirrors;

pub(in super::super) const TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STRUCTURE_GUARD_CHILDREN: &[(
    &str,
    &str,
    &str,
)] = &[
    (
        "route_ownership",
        TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STRUCTURE_GUARD_ROUTE_CHILD,
        "pub(in super::super) fn assert_typed_error_source_inventory_child_sources_structure_guard_is_child_backed",
    ),
    (
        "status_mirrors",
        TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STRUCTURE_GUARD_STATUS_MIRRORS_CHILD,
        "pub(in super::super) fn assert_typed_error_source_inventory_child_sources_structure_guard_status_is_current",
    ),
    (
        "budgets",
        TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STRUCTURE_GUARD_BUDGETS_CHILD,
        "pub(in super::super) fn assert_typed_error_source_inventory_child_sources_structure_guard_budgets_are_current",
    ),
];

pub(in super::super) fn assert_typed_error_source_inventory_child_sources_are_folder_backed() {
    route_ownership::assert_typed_error_source_inventory_child_sources_structure_guard_is_child_backed();
    status_mirrors::assert_typed_error_source_inventory_child_sources_structure_guard_status_is_current();
    budgets::assert_typed_error_source_inventory_child_sources_structure_guard_budgets_are_current(
    );
}

#[test]
fn runtime_15_typed_error_source_inventory_child_sources_structure_guard_is_child_backed() {
    assert_typed_error_source_inventory_child_sources_are_folder_backed();
}
