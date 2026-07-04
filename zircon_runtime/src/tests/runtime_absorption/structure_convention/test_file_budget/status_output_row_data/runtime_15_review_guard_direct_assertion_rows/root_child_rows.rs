use super::*;

pub(super) const DIRECT_ASSERTION_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_direct_assertion_rows/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "export_chain",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_direct_assertion_rows/export_chain.rs",
        "runtime_15_review_guard_direct_assertion_export_chain_is_current",
    ),
    (
        "root_inventory",
        ROOT_INVENTORY_GUARD_PATH,
        ROOT_INVENTORY_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "row_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_direct_assertion_rows/row_ownership.rs",
        CHILD_OWNER_GUARD_NAME,
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_direct_assertion_rows/status_mirrors.rs",
        "runtime_15_review_guard_direct_assertion_folder_backed_status_mirrors_are_current",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_direct_assertion_rows/budgets.rs",
        "runtime_15_review_guard_direct_assertion_child_budgets_stay_focused",
    ),
];
