use super::super::*;

pub(in super::super) const TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_CHILDREN: &[(
    &str,
    &str,
    &str,
)] = &[
    (
        "parent_delegation",
        TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_PARENT_CHILD,
        "pub(super) fn assert_typed_error_structure_delegates_source_inventory",
    ),
    (
        "source_inventory_mounts",
        TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_SOURCE_INVENTORY_MOUNTS_CHILD,
        "pub(super) fn assert_typed_error_source_inventory_parent_mounts_focused_owners",
    ),
    (
        "source_ownership",
        TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_SOURCE_OWNERSHIP_CHILD,
        "pub(super) fn assert_typed_error_source_inventory_paths_and_reads_are_child_owned",
    ),
];
