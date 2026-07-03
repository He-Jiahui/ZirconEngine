use super::*;

#[path = "runtime_15_m3_child_group_status_row_docs/budgets.rs"]
mod budgets;
#[path = "runtime_15_m3_child_group_status_row_docs/delegation.rs"]
mod delegation;
#[path = "runtime_15_m3_child_group_status_row_docs/row_sources.rs"]
mod row_sources;
#[path = "runtime_15_m3_child_group_status_row_docs/status_maps.rs"]
mod status_maps;
#[path = "runtime_15_m3_child_group_status_row_docs/status_mirrors.rs"]
mod status_mirrors;

pub(super) const STATUS_OUTPUT_ROW_DATA_PARENT_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs";
pub(super) const STATUS_DOCS_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_docs.rs";
pub(super) const STATUS_ROW_DOCS_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_row_docs.rs";
pub(super) const LOCK_POISON_STATUS_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs";
pub(super) const MODULE_CONVENTION_STATUS_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/module_convention_status.rs";
pub(super) const REVIEW_STATUS_SYNC_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_status_sync.rs";
pub(super) const PRODUCTION_GUARD_SUPPORT_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs";
pub(super) const M3_STRUCTURE_SUPPORT_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs";
pub(super) const M3_STRUCTURE_SUPPORT_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs";
pub(super) const REVIEW_GUARD_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(super) const REVIEW_GUARD_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(super) const STATUS_SUPPORT_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs";
pub(super) const STATUS_SUPPORT_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs";

pub(super) const CHILD_OWNER_STATUS_NAME: &str =
    "Runtime 15 M3 child-group status-row-doc guard child-owner split";
pub(super) const CHILD_OWNER_STATUS_ID: &str =
    "runtime_15_m3_child_group_status_row_docs_child_owner_split_static_passed_cargo_deferred";
pub(super) const CHILD_OWNER_GUARD_NAME: &str =
    "runtime_15_status_output_m3_child_group_status_row_docs_are_child_owner";
pub(super) const FOLDER_BACKED_STATUS_NAME: &str =
    "Runtime 15 M3 child-group status-row-doc guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS_ID: &str =
    "runtime_15_m3_child_group_status_row_docs_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_GUARD_NAME: &str =
    "runtime_15_m3_child_group_status_row_docs_guard_is_folder_backed";

pub(super) const STATUS_ROW_DOCS_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_row_docs/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "row_sources",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_row_docs/row_sources.rs",
        CHILD_OWNER_GUARD_NAME,
    ),
    (
        "status_maps",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_row_docs/status_maps.rs",
        "runtime_15_m3_child_group_status_row_doc_maps_are_current",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_row_docs/status_mirrors.rs",
        "runtime_15_m3_child_group_status_row_doc_folder_backed_status_mirrors_are_current",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_row_docs/budgets.rs",
        "runtime_15_m3_child_group_status_row_doc_child_budgets_stay_focused",
    ),
];

pub(super) fn status_row_doc_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in STATUS_ROW_DOCS_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
