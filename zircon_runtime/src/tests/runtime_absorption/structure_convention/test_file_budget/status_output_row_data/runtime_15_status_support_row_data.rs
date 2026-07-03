use super::*;

#[path = "runtime_15_status_support_row_data/budgets.rs"]
mod budgets;
#[path = "runtime_15_status_support_row_data/delegation.rs"]
mod delegation;
#[path = "runtime_15_status_support_row_data/export_chain.rs"]
mod export_chain;
#[path = "runtime_15_status_support_row_data/row_ownership.rs"]
mod row_ownership;
#[path = "runtime_15_status_support_row_data/status_mirrors.rs"]
mod status_mirrors;

pub(super) const STATUS_OUTPUT_ROW_DATA_PARENT_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs";
pub(super) const TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs";
pub(super) const RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs";
pub(super) const RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs";
pub(super) const STATUS_SUPPORT_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs";
pub(super) const STATUS_SUPPORT_ROW_DATA_AND_BUDGET_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget.rs";
pub(super) const STATUS_SUPPORT_EXPECTED_SLICE_MAPS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps.rs";
pub(super) const STATUS_SUPPORT_RUNTIME_INDEX_ANCHORS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors.rs";
pub(super) const STATUS_SUPPORT_PRIORITY_PLAN_DOCS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs.rs";
pub(super) const PRIORITY_PLAN_DOCS_INTEGRITY_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/integrity_guards.rs";
pub(super) const PRIORITY_PLAN_DOCS_OWNER_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards.rs";
pub(super) const PRIORITY_PLAN_DOCS_FOLLOWUPS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/status_followups.rs";
pub(super) const PRIORITY_PLAN_DOCS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/row_data_owner.rs";
pub(super) const STATUS_SUPPORT_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs";
pub(super) const STATUS_SUPPORT_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs";
pub(super) const PRODUCTION_GUARD_SUPPORT_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs";

pub(super) const CHILD_OWNER_STATUS_NAME: &str =
    "Runtime 15 M3 status-support row-data owner child split";
pub(super) const CHILD_OWNER_STATUS_ID: &str =
    "runtime_15_status_support_row_data_owner_child_split_static_passed_cargo_deferred";
pub(super) const CHILD_OWNER_GUARD_NAME: &str =
    "runtime_15_status_support_row_data_owner_is_child_backed";
pub(super) const FOLDER_BACKED_STATUS_NAME: &str =
    "Runtime 15 M3 status-support row-data guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS_ID: &str =
    "runtime_15_status_support_row_data_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_GUARD_NAME: &str =
    "runtime_15_status_support_row_data_guard_is_folder_backed";

pub(super) const STATUS_SUPPORT_ROW_DATA_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_status_support_row_data/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "row_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_status_support_row_data/row_ownership.rs",
        CHILD_OWNER_GUARD_NAME,
    ),
    (
        "export_chain",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_status_support_row_data/export_chain.rs",
        "runtime_15_status_support_row_data_export_chain_is_current",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_status_support_row_data/status_mirrors.rs",
        "runtime_15_status_support_row_data_folder_backed_status_mirrors_are_current",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_status_support_row_data/budgets.rs",
        "runtime_15_status_support_row_data_child_budgets_stay_focused",
    ),
];

pub(super) const STATUS_SUPPORT_ROW_OWNER_PATHS: &[(&str, &str, usize)] = &[
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs",
        STATUS_SUPPORT_ROWS_PATH,
        120,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget.rs",
        STATUS_SUPPORT_ROW_DATA_AND_BUDGET_PATH,
        280,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps.rs",
        STATUS_SUPPORT_EXPECTED_SLICE_MAPS_PATH,
        160,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors.rs",
        STATUS_SUPPORT_RUNTIME_INDEX_ANCHORS_PATH,
        220,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs.rs",
        STATUS_SUPPORT_PRIORITY_PLAN_DOCS_PATH,
        80,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/integrity_guards.rs",
        PRIORITY_PLAN_DOCS_INTEGRITY_PATH,
        120,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards.rs",
        PRIORITY_PLAN_DOCS_OWNER_PATH,
        120,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/status_followups.rs",
        PRIORITY_PLAN_DOCS_FOLLOWUPS_PATH,
        80,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/row_data_owner.rs",
        PRIORITY_PLAN_DOCS_ROW_DATA_PATH,
        80,
    ),
];

pub(super) fn status_support_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in STATUS_SUPPORT_ROW_DATA_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
