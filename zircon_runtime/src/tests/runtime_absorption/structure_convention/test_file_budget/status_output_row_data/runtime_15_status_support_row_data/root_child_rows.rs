use super::*;

pub(super) const STATUS_SUPPORT_ROW_DATA_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_status_support_row_data/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "row_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_status_support_row_data/row_ownership.rs",
        CHILD_OWNER_GUARD_NAME,
    ),
    (
        "row_data_and_budget",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_status_support_row_data/row_data_and_budget.rs",
        ROW_DATA_AND_BUDGET_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "root_inventory",
        ROOT_INVENTORY_GUARD_PATH,
        ROOT_INVENTORY_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "export_chain",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_status_support_row_data/export_chain.rs",
        "runtime_15_status_support_row_data_export_chain_is_current",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_status_support_row_data/status_mirrors.rs",
        "runtime_15_status_support_row_data_folder_backed_status_mirrors_are_current",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_status_support_row_data/budgets.rs",
        "runtime_15_status_support_row_data_child_budgets_stay_focused",
    ),
];

pub(super) const ROW_DATA_AND_BUDGET_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "test_file_budget",
        STATUS_SUPPORT_ROW_DATA_TEST_FILE_BUDGET_PATH,
        "Runtime 15 M3 test file budget root-layout child split",
    ),
    (
        "runtime_row_data",
        STATUS_SUPPORT_ROW_DATA_RUNTIME_ROW_DATA_PATH,
        "Runtime 15 M3 status output Runtime 15 row data split",
    ),
    (
        "hub_editor_support",
        STATUS_SUPPORT_ROW_DATA_HUB_EDITOR_SUPPORT_PATH,
        "Runtime 15 M3 support Hub project-actions tests child-owner split",
    ),
    (
        "render_shader_support",
        STATUS_SUPPORT_ROW_DATA_RENDER_SHADER_SUPPORT_PATH,
        "Runtime 15 M3 render shader template assembly guard support child-owner split",
    ),
    (
        "m3_m4_row_data",
        STATUS_SUPPORT_ROW_DATA_M3_M4_ROW_DATA_PATH,
        "Runtime 15 M3 Runtime 15 M4 row-data guard folder-backed split",
    ),
];
