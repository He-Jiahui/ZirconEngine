use super::*;

pub(super) const TYPED_ERROR_NATIVE_STRUCTURE_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "budgets",
        TYPED_ERROR_NATIVE_STRUCTURE_BUDGETS_CHILD,
        "assert_typed_error_native_plugin_loader_structure_budgets_are_focused",
    ),
    (
        "delegation",
        TYPED_ERROR_NATIVE_STRUCTURE_DELEGATION_CHILD,
        TYPED_ERROR_NATIVE_STRUCTURE_FOLDER_BACKED_GUARD,
    ),
    (
        "routes",
        TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_CHILD,
        "assert_typed_error_native_plugin_loader_routes_are_folder_backed",
    ),
];

pub(super) const TYPED_ERROR_NATIVE_STRUCTURE_SOURCE_HELPER_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "child_inventory",
        TYPED_ERROR_NATIVE_STRUCTURE_CHILD_INVENTORY_CHILD,
        "TYPED_ERROR_NATIVE_STRUCTURE_SOURCE_HELPER_CHILDREN",
    ),
    (
        "metadata",
        TYPED_ERROR_NATIVE_STRUCTURE_METADATA_CHILD,
        TYPED_ERROR_NATIVE_STRUCTURE_SOURCE_HELPER_SLICE,
    ),
    (
        "source_helper_ownership",
        TYPED_ERROR_NATIVE_STRUCTURE_SOURCE_HELPER_OWNERSHIP_CHILD,
        TYPED_ERROR_NATIVE_STRUCTURE_SOURCE_HELPER_OWNERSHIP_GUARD,
    ),
    (
        "sources",
        TYPED_ERROR_NATIVE_STRUCTURE_SOURCES_CHILD,
        "pub(super) fn typed_error_native_plugin_loader_child_source_blob",
    ),
];
