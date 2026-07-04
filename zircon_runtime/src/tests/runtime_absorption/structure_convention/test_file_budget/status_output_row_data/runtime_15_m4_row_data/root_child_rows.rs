use super::*;

pub(super) const RUNTIME_15_M4_ROW_DATA_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "row_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data/row_ownership.rs",
        ROW_DATA_SPLIT_GUARD_NAME,
    ),
    (
        "root_inventory",
        ROOT_INVENTORY_GUARD_PATH,
        ROOT_INVENTORY_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data/status_mirrors.rs",
        "runtime_15_m4_row_data_guard_folder_backed_status_mirrors_are_current",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data/budgets.rs",
        "runtime_15_m4_row_data_guard_children_stay_focused",
    ),
];
