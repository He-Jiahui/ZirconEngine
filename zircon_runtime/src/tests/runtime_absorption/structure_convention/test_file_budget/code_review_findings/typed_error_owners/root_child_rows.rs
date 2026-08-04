use super::*;

pub(super) const FOLDER_BACKED_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "child_ownership",
        TYPED_ERROR_TOP_LEVEL_CHILD_OWNERSHIP_CHILD,
        GUARD,
    ),
    (
        "structure_assertions",
        TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD,
        "pub(super) fn assert_typed_error_child_owners_are_folder_backed",
    ),
];

pub(super) const FOLDER_BACKED_CHILD_PATH_AUDIT: &[&str] = &[
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/delegation.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/child_ownership.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/source_inventory.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure_assertions.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/budgets.rs",
];
