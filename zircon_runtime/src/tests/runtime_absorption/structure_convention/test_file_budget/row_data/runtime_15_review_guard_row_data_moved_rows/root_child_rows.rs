use super::*;

pub(super) const MOVED_ROWS_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/delegation.rs",
        "runtime_15_status_output_m3_review_guard_row_data_moved_rows_are_child_owner",
    ),
    (
        "code_review_rows",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows.rs",
        "runtime_15_review_guard_moved_row_code_review_rows_are_child_owned",
    ),
    (
        "typed_error_rows",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/typed_error_rows.rs",
        "runtime_15_review_guard_moved_row_typed_error_rows_are_child_owned",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/status_mirrors.rs",
        "runtime_15_review_guard_moved_row_folder_backed_status_mirrors_are_current",
    ),
    (
        "root_inventory",
        ROOT_INVENTORY_GUARD_PATH,
        ROOT_INVENTORY_CHILD_SPLIT_GUARD_NAME,
    ),
];
