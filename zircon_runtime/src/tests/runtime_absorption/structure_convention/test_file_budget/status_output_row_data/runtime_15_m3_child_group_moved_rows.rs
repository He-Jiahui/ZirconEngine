use super::*;

#[path = "runtime_15_m3_child_group_moved_rows/budgets.rs"]
mod budgets;
#[path = "runtime_15_m3_child_group_moved_rows/delegation.rs"]
mod delegation;
#[path = "runtime_15_m3_child_group_moved_rows/lock_poison_rows.rs"]
mod lock_poison_rows;
#[path = "runtime_15_m3_child_group_moved_rows/module_convention_rows.rs"]
mod module_convention_rows;
#[path = "runtime_15_m3_child_group_moved_rows/review_top_rows.rs"]
mod review_top_rows;
#[path = "runtime_15_m3_child_group_moved_rows/status_mirrors.rs"]
mod status_mirrors;

pub(super) const STATUS_OUTPUT_ROW_DATA_PARENT_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs";
pub(super) const CHILD_GROUPS_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_groups.rs";
pub(super) const MOVED_ROWS_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows.rs";
pub(super) const FOUNDATION_GUARDS_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs";
pub(super) const LOCK_POISON_STATUS_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs";
pub(super) const MODULE_CONVENTION_STATUS_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/module_convention_status.rs";
pub(super) const REVIEW_STATUS_SYNC_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_status_sync.rs";
pub(super) const PRODUCTION_GUARD_SUPPORT_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs";
pub(super) const STATUS_SUPPORT_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs";
pub(super) const STATUS_SUPPORT_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs";

pub(super) const CHILD_OWNER_STATUS_NAME: &str =
    "Runtime 15 M3 child-group moved-row guard child-owner split";
pub(super) const CHILD_OWNER_STATUS_ID: &str =
    "runtime_15_m3_child_group_moved_row_guard_child_owner_split_static_passed_cargo_deferred";
pub(super) const CHILD_OWNER_GUARD_NAME: &str =
    "runtime_15_status_output_m3_child_group_moved_rows_are_child_owner";
pub(super) const FOLDER_BACKED_STATUS_NAME: &str =
    "Runtime 15 M3 child-group moved-row guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS_ID: &str =
    "runtime_15_m3_child_group_moved_row_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_GUARD_NAME: &str =
    "runtime_15_m3_child_group_moved_rows_guard_is_folder_backed";

pub(super) const MOVED_ROW_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "lock_poison_rows",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows/lock_poison_rows.rs",
        "runtime_15_m3_child_group_moved_lock_poison_rows_are_child_owned",
    ),
    (
        "module_convention_rows",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows/module_convention_rows.rs",
        "runtime_15_m3_child_group_moved_module_convention_rows_are_child_owned",
    ),
    (
        "review_top_rows",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows/review_top_rows.rs",
        "runtime_15_m3_child_group_moved_review_top_rows_are_child_owned",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows/status_mirrors.rs",
        "runtime_15_m3_child_group_moved_row_folder_backed_status_mirrors_are_current",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows/budgets.rs",
        "runtime_15_m3_child_group_moved_row_child_budgets_stay_focused",
    ),
];

pub(super) fn moved_row_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in MOVED_ROW_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
