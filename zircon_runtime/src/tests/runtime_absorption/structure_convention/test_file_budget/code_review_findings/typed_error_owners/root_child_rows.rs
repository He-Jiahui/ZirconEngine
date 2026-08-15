use super::*;

pub(super) const FOLDER_BACKED_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        TYPED_ERROR_TOP_LEVEL_DELEGATION_CHILD,
        "runtime_15_typed_error_structure_guard_is_folder_backed",
    ),
    (
        "child_ownership",
        TYPED_ERROR_TOP_LEVEL_CHILD_OWNERSHIP_CHILD,
        "runtime_15_code_review_findings_typed_error_structure_guard_is_child_owner",
    ),
    (
        "source_inventory",
        TYPED_ERROR_SOURCE_INVENTORY_CHILD,
        "runtime_15_typed_error_source_inventory_guard_is_folder_backed",
    ),
    (
        "structure_assertions",
        TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD,
        "pub(super) fn assert_typed_error_child_owners_are_folder_backed",
    ),
    (
        "budgets",
        TYPED_ERROR_TOP_LEVEL_BUDGETS_CHILD,
        "runtime_15_typed_error_structure_guard_budgets_are_focused",
    ),
];

pub(super) const FOLDER_BACKED_CHILD_PATH_AUDIT: &[&str] = &[
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/delegation.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/child_ownership.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/source_inventory.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure_assertions.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/budgets.rs",
];
