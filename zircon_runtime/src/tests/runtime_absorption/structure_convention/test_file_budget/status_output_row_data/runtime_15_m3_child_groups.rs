use super::*;

#[path = "runtime_15_m3_child_groups/budgets.rs"]
mod budgets;
#[path = "runtime_15_m3_child_groups/delegation.rs"]
mod delegation;
#[path = "runtime_15_m3_child_groups/exports.rs"]
mod exports;
#[path = "runtime_15_m3_child_groups/row_ownership.rs"]
mod row_ownership;
#[path = "runtime_15_m3_child_groups/status_mirrors.rs"]
mod status_mirrors;

pub(super) const STATUS_OUTPUT_ROW_DATA_PARENT_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs";
pub(super) const M3_CHILD_GROUPS_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_groups.rs";
pub(super) const TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs";
pub(super) const RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs";
pub(super) const RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs";
pub(super) const FOUNDATION_GUARDS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs";
pub(super) const LOCK_POISON_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs";
pub(super) const MODULE_CONVENTION_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/module_convention_status.rs";
pub(super) const REVIEW_STATUS_SYNC_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_status_sync.rs";
pub(super) const STATUS_SUPPORT_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs";
pub(super) const UI_TESTS_SECOND_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/ui_tests_second.rs";
pub(super) const PRODUCTION_GUARD_SUPPORT_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs";
pub(super) const HISTORICAL_CHILD_OWNER_STATUS_NAME: &str =
    "Runtime 15 M3 status output M3 row data child-owner split";
pub(super) const HISTORICAL_CHILD_OWNER_STATUS_ID: &str =
    "runtime_15_status_output_m3_row_data_child_owner_split_static_passed_cargo_deferred";
pub(super) const HISTORICAL_CHILD_OWNER_GUARD_NAME: &str =
    "runtime_15_status_output_m3_row_data_child_owner_split";
pub(super) const FOLDER_BACKED_STATUS_NAME: &str =
    "Runtime 15 M3 child-groups row-data guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS_ID: &str =
    "runtime_15_m3_child_groups_row_data_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_GUARD_NAME: &str =
    "runtime_15_m3_child_groups_row_data_guard_is_folder_backed";

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
        "row_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_groups/row_ownership.rs",
        "runtime_15_m3_child_groups_representative_rows_are_child_owned",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_groups/status_mirrors.rs",
        "runtime_15_m3_child_groups_row_data_guard_folder_backed_status_mirrors_are_current",
    ),
];

pub(super) const M3_CHILD_GROUP_OWNER_PATHS: &[(&str, &str, usize)] = &[
    (
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_groups.rs",
        M3_CHILD_GROUPS_GUARD_PATH,
        140,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data.rs",
        TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH,
        240,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
        RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH,
        320,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
        RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH,
        220,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs",
        PRODUCTION_GUARD_SUPPORT_ROWS_PATH,
        800,
    ),
];

pub(super) fn m3_child_group_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in M3_CHILD_GROUP_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
