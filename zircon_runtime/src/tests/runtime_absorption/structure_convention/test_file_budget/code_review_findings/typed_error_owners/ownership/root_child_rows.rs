use super::*;

pub(super) const TYPED_ERROR_CHILD_OWNERSHIP_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "budgets",
        TYPED_ERROR_CHILD_OWNERSHIP_BUDGETS_CHILD,
        "assert_typed_error_child_ownership_budgets_are_focused",
    ),
    (
        "delegation",
        TYPED_ERROR_CHILD_OWNERSHIP_DELEGATION_CHILD,
        TYPED_ERROR_CHILD_OWNERSHIP_FOLDER_BACKED_GUARD,
    ),
    (
        "review_guards",
        TYPED_ERROR_CHILD_OWNERSHIP_REVIEW_GUARDS_CHILD,
        "assert_typed_error_review_guards_are_preserved",
    ),
    (
        "status_mirrors",
        TYPED_ERROR_CHILD_OWNERSHIP_STATUS_MIRRORS_CHILD,
        TYPED_ERROR_CHILD_OWNERSHIP_STATUS_GUARD,
    ),
    (
        "structure_subtree",
        TYPED_ERROR_CHILD_OWNERSHIP_STRUCTURE_SUBTREE_CHILD,
        "assert_typed_error_structure_subtree_is_child_owned",
    ),
];

pub(super) const TYPED_ERROR_CHILD_OWNERSHIP_CHILD_PATH_AUDIT: &[&str] = &[
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/ownership/budgets.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/ownership/delegation.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/ownership/review_guards.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/ownership/status_mirrors.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/ownership/structure_subtree.rs",
];

pub(super) const TYPED_ERROR_CHILD_OWNERSHIP_ROOT_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "root_paths",
        TYPED_ERROR_CHILD_OWNERSHIP_ROOT_PATHS_CHILD,
        "TYPED_ERROR_CHILD_OWNERSHIP_ROOT_PATHS_CHILD",
    ),
    (
        "root_statuses",
        TYPED_ERROR_CHILD_OWNERSHIP_ROOT_STATUSES_CHILD,
        TYPED_ERROR_CHILD_OWNERSHIP_ROOT_INVENTORY_STATUS,
    ),
    (
        "root_child_rows",
        TYPED_ERROR_CHILD_OWNERSHIP_ROOT_CHILD_ROWS_CHILD,
        "TYPED_ERROR_CHILD_OWNERSHIP_ROOT_CHILDREN",
    ),
    (
        "root_sources",
        TYPED_ERROR_CHILD_OWNERSHIP_ROOT_SOURCES_CHILD,
        "typed_error_child_ownership_sources",
    ),
    (
        "root_inventory",
        TYPED_ERROR_CHILD_OWNERSHIP_ROOT_INVENTORY_CHILD,
        TYPED_ERROR_CHILD_OWNERSHIP_ROOT_INVENTORY_GUARD,
    ),
];
