use super::*;

pub(super) const DIRECT_ASSERTION_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "export_chain",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/export_chain.rs",
        EXPORT_CHAIN_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "root_inventory",
        ROOT_INVENTORY_GUARD_PATH,
        ROOT_INVENTORY_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "row_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/row_ownership.rs",
        CHILD_OWNER_GUARD_NAME,
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/status_mirrors.rs",
        "runtime_15_review_guard_direct_assertion_folder_backed_status_mirrors_are_current",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/budgets.rs",
        "runtime_15_review_guard_direct_assertion_child_budgets_stay_focused",
    ),
];

pub(super) const DIRECT_ASSERTION_ROW_DATA_CHILD_ROWS: &[(&str, &str, &str)] = &[
    (
        "core_rows",
        DIRECT_ASSERTION_CORE_ROWS_PATH,
        "Runtime 15 M3 code review findings direct assertions child-owner split",
    ),
    (
        "f12_rows",
        DIRECT_ASSERTION_F12_ROWS_PATH,
        "Runtime 15 M3 code review findings F12 direct assertions child-owner split",
    ),
    (
        "root_parent_rows",
        DIRECT_ASSERTION_ROOT_PARENT_ROWS_PATH,
        "Runtime 15 M3 code review findings root-parent direct assertions child-owner split",
    ),
    (
        "render_rows",
        DIRECT_ASSERTION_RENDER_ROWS_PATH,
        "Runtime 15 M3 code review findings render direct assertions child-owner split",
    ),
    (
        "f8_rows",
        DIRECT_ASSERTION_F8_ROWS_PATH,
        "Runtime 15 M3 code review findings F8 direct assertions child-owner split",
    ),
    (
        "p0_rows",
        DIRECT_ASSERTION_P0_ROWS_PATH,
        "Runtime 15 M3 code review findings P0 direct assertions child-owner split",
    ),
    (
        "row_data_owner_rows",
        DIRECT_ASSERTION_ROW_DATA_OWNER_ROWS_PATH,
        ROW_DATA_FOLDER_BACKED_STATUS_NAME,
    ),
];

pub(super) const DIRECT_ASSERTION_ROW_OWNERSHIP_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "child_owner_rows",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/row_ownership/child_owner_rows.rs",
        CHILD_OWNER_GUARD_NAME,
    ),
    (
        "folder_backed_rows",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/row_ownership/folder_backed_rows.rs",
        ROW_DATA_FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "status_current",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/row_ownership/status_current.rs",
        "runtime_15_review_guard_direct_assertion_row_ownership_status_is_current",
    ),
];

pub(super) const DIRECT_ASSERTION_EXPORT_CHAIN_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "review_guard_row_data",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/export_chain/review_guard_row_data.rs",
        "runtime_15_review_guard_direct_assertion_export_chain_row_data_aggregation_is_current",
    ),
    (
        "code_review_rows",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/export_chain/code_review_rows.rs",
        "runtime_15_review_guard_direct_assertion_export_chain_code_review_rows_are_current",
    ),
    (
        "review_guard_splits",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/export_chain/review_guard_splits.rs",
        "runtime_15_review_guard_direct_assertion_export_chain_review_guard_splits_are_current",
    ),
    (
        "runtime_aggregation",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/export_chain/runtime_aggregation.rs",
        "runtime_15_review_guard_direct_assertion_export_chain_runtime_aggregation_is_current",
    ),
    (
        "status_current",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/export_chain/status_current.rs",
        "runtime_15_review_guard_direct_assertion_export_chain_status_is_current",
    ),
];

pub(super) const DIRECT_ASSERTION_ROW_OWNERSHIP_FOLDER_BACKED_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "exports",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/row_ownership/folder_backed_rows/exports.rs",
        "runtime_15_review_guard_direct_assertion_row_data_exports_are_child_owned",
    ),
    (
        "status_maps",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/row_ownership/folder_backed_rows/status_maps.rs",
        "runtime_15_review_guard_direct_assertion_row_data_status_maps_are_child_owned",
    ),
];
