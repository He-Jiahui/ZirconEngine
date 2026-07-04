use super::super::*;

pub(super) const TYPED_ERROR_SOURCE_INVENTORY_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "paths",
        TYPED_ERROR_SOURCE_INVENTORY_PATHS_CHILD,
        "const TYPED_ERROR_SOURCE_PATHS",
    ),
    (
        "reads",
        TYPED_ERROR_SOURCE_INVENTORY_READS_CHILD,
        "pub(super) fn typed_error_children_source",
    ),
    (
        "budgets",
        TYPED_ERROR_SOURCE_INVENTORY_BUDGETS_CHILD,
        TYPED_ERROR_SOURCE_INVENTORY_BUDGET_GUARD,
    ),
    (
        "delegation",
        TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_CHILD,
        TYPED_ERROR_SOURCE_INVENTORY_FOLDER_BACKED_GUARD,
    ),
    (
        "status_mirrors",
        TYPED_ERROR_SOURCE_INVENTORY_STATUS_MIRRORS_CHILD,
        TYPED_ERROR_SOURCE_INVENTORY_STATUS_GUARD,
    ),
];
