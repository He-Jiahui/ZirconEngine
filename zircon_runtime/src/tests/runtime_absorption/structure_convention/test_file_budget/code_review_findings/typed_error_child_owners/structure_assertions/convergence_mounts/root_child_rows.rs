use super::*;

pub(super) const TYPED_ERROR_CONVERGENCE_MOUNT_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "top_level",
        TYPED_ERROR_CONVERGENCE_MOUNTS_TOP_LEVEL_CHILD,
        "assert_typed_error_convergence_top_level_parent_is_folder_backed",
    ),
    (
        "asset_parents",
        TYPED_ERROR_CONVERGENCE_MOUNTS_ASSET_PARENTS_CHILD,
        "assert_typed_error_asset_parents_are_folder_backed",
    ),
    (
        "runtime_parents",
        TYPED_ERROR_CONVERGENCE_MOUNTS_RUNTIME_PARENTS_CHILD,
        "assert_typed_error_runtime_parents_are_folder_backed",
    ),
    (
        "budgets",
        TYPED_ERROR_CONVERGENCE_MOUNTS_BUDGETS_CHILD,
        "assert_typed_error_convergence_mount_budgets_are_focused",
    ),
    (
        "status_mirrors",
        TYPED_ERROR_CONVERGENCE_MOUNTS_STATUS_MIRRORS_CHILD,
        TYPED_ERROR_CONVERGENCE_MOUNTS_STATUS_GUARD,
    ),
];

pub(super) const TYPED_ERROR_CONVERGENCE_MOUNT_ROOT_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "root_paths",
        TYPED_ERROR_CONVERGENCE_MOUNTS_ROOT_PATHS_CHILD,
        "TYPED_ERROR_CONVERGENCE_MOUNTS_ROOT_PATHS_CHILD",
    ),
    (
        "root_statuses",
        TYPED_ERROR_CONVERGENCE_MOUNTS_ROOT_STATUSES_CHILD,
        TYPED_ERROR_CONVERGENCE_MOUNTS_ROOT_INVENTORY_STATUS,
    ),
    (
        "root_child_rows",
        TYPED_ERROR_CONVERGENCE_MOUNTS_ROOT_CHILD_ROWS_CHILD,
        "TYPED_ERROR_CONVERGENCE_MOUNT_ROOT_CHILDREN",
    ),
    (
        "root_sources",
        TYPED_ERROR_CONVERGENCE_MOUNTS_ROOT_SOURCES_CHILD,
        "typed_error_convergence_mount_sources",
    ),
    (
        "root_inventory",
        TYPED_ERROR_CONVERGENCE_MOUNTS_ROOT_INVENTORY_CHILD,
        TYPED_ERROR_CONVERGENCE_MOUNTS_ROOT_INVENTORY_GUARD,
    ),
];
