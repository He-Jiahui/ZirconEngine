use super::*;

pub(super) const M3_CHILD_GROUP_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/budgets.rs",
        "runtime_15_m3_child_groups_row_data_guard_children_stay_focused",
    ),
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "exports",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/exports.rs",
        HISTORICAL_CHILD_OWNER_GUARD_NAME,
    ),
    (
        "module_convention_status",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/module_convention_status.rs",
        "runtime_15_module_convention_status_row_data_owner_is_child_backed",
    ),
    (
        "production_guard_support",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/production_guard_support.rs",
        PRODUCTION_GUARD_SUPPORT_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "production_guard_support_inventory_row_data",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/production_guard_support/inventory_row_data.rs",
        INVENTORY_ROW_DATA_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "production_guard_support_row_data_children",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/production_guard_support/row_data_children.rs",
        PRODUCTION_GUARD_SUPPORT_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "production_guard_runtime_row_data",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/production_guard_runtime_row_data.rs",
        PRODUCTION_GUARD_RUNTIME_ROW_DATA_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "row_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/row_ownership.rs",
        "runtime_15_m3_child_groups_representative_rows_are_child_owned",
    ),
    (
        "review_status_sync",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/review_status_sync.rs",
        REVIEW_STATUS_SYNC_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "root_inventory",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/root_inventory.rs",
        ROOT_INVENTORY_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/status_mirrors.rs",
        "runtime_15_m3_child_groups_row_data_guard_folder_backed_status_mirrors_are_current",
    ),
    (
        "ui_tests_first",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/ui_tests_first.rs",
        UI_TESTS_FIRST_ROW_DATA_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "ui_tests_second",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/ui_tests_second.rs",
        UI_TESTS_SECOND_ROW_DATA_CHILD_SPLIT_GUARD_NAME,
    ),
];

pub(super) const PRODUCTION_GUARD_SUPPORT_CHILD_ROWS: &[(&str, &str, &str)] = &[
    (
        "core_and_evidence",
        PRODUCTION_GUARD_SUPPORT_CORE_AND_EVIDENCE_ROWS_PATH,
        "pub(super) const CHILD_GROUP_INVENTORY_GUARD_INVENTORY_EXPECTED_STATUS_OUTPUT_SLICES",
    ),
    (
        "module_layout",
        PRODUCTION_GUARD_SUPPORT_MODULE_LAYOUT_ROWS_PATH,
        "pub(super) const CHILD_SUMMARY_STATUS_DOC_EXPECTED_STATUS_OUTPUT_SLICES",
    ),
    (
        "review_guard",
        PRODUCTION_GUARD_SUPPORT_REVIEW_GUARD_ROWS_PATH,
        "pub(super) const DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES",
    ),
    (
        "runtime_row_data",
        PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_ROWS_PATH,
        "pub(super) const FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
    ),
    (
        "status_docs",
        PRODUCTION_GUARD_SUPPORT_STATUS_DOCS_ROWS_PATH,
        "pub(super) const CHILD_GROUP_MOVED_ROW_EXPECTED_STATUS_OUTPUT_SLICES",
    ),
    (
        "expected_slice_guards",
        PRODUCTION_GUARD_SUPPORT_EXPECTED_SLICE_GUARDS_ROWS_PATH,
        "Runtime 15 M3 status output expected-slice guard child-owner split",
    ),
];
