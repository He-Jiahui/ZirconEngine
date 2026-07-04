use super::*;

pub(super) const CODE_REVIEW_ROWS_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "row_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/row_ownership.rs",
        CODE_REVIEW_ROWS_ROW_DATA_GUARD_NAME,
    ),
    (
        "root_and_children",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/root_and_children.rs",
        ROOT_AND_CHILDREN_ROW_DATA_GUARD_NAME,
    ),
    (
        "root_inventory",
        ROOT_INVENTORY_GUARD_PATH,
        ROOT_INVENTORY_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "plugin_importer_rows",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows.rs",
        PLUGIN_IMPORTER_STATUS_OUTPUT_GUARD_GUARD_NAME,
    ),
    (
        "export_chain",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/export_chain.rs",
        "runtime_15_review_guard_code_review_row_exports_are_current",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/budgets.rs",
        "runtime_15_review_guard_code_review_rows_child_budgets_stay_focused",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/status_mirrors.rs",
        "runtime_15_review_guard_code_review_rows_status_mirrors_are_current",
    ),
];

pub(super) const CODE_REVIEW_CHILD_EXPORTS: &[&str] = &[
    "CODE_REVIEW_EXPECTED_STATUS_OUTPUT_SLICES",
    "CODE_REVIEW_DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES",
    "CODE_REVIEW_PLUGIN_IMPORTER_EXPECTED_STATUS_OUTPUT_SLICES",
    "CODE_REVIEW_STRUCTURE_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
    "CODE_REVIEW_STRUCTURE_GUARD_STATUS_DOCS_EXPECTED_STATUS_OUTPUT_SLICES",
    "CODE_REVIEW_STRUCTURE_GUARD_FOLDER_BACKED_SUMMARY_EXPECTED_STATUS_OUTPUT_SLICES",
    "CODE_REVIEW_STRUCTURE_GUARD_TYPED_ERROR_EXPECTED_STATUS_OUTPUT_SLICES",
    "CODE_REVIEW_STRUCTURE_GUARD_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
    "CODE_REVIEW_TYPED_ERROR_STRUCTURE_EXPECTED_STATUS_OUTPUT_SLICES",
    "CODE_REVIEW_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
];
