use super::*;

#[path = "runtime_15_review_guard_row_data_moved_rows/code_review_rows.rs"]
mod code_review_rows;
#[path = "runtime_15_review_guard_row_data_moved_rows/delegation.rs"]
mod delegation;
#[path = "runtime_15_review_guard_row_data_moved_rows/status_mirrors.rs"]
mod status_mirrors;
#[path = "runtime_15_review_guard_row_data_moved_rows/typed_error_rows.rs"]
mod typed_error_rows;

pub(super) const MOVED_ROWS_PARENT_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows.rs";
pub(super) const REVIEW_GUARD_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data.rs";
pub(super) const FOUNDATION_GUARDS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs";
pub(super) const REVIEW_GUARD_SPLITS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits.rs";
pub(super) const CODE_REVIEW_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows.rs";
pub(super) const REVIEW_GUARD_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/review_guard_rows.rs";
pub(super) const PLUGIN_IMPORTER_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows.rs";
pub(super) const STRUCTURE_GUARD_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows.rs";
pub(super) const STRUCTURE_GUARD_ROOT_AND_CHILDREN_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children.rs";
pub(super) const STRUCTURE_GUARD_STATUS_DOCS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/status_docs.rs";
pub(super) const STRUCTURE_GUARD_FOLDER_BACKED_SUMMARY_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/folder_backed_summary.rs";
pub(super) const STRUCTURE_GUARD_TYPED_ERROR_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/typed_error.rs";
pub(super) const STRUCTURE_GUARD_ROW_DATA_OWNER_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/row_data_owner.rs";
pub(super) const TYPED_ERROR_STRUCTURE_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows.rs";
pub(super) const TYPED_ERROR_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/typed_error_rows.rs";

pub(super) const CHILD_OWNER_STATUS_NAME: &str =
    "Runtime 15 M3 review-guard row-data moved-row guard child-owner split";
pub(super) const CHILD_OWNER_STATUS_ID: &str =
    "runtime_15_review_guard_row_data_moved_rows_child_owner_split_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_STATUS_NAME: &str =
    "Runtime 15 M3 review-guard moved-row guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS_ID: &str =
    "runtime_15_review_guard_moved_row_guard_folder_backed_static_passed_cargo_deferred";

pub(super) const MOVED_ROWS_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/delegation.rs",
        "runtime_15_status_output_m3_review_guard_row_data_moved_rows_are_child_owner",
    ),
    (
        "code_review_rows",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows.rs",
        "runtime_15_review_guard_moved_row_code_review_rows_are_child_owned",
    ),
    (
        "typed_error_rows",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/typed_error_rows.rs",
        "runtime_15_review_guard_moved_row_typed_error_rows_are_child_owned",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/status_mirrors.rs",
        "runtime_15_review_guard_moved_row_folder_backed_status_mirrors_are_current",
    ),
];

pub(super) const MOVED_ROWS_STATUS_ANCHORS: &[&str] = &[
    CHILD_OWNER_STATUS_NAME,
    CHILD_OWNER_STATUS_ID,
    FOLDER_BACKED_STATUS_NAME,
    FOLDER_BACKED_STATUS_ID,
];

pub(super) fn moved_row_child_sources() -> Vec<(&'static str, String)> {
    MOVED_ROWS_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn moved_row_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in moved_row_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
