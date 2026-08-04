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
];

pub(super) const TYPED_ERROR_CONVERGENCE_MOUNT_CHILD_PATH_AUDIT: &[&str] = &[
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/convergence_mounts/top_level.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/convergence_mounts/asset_parents.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/convergence_mounts/runtime_parents.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/convergence_mounts/budgets.rs",
];

pub(super) const TYPED_ERROR_CONVERGENCE_MOUNT_ROOT_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "root_paths",
        TYPED_ERROR_CONVERGENCE_MOUNTS_ROOT_PATHS_CHILD,
        "TYPED_ERROR_CONVERGENCE_MOUNTS_ROOT_PATHS_CHILD",
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
];
