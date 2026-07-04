use super::*;

pub(super) const M3_CHILD_GROUP_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_groups/budgets.rs",
        "runtime_15_m3_child_groups_row_data_guard_children_stay_focused",
    ),
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_groups/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "exports",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_groups/exports.rs",
        HISTORICAL_CHILD_OWNER_GUARD_NAME,
    ),
    (
        "production_guard_support",
        PRODUCTION_GUARD_SUPPORT_GUARD_PATH,
        PRODUCTION_GUARD_SUPPORT_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "production_guard_runtime_row_data",
        PRODUCTION_GUARD_RUNTIME_ROW_DATA_GUARD_PATH,
        PRODUCTION_GUARD_RUNTIME_ROW_DATA_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "row_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_groups/row_ownership.rs",
        "runtime_15_m3_child_groups_representative_rows_are_child_owned",
    ),
    (
        "root_inventory",
        ROOT_INVENTORY_GUARD_PATH,
        ROOT_INVENTORY_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_groups/status_mirrors.rs",
        "runtime_15_m3_child_groups_row_data_guard_folder_backed_status_mirrors_are_current",
    ),
];

pub(super) const PRODUCTION_GUARD_SUPPORT_CHILD_ROWS: &[(&str, &str, &str)] = &[
    (
        "core_and_evidence",
        PRODUCTION_GUARD_SUPPORT_CORE_AND_EVIDENCE_ROWS_PATH,
        "Runtime 15 M3 status output evidence anchors guard folder-backed split",
    ),
    (
        "module_layout",
        PRODUCTION_GUARD_SUPPORT_MODULE_LAYOUT_ROWS_PATH,
        "Runtime 15 M3 module-layout child-summary status-doc status-mirror child split",
    ),
    (
        "review_guard",
        PRODUCTION_GUARD_SUPPORT_REVIEW_GUARD_ROWS_PATH,
        "Runtime 15 M3 review-guard direct-assertion status-mirror child split",
    ),
    (
        "runtime_row_data",
        PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_ROWS_PATH,
        "pub(super) const FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
    ),
    (
        "status_docs",
        PRODUCTION_GUARD_SUPPORT_STATUS_DOCS_ROWS_PATH,
        "Runtime 15 M3 child-group moved-row status-mirror child split",
    ),
    (
        "expected_slice_guards",
        PRODUCTION_GUARD_SUPPORT_EXPECTED_SLICE_GUARDS_ROWS_PATH,
        "Runtime 15 M3 status output expected-slice guard child-owner split",
    ),
];
