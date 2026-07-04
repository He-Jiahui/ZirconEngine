use super::*;

pub(super) const FOLDER_BACKED_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        P0_NATIVE_FIXTURE_DELEGATION_CHILD,
        FOLDER_BACKED_GUARD,
    ),
    (
        "leaf_ownership",
        P0_NATIVE_FIXTURE_LEAF_OWNERSHIP_CHILD,
        GUARD,
    ),
    (
        "status_mirrors",
        P0_NATIVE_FIXTURE_STATUS_MIRRORS_CHILD,
        FOLDER_BACKED_STATUS_GUARD,
    ),
    ("budgets", P0_NATIVE_FIXTURE_BUDGETS_CHILD, BUDGET_GUARD),
];

pub(super) const P0_NATIVE_FIXTURE_ROOT_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "root_paths",
        P0_NATIVE_FIXTURE_ROOT_PATHS_CHILD,
        "P0_NATIVE_FIXTURE_ROOT_PATHS_CHILD",
    ),
    (
        "root_statuses",
        P0_NATIVE_FIXTURE_ROOT_STATUSES_CHILD,
        P0_NATIVE_FIXTURE_ROOT_INVENTORY_STATUS,
    ),
    (
        "root_child_rows",
        P0_NATIVE_FIXTURE_ROOT_CHILD_ROWS_CHILD,
        "P0_NATIVE_FIXTURE_ROOT_CHILDREN",
    ),
    (
        "root_sources",
        P0_NATIVE_FIXTURE_ROOT_SOURCES_CHILD,
        "folder_backed_child_sources",
    ),
    (
        "root_inventory",
        P0_NATIVE_FIXTURE_ROOT_INVENTORY_CHILD,
        P0_NATIVE_FIXTURE_ROOT_INVENTORY_GUARD,
    ),
];
