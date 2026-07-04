use super::*;

pub(super) const FOLDER_BACKED_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        TYPED_ERROR_TOP_LEVEL_DELEGATION_CHILD,
        FOLDER_BACKED_GUARD,
    ),
    (
        "child_ownership",
        TYPED_ERROR_TOP_LEVEL_CHILD_OWNERSHIP_CHILD,
        GUARD,
    ),
    (
        "source_inventory",
        TYPED_ERROR_SOURCE_INVENTORY_CHILD,
        "runtime_15_typed_error_source_inventory_is_child_owner",
    ),
    (
        "status_docs",
        TYPED_ERROR_STATUS_DOCS_CHILD,
        "runtime_15_typed_error_status_docs_are_folder_backed",
    ),
    (
        "structure_assertions",
        TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD,
        "pub(super) fn assert_typed_error_child_owners_are_folder_backed",
    ),
    (
        "status_mirrors",
        TYPED_ERROR_TOP_LEVEL_STATUS_MIRRORS_CHILD,
        FOLDER_BACKED_STATUS_GUARD,
    ),
    ("budgets", TYPED_ERROR_TOP_LEVEL_BUDGETS_CHILD, BUDGET_GUARD),
];

pub(super) const TYPED_ERROR_ROOT_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "root_paths",
        TYPED_ERROR_ROOT_PATHS_CHILD,
        "TYPED_ERROR_ROOT_PATHS_CHILD",
    ),
    (
        "root_statuses",
        TYPED_ERROR_ROOT_STATUSES_CHILD,
        TYPED_ERROR_ROOT_INVENTORY_STATUS,
    ),
    (
        "root_child_rows",
        TYPED_ERROR_ROOT_CHILD_ROWS_CHILD,
        "TYPED_ERROR_ROOT_CHILDREN",
    ),
    (
        "root_sources",
        TYPED_ERROR_ROOT_SOURCES_CHILD,
        "typed_error_structure_status_row_source",
    ),
    (
        "root_inventory",
        TYPED_ERROR_ROOT_INVENTORY_CHILD,
        TYPED_ERROR_ROOT_INVENTORY_GUARD,
    ),
];
