use super::super::*;

pub(in super::super) const TYPED_ERROR_SOURCE_INVENTORY_SOURCE_HELPER_CHILDREN: &[(
    &str,
    &str,
    &str,
)] = &[
    (
        "child_sources",
        TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_CHILD,
        "TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_CHILDREN",
    ),
    (
        "child_inventory",
        TYPED_ERROR_SOURCE_INVENTORY_CHILD_INVENTORY_CHILD,
        "pub(super) use source_helper_children::TYPED_ERROR_SOURCE_INVENTORY_SOURCE_HELPER_CHILDREN;",
    ),
    (
        "metadata",
        TYPED_ERROR_SOURCE_INVENTORY_METADATA_CHILD,
        "TYPED_ERROR_SOURCE_INVENTORY_METADATA_CHILDREN",
    ),
    (
        "source_helper_ownership",
        TYPED_ERROR_SOURCE_INVENTORY_SOURCE_HELPER_OWNERSHIP_CHILD,
        TYPED_ERROR_SOURCE_INVENTORY_SOURCE_HELPER_OWNERSHIP_GUARD,
    ),
    (
        "source_helper_status",
        TYPED_ERROR_SOURCE_INVENTORY_SOURCE_HELPER_STATUS_CHILD,
        TYPED_ERROR_SOURCE_INVENTORY_SOURCE_HELPER_STATUS_GUARD,
    ),
];
