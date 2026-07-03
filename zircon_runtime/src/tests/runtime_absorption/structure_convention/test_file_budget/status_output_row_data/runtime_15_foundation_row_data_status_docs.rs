use super::*;

#[path = "runtime_15_foundation_row_data_status_docs/delegation.rs"]
mod delegation;
#[path = "runtime_15_foundation_row_data_status_docs/doc_mirrors.rs"]
mod doc_mirrors;
#[path = "runtime_15_foundation_row_data_status_docs/row_count.rs"]
mod row_count;
#[path = "runtime_15_foundation_row_data_status_docs/status_maps.rs"]
mod status_maps;

pub(super) const STATUS_DOCS_PARENT_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data_status_docs.rs";
pub(super) const FOUNDATION_ROW_DATA_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data.rs";
pub(super) const STATUS_SUPPORT_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs";
pub(super) const PRODUCTION_GUARD_SUPPORT_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs";
pub(super) const STATUS_SUPPORT_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs";
pub(super) const STATUS_SUPPORT_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs";

pub(super) const FOUNDATION_ROW_DATA_SPLIT_NAME: &str =
    "Runtime 15 M3 status output Runtime 15 foundation row data split";
pub(super) const FOUNDATION_ROW_DATA_SPLIT_ID: &str =
    "runtime_15_status_output_runtime_15_foundation_row_data_split_static_passed_cargo_deferred";
pub(super) const FOUNDATION_TOPIC_SPLIT_NAME: &str =
    "Runtime 15 M3 foundation row-data topic child-owner split";
pub(super) const FOUNDATION_TOPIC_SPLIT_ID: &str =
    "runtime_15_foundation_row_data_topic_child_owner_split_static_passed_cargo_deferred";
pub(super) const FOUNDATION_GUARD_SPLIT_NAME: &str =
    "Runtime 15 M3 foundation row-data guard child-owner split";
pub(super) const FOUNDATION_GUARD_SPLIT_ID: &str =
    "runtime_15_foundation_row_data_guard_child_owner_split_static_passed_cargo_deferred";
pub(super) const STATUS_DOC_SPLIT_NAME: &str =
    "Runtime 15 M3 foundation row-data status-doc guard child-owner split";
pub(super) const STATUS_DOC_SPLIT_ID: &str =
    "runtime_15_foundation_row_data_status_docs_child_owner_split_static_passed_cargo_deferred";
pub(super) const ROW_COUNT_SYNC_NAME: &str = "Runtime 15 M3 foundation row-data 73-row docs sync";
pub(super) const ROW_COUNT_SYNC_ID: &str =
    "runtime_15_foundation_row_data_71_row_docs_sync_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_STATUS_NAME: &str =
    "Runtime 15 M3 foundation row-data status-doc guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS_ID: &str =
    "runtime_15_foundation_row_data_status_docs_folder_backed_static_passed_cargo_deferred";

pub(super) const STATUS_DOC_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data_status_docs/delegation.rs",
        "runtime_15_status_output_foundation_row_data_status_docs_are_child_owner",
    ),
    (
        "status_maps",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data_status_docs/status_maps.rs",
        "runtime_15_foundation_row_data_status_doc_maps_are_child_owned",
    ),
    (
        "doc_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data_status_docs/doc_mirrors.rs",
        "runtime_15_foundation_row_data_status_doc_mirrors_are_current",
    ),
    (
        "row_count",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data_status_docs/row_count.rs",
        "runtime_15_foundation_row_data_docs_record_current_row_count",
    ),
];

pub(super) const STATUS_DOC_STATUS_ANCHORS: &[&str] = &[
    STATUS_DOC_SPLIT_NAME,
    STATUS_DOC_SPLIT_ID,
    ROW_COUNT_SYNC_NAME,
    ROW_COUNT_SYNC_ID,
    FOLDER_BACKED_STATUS_NAME,
    FOLDER_BACKED_STATUS_ID,
];

pub(super) fn runtime_15_row_count(source: &str) -> usize {
    source
        .lines()
        .filter(|line| line.starts_with("        \"Runtime 15 "))
        .count()
}

pub(super) fn status_doc_child_sources() -> Vec<(&'static str, String)> {
    STATUS_DOC_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn status_doc_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in status_doc_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
