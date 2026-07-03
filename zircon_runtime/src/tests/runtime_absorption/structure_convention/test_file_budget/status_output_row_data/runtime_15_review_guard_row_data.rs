use super::*;

#[path = "runtime_15_review_guard_row_data/aggregation.rs"]
mod aggregation;
#[path = "runtime_15_review_guard_row_data/budgets.rs"]
mod budgets;
#[path = "runtime_15_review_guard_row_data/delegation.rs"]
mod delegation;
#[path = "runtime_15_review_guard_row_data/moved_rows.rs"]
mod moved_rows;
#[path = "runtime_15_review_guard_row_data/status_mirrors.rs"]
mod status_mirrors;
#[path = "runtime_15_review_guard_row_data/typed_error_rows.rs"]
mod typed_error_rows;

pub(super) const STATUS_OUTPUT_ROW_DATA_PARENT_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs";
pub(super) const RUNTIME_15_ROW_DATA_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data.rs";
pub(super) const REVIEW_GUARD_ROW_DATA_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data.rs";
pub(super) const MOVED_ROWS_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows.rs";
pub(super) const MOVED_ROWS_DELEGATION_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/delegation.rs";
pub(super) const MOVED_ROWS_CODE_REVIEW_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows.rs";
pub(super) const MOVED_ROWS_TYPED_ERROR_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/typed_error_rows.rs";
pub(super) const MOVED_ROWS_STATUS_MIRRORS_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/status_mirrors.rs";
pub(super) const TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs";
pub(super) const RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs";
pub(super) const RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs";
pub(super) const REVIEW_GUARD_SPLITS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits.rs";
pub(super) const REVIEW_GUARD_STATUS_SUPPORT_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows.rs";
pub(super) const TYPED_ERROR_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/typed_error_rows.rs";
pub(super) const TYPED_ERROR_NATIVE_PLUGIN_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/typed_error_rows/native_plugin_rows.rs";
pub(super) const TYPED_ERROR_RUNTIME_SURFACE_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/typed_error_rows/runtime_surface_rows.rs";
pub(super) const TYPED_ERROR_ASSET_SHADER_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/typed_error_rows/asset_shader_rows.rs";
pub(super) const REVIEW_GUARD_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(super) const REVIEW_GUARD_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs";

pub(super) const FOLDER_BACKED_STATUS_NAME: &str =
    "Runtime 15 M3 review-guard row-data guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS_ID: &str =
    "runtime_15_review_guard_row_data_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_GUARD_NAME: &str =
    "runtime_15_review_guard_row_data_guard_is_folder_backed";
pub(super) const CHILD_OWNER_STATUS_NAME: &str =
    "Runtime 15 M3 status output review-guard row-data guard child-owner split";
pub(super) const CHILD_OWNER_STATUS_ID: &str =
    "runtime_15_status_output_review_guard_row_data_guard_child_owner_split_static_passed_cargo_deferred";
pub(super) const CHILD_OWNER_GUARD_NAME: &str =
    "runtime_15_status_output_m3_review_guard_row_data_is_child_owner";
pub(super) const TYPED_ERROR_ROW_DATA_STATUS_NAME: &str =
    "Runtime 15 M3 review-guard typed-error row-data child split";
pub(super) const TYPED_ERROR_ROW_DATA_STATUS_ID: &str =
    "runtime_15_review_guard_typed_error_row_data_child_split_static_passed_cargo_deferred";
pub(super) const TYPED_ERROR_ROW_DATA_GUARD_NAME: &str =
    "runtime_15_review_guard_typed_error_rows_are_child_owned";

pub(super) const REVIEW_GUARD_ROW_DATA_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/delegation.rs",
        CHILD_OWNER_GUARD_NAME,
    ),
    (
        "moved_rows",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/moved_rows.rs",
        "runtime_15_review_guard_row_data_moved_rows_are_child_owned",
    ),
    (
        "aggregation",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/aggregation.rs",
        "runtime_15_review_guard_row_data_aggregation_exports_are_current",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/budgets.rs",
        "runtime_15_review_guard_row_data_child_budgets_stay_focused",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/status_mirrors.rs",
        "runtime_15_review_guard_row_data_folder_backed_status_mirrors_are_current",
    ),
    (
        "typed_error_rows",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/typed_error_rows.rs",
        TYPED_ERROR_ROW_DATA_GUARD_NAME,
    ),
];

pub(super) fn review_guard_row_data_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in REVIEW_GUARD_ROW_DATA_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}

pub(super) fn moved_rows_child_source_blob() -> String {
    [
        read_runtime_src(MOVED_ROWS_DELEGATION_PATH),
        read_runtime_src(MOVED_ROWS_CODE_REVIEW_PATH),
        read_runtime_src(MOVED_ROWS_TYPED_ERROR_PATH),
        read_runtime_src(MOVED_ROWS_STATUS_MIRRORS_PATH),
    ]
    .join("\n")
}
