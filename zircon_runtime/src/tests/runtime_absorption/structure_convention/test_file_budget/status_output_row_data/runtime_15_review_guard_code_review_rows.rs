use super::*;

#[path = "runtime_15_review_guard_code_review_rows/budgets.rs"]
mod budgets;
#[path = "runtime_15_review_guard_code_review_rows/delegation.rs"]
mod delegation;
#[path = "runtime_15_review_guard_code_review_rows/export_chain.rs"]
mod export_chain;
#[path = "runtime_15_review_guard_code_review_rows/root_and_children.rs"]
mod root_and_children;
#[path = "runtime_15_review_guard_code_review_rows/row_ownership.rs"]
mod row_ownership;
#[path = "runtime_15_review_guard_code_review_rows/status_mirrors.rs"]
mod status_mirrors;

pub(super) const STATUS_OUTPUT_ROW_DATA_PARENT_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs";
pub(super) const CODE_REVIEW_ROWS_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows.rs";
pub(super) const TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs";
pub(super) const RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs";
pub(super) const RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs";
pub(super) const REVIEW_GUARD_SPLITS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits.rs";
pub(super) const CODE_REVIEW_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows.rs";
pub(super) const REVIEW_GUARD_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/review_guard_rows.rs";
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
pub(super) const ROW_DATA_OWNER_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/row_data_owner.rs";
pub(super) const STATUS_SUPPORT_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs";
pub(super) const STATUS_SUPPORT_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs";
pub(super) const STATUS_SUPPORT_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs";
pub(super) const REVIEW_GUARD_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(super) const REVIEW_GUARD_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs";

pub(super) const FOLDER_BACKED_STATUS_NAME: &str =
    "Runtime 15 M3 review-guard code-review row-data guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS_ID: &str =
    "runtime_15_review_guard_code_review_rows_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_GUARD_NAME: &str =
    "runtime_15_review_guard_code_review_rows_guard_is_folder_backed";

pub(super) const CODE_REVIEW_ROWS_ROW_DATA_STATUS_NAME: &str =
    "Runtime 15 M3 code-review row-data owner child split";
pub(super) const CODE_REVIEW_ROWS_ROW_DATA_STATUS_ID: &str =
    "runtime_15_code_review_rows_row_data_owner_child_split_static_passed_cargo_deferred";
pub(super) const CODE_REVIEW_ROWS_ROW_DATA_GUARD_NAME: &str =
    "runtime_15_code_review_rows_row_data_owner_is_child_backed";
pub(super) const STRUCTURE_GUARD_ROW_DATA_STATUS_NAME: &str =
    "Runtime 15 M3 code-review structure-guard row-data folder-backed split";
pub(super) const STRUCTURE_GUARD_ROW_DATA_STATUS_ID: &str =
    "runtime_15_code_review_structure_guard_row_data_folder_backed_static_passed_cargo_deferred";
pub(super) const STRUCTURE_GUARD_ROW_DATA_GUARD_NAME: &str =
    "runtime_15_code_review_structure_guard_row_data_is_folder_backed";
pub(super) const ROOT_AND_CHILDREN_ROW_DATA_STATUS_NAME: &str =
    "Runtime 15 M3 code-review structure-guard root-and-children row-data child split";
pub(super) const ROOT_AND_CHILDREN_ROW_DATA_STATUS_ID: &str =
    "runtime_15_code_review_structure_guard_root_and_children_row_data_child_split_static_passed_cargo_deferred";
pub(super) const ROOT_AND_CHILDREN_ROW_DATA_GUARD_NAME: &str =
    "runtime_15_code_review_structure_guard_root_and_children_row_data_is_child_backed";

pub(super) const CODE_REVIEW_ROWS_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "row_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/row_ownership.rs",
        CODE_REVIEW_ROWS_ROW_DATA_GUARD_NAME,
    ),
    (
        "root_and_children",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/root_and_children.rs",
        ROOT_AND_CHILDREN_ROW_DATA_GUARD_NAME,
    ),
    (
        "export_chain",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/export_chain.rs",
        "runtime_15_review_guard_code_review_row_exports_are_current",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/budgets.rs",
        "runtime_15_review_guard_code_review_rows_child_budgets_stay_focused",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/status_mirrors.rs",
        "runtime_15_review_guard_code_review_rows_status_mirrors_are_current",
    ),
];

pub(super) fn code_review_rows_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in CODE_REVIEW_ROWS_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}

pub(super) fn structure_guard_rows_source_blob() -> String {
    [
        read_runtime_src(STRUCTURE_GUARD_ROWS_PATH),
        structure_guard_root_and_children_source_blob(),
        read_runtime_src(STRUCTURE_GUARD_STATUS_DOCS_PATH),
        read_runtime_src(STRUCTURE_GUARD_FOLDER_BACKED_SUMMARY_PATH),
        read_runtime_src(STRUCTURE_GUARD_TYPED_ERROR_PATH),
        read_runtime_src(STRUCTURE_GUARD_ROW_DATA_OWNER_PATH),
    ]
    .concat()
}

pub(super) fn structure_guard_root_and_children_source_blob() -> String {
    [
        read_runtime_src(STRUCTURE_GUARD_ROOT_AND_CHILDREN_PATH),
        read_runtime_src("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/code_review_findings.rs"),
        read_runtime_src("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/p0_robustness.rs"),
        read_runtime_src("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/plugin_importer_dx.rs"),
        read_runtime_src("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/p0_native_fixture.rs"),
        read_runtime_src("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/f8_child_owner.rs"),
        read_runtime_src("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/late_api_cleanup.rs"),
    ]
    .concat()
}

pub(super) const CODE_REVIEW_CHILD_EXPORTS: &[&str] = &[
    "CODE_REVIEW_EXPECTED_STATUS_OUTPUT_SLICES",
    "CODE_REVIEW_DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES",
    "CODE_REVIEW_PLUGIN_IMPORTER_EXPECTED_STATUS_OUTPUT_SLICES",
    "CODE_REVIEW_STRUCTURE_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
    "CODE_REVIEW_STRUCTURE_GUARD_STATUS_DOCS_EXPECTED_STATUS_OUTPUT_SLICES",
    "CODE_REVIEW_STRUCTURE_GUARD_FOLDER_BACKED_SUMMARY_EXPECTED_STATUS_OUTPUT_SLICES",
    "CODE_REVIEW_STRUCTURE_GUARD_TYPED_ERROR_EXPECTED_STATUS_OUTPUT_SLICES",
    "CODE_REVIEW_STRUCTURE_GUARD_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
    "CODE_REVIEW_TYPED_ERROR_STRUCTURE_EXPECTED_STATUS_OUTPUT_SLICES",
    "CODE_REVIEW_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
];
