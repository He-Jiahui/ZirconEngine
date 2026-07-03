use super::*;

#[path = "runtime_15_foundation_row_data/budgets.rs"]
mod budgets;
#[path = "runtime_15_foundation_row_data/delegation.rs"]
mod delegation;
#[path = "runtime_15_foundation_row_data/exports.rs"]
mod exports;
#[path = "runtime_15_foundation_row_data/row_ownership.rs"]
mod row_ownership;
#[path = "runtime_15_foundation_row_data/status_mirrors.rs"]
mod status_mirrors;
#[path = "runtime_15_foundation_row_data/topic_rows.rs"]
mod topic_rows;

pub(super) const STATUS_OUTPUT_ROW_DATA_PARENT_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs";
pub(super) const RUNTIME_15_ROW_DATA_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data.rs";
pub(super) const FOUNDATION_ROW_DATA_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data.rs";
pub(super) const TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs";
pub(super) const RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs";
pub(super) const RUNTIME_15_FOUNDATION_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs";
pub(super) const FOUNDATION_CORE_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation/core_rows.rs";
pub(super) const FOUNDATION_TYPED_ERROR_RUNTIME_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation/typed_error_runtime_rows.rs";
pub(super) const FOUNDATION_TYPED_ERROR_PLUGIN_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation/typed_error_plugin_rows.rs";
pub(super) const FOUNDATION_TYPED_ERROR_SCENE_ASSET_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation/typed_error_scene_asset_rows.rs";

pub(super) const CHILD_OWNER_STATUS_NAME: &str =
    "Runtime 15 M3 foundation row-data guard child-owner split";
pub(super) const CHILD_OWNER_STATUS_ID: &str =
    "runtime_15_foundation_row_data_guard_child_owner_split_static_passed_cargo_deferred";
pub(super) const CHILD_OWNER_GUARD_NAME: &str =
    "runtime_15_status_output_runtime_15_foundation_row_data_is_child_owner";
pub(super) const FOLDER_BACKED_STATUS_NAME: &str =
    "Runtime 15 M3 foundation row-data guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS_ID: &str =
    "runtime_15_foundation_row_data_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_GUARD_NAME: &str =
    "runtime_15_foundation_row_data_guard_is_folder_backed";

pub(super) const FOUNDATION_ROW_DATA_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "row_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data/row_ownership.rs",
        CHILD_OWNER_GUARD_NAME,
    ),
    (
        "exports",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data/exports.rs",
        "runtime_15_foundation_row_data_exports_are_child_owned",
    ),
    (
        "topic_rows",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data/topic_rows.rs",
        "runtime_15_foundation_topic_rows_are_child_owned",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data/status_mirrors.rs",
        "runtime_15_foundation_row_data_folder_backed_status_mirrors_are_current",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data/budgets.rs",
        "runtime_15_foundation_row_data_child_budgets_stay_focused",
    ),
];

pub(super) fn runtime_15_row_count(source: &str) -> usize {
    source
        .lines()
        .filter(|line| line.starts_with("        \"Runtime 15 "))
        .count()
}

pub(super) fn foundation_row_data_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in FOUNDATION_ROW_DATA_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
