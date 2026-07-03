use super::*;

#[path = "runtime_15_m2_row_data/budgets.rs"]
mod budgets;
#[path = "runtime_15_m2_row_data/delegation.rs"]
mod delegation;
#[path = "runtime_15_m2_row_data/row_ownership.rs"]
mod row_ownership;
#[path = "runtime_15_m2_row_data/status_mirrors.rs"]
mod status_mirrors;

pub(super) const STATUS_OUTPUT_ROW_DATA_PARENT_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs";
pub(super) const RUNTIME_15_ROW_DATA_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data.rs";
pub(super) const M2_ROW_DATA_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data.rs";
pub(super) const TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs";
pub(super) const RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs";
pub(super) const RUNTIME_15_FOUNDATION_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs";
pub(super) const RUNTIME_15_M2_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m2.rs";
pub(super) const STATUS_SUPPORT_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs";
pub(super) const STATUS_SUPPORT_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs";
pub(super) const PRODUCTION_GUARD_SUPPORT_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs";

pub(super) const ROW_DATA_SPLIT_STATUS_NAME: &str =
    "Runtime 15 M3 status output Runtime 15 M2 row data split";
pub(super) const ROW_DATA_SPLIT_STATUS_ID: &str =
    "runtime_15_status_output_runtime_15_m2_row_data_split_static_passed_cargo_deferred";
pub(super) const CHILD_OWNER_STATUS_NAME: &str =
    "Runtime 15 M3 M2 row-data guard child-owner split";
pub(super) const CHILD_OWNER_STATUS_ID: &str =
    "runtime_15_m2_row_data_guard_child_owner_split_static_passed_cargo_deferred";
pub(super) const CHILD_OWNER_GUARD_NAME: &str =
    "runtime_15_status_output_runtime_15_m2_row_data_is_child_owner";
pub(super) const FOLDER_BACKED_STATUS_NAME: &str =
    "Runtime 15 M3 M2 row-data guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS_ID: &str =
    "runtime_15_m2_row_data_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_GUARD_NAME: &str = "runtime_15_m2_row_data_guard_is_folder_backed";

pub(super) const M2_ROW_DATA_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "row_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data/row_ownership.rs",
        CHILD_OWNER_GUARD_NAME,
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data/status_mirrors.rs",
        "runtime_15_m2_row_data_guard_folder_backed_status_mirrors_are_current",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data/budgets.rs",
        "runtime_15_m2_row_data_guard_children_stay_focused",
    ),
];

pub(super) const M2_ROW_OWNER_PATHS: &[(&str, &str, usize)] = &[
    (
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data.rs",
        RUNTIME_15_ROW_DATA_GUARD_PATH,
        220,
    ),
    (
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data.rs",
        M2_ROW_DATA_GUARD_PATH,
        120,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        RUNTIME_15_FOUNDATION_EXPECTED_STATUS_ROW_DATA_PATH,
        120,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m2.rs",
        RUNTIME_15_M2_EXPECTED_STATUS_ROW_DATA_PATH,
        120,
    ),
];

pub(super) fn m2_row_data_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in M2_ROW_DATA_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
