use super::*;

#[path = "runtime_15_m3_child_group_status_docs/budgets.rs"]
mod budgets;
#[path = "runtime_15_m3_child_group_status_docs/delegation.rs"]
mod delegation;
#[path = "runtime_15_m3_child_group_status_docs/source_ownership.rs"]
mod source_ownership;
#[path = "runtime_15_m3_child_group_status_docs/status_maps.rs"]
mod status_maps;
#[path = "runtime_15_m3_child_group_status_docs/status_mirrors.rs"]
mod status_mirrors;

pub(super) const STATUS_OUTPUT_ROW_DATA_PARENT_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs";
pub(super) const CHILD_GROUPS_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_groups.rs";
pub(super) const STATUS_DOCS_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_docs.rs";
pub(super) const STATUS_ROW_DOCS_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_row_docs.rs";
pub(super) const STATUS_ROW_DOCS_ROW_SOURCES_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_row_docs/row_sources.rs";
pub(super) const PRODUCTION_GUARD_SUPPORT_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs";
pub(super) const STATUS_SUPPORT_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs";
pub(super) const STATUS_SUPPORT_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs";

pub(super) const ROW_DATA_STATUS_NAME: &str =
    "Runtime 15 M3 status output M3 row data child-owner split";
pub(super) const ROW_DATA_STATUS_ID: &str =
    "runtime_15_status_output_m3_row_data_child_owner_split_static_passed_cargo_deferred";
pub(super) const HISTORICAL_STATUS_NAME: &str =
    "Runtime 15 M3 child-groups status-doc guard child-owner split";
pub(super) const HISTORICAL_STATUS_ID: &str =
    "runtime_15_m3_child_groups_status_docs_child_owner_split_static_passed_cargo_deferred";
pub(super) const HISTORICAL_GUARD_NAME: &str =
    "runtime_15_status_output_m3_child_group_status_docs_are_child_owner";
pub(super) const STATUS_ROW_DOC_STATUS_NAME: &str =
    "Runtime 15 M3 child-group status-row-doc guard child-owner split";
pub(super) const STATUS_ROW_DOC_STATUS_ID: &str =
    "runtime_15_m3_child_group_status_row_docs_child_owner_split_static_passed_cargo_deferred";
pub(super) const STATUS_ROW_DOC_GUARD_NAME: &str =
    "runtime_15_status_output_m3_child_group_status_row_docs_are_child_owner";
pub(super) const FOLDER_BACKED_STATUS_NAME: &str =
    "Runtime 15 M3 child-groups status-doc guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS_ID: &str =
    "runtime_15_m3_child_groups_status_docs_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_GUARD_NAME: &str =
    "runtime_15_m3_child_groups_status_docs_guard_is_folder_backed";

pub(super) const STATUS_DOCS_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_docs/budgets.rs",
        "runtime_15_m3_child_group_status_docs_child_budgets_stay_focused",
    ),
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_docs/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "source_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_docs/source_ownership.rs",
        "runtime_15_m3_child_group_status_doc_sources_are_child_owned",
    ),
    (
        "status_maps",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_docs/status_maps.rs",
        "runtime_15_m3_child_group_status_doc_maps_are_current",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_docs/status_mirrors.rs",
        HISTORICAL_GUARD_NAME,
    ),
];

pub(super) fn status_docs_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in STATUS_DOCS_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}

pub(super) fn status_row_docs_guard_source() -> String {
    format!(
        "{}\n{}",
        read_runtime_src(STATUS_ROW_DOCS_GUARD_PATH),
        read_runtime_src(STATUS_ROW_DOCS_ROW_SOURCES_PATH)
    )
}
