use super::*;

#[path = "runtime_15_review_guard_direct_assertion_rows/budgets.rs"]
mod budgets;
#[path = "runtime_15_review_guard_direct_assertion_rows/delegation.rs"]
mod delegation;
#[path = "runtime_15_review_guard_direct_assertion_rows/export_chain.rs"]
mod export_chain;
#[path = "runtime_15_review_guard_direct_assertion_rows/row_ownership.rs"]
mod row_ownership;
#[path = "runtime_15_review_guard_direct_assertion_rows/status_mirrors.rs"]
mod status_mirrors;

pub(super) const STATUS_OUTPUT_ROW_DATA_PARENT_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs";
pub(super) const DIRECT_ASSERTION_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_direct_assertion_rows.rs";
pub(super) const REVIEW_GUARD_ROW_DATA_AGGREGATION_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/aggregation.rs";
pub(super) const REVIEW_GUARD_SPLITS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits.rs";
pub(super) const CODE_REVIEW_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows.rs";
pub(super) const DIRECT_ASSERTION_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/direct_assertion_rows.rs";
pub(super) const PLUGIN_IMPORTER_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows.rs";
pub(super) const TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs";
pub(super) const RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs";
pub(super) const RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs";
pub(super) const STATUS_SUPPORT_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows.rs";
pub(super) const PRODUCTION_GUARD_SUPPORT_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs";
pub(super) const REVIEW_GUARD_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(super) const REVIEW_GUARD_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(super) const STATUS_SUPPORT_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs";
pub(super) const STATUS_SUPPORT_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs";

pub(super) const CHILD_OWNER_STATUS_NAME: &str =
    "Runtime 15 M3 review-guard direct-assertion row-data child-owner split";
pub(super) const CHILD_OWNER_STATUS_ID: &str =
    "runtime_15_review_guard_direct_assertion_row_data_child_owner_split_static_passed_cargo_deferred";
pub(super) const CHILD_OWNER_GUARD_NAME: &str =
    "runtime_15_status_output_review_guard_direct_assertion_rows_are_child_owner";
pub(super) const FOLDER_BACKED_STATUS_NAME: &str =
    "Runtime 15 M3 review-guard direct-assertion row-data guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS_ID: &str =
    "runtime_15_review_guard_direct_assertion_row_data_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_GUARD_NAME: &str =
    "runtime_15_review_guard_direct_assertion_rows_guard_is_folder_backed";

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

pub(super) fn direct_assertion_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in DIRECT_ASSERTION_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
