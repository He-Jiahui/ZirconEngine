use super::*;

#[path = "module_layout_child_summary_status_docs/budgets.rs"]
mod budgets;
#[path = "module_layout_child_summary_status_docs/delegation.rs"]
mod delegation;
#[path = "module_layout_child_summary_status_docs/source_ownership.rs"]
mod source_ownership;
#[path = "module_layout_child_summary_status_docs/status_mirrors.rs"]
mod status_mirrors;

pub(super) const STATUS_OUTPUT_ROW_DATA_PARENT_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs";
pub(super) const CHILD_SUMMARY_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries.rs";
pub(super) const CHILD_SUMMARY_STATUS_DOCS_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summary_status_docs.rs";
pub(super) const PRODUCTION_GUARD_SUPPORT_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs";

pub(super) const ROW_DATA_GUARD_STATUS_NAME: &str =
    "Runtime 15 M3 status output row-data module-layout child-summary guard child-owner split";
pub(super) const ROW_DATA_GUARD_STATUS_ID: &str =
    "runtime_15_status_output_row_data_module_layout_child_summary_guard_child_owner_split_static_passed_cargo_deferred";
pub(super) const ROW_DATA_GUARD_NAME: &str =
    "runtime_15_status_output_row_data_module_layout_child_summaries_are_child_owner";
pub(super) const HISTORICAL_STATUS_NAME: &str =
    "Runtime 15 M3 module-layout child-summary status-doc guard child-owner split";
pub(super) const HISTORICAL_STATUS_ID: &str =
    "runtime_15_module_layout_child_summary_status_docs_child_owner_split_static_passed_cargo_deferred";
pub(super) const HISTORICAL_GUARD_NAME: &str =
    "runtime_15_module_layout_child_summary_status_docs_are_child_owner";
pub(super) const FOLDER_BACKED_STATUS_NAME: &str =
    "Runtime 15 M3 module-layout child-summary status-doc guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS_ID: &str =
    "runtime_15_module_layout_child_summary_status_docs_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_GUARD_NAME: &str =
    "runtime_15_module_layout_child_summary_status_docs_guard_is_folder_backed";

pub(super) const CHILD_SUMMARY_STATUS_DOC_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summary_status_docs/budgets.rs",
        "runtime_15_module_layout_child_summary_status_docs_guard_children_stay_focused",
    ),
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summary_status_docs/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "source_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summary_status_docs/source_ownership.rs",
        "runtime_15_module_layout_child_summary_status_doc_sources_are_child_owned",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summary_status_docs/status_mirrors.rs",
        HISTORICAL_GUARD_NAME,
    ),
];

pub(super) const CHILD_SUMMARY_STATUS_DOC_OWNER_BUDGETS: &[(&str, &str, usize)] = &[
    (
        "structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries.rs",
        CHILD_SUMMARY_GUARD_PATH,
        120,
    ),
    (
        "structure_convention/test_file_budget/status_output_row_data/module_layout_child_summary_status_docs.rs",
        CHILD_SUMMARY_STATUS_DOCS_GUARD_PATH,
        110,
    ),
];

pub(super) fn child_summary_status_doc_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in CHILD_SUMMARY_STATUS_DOC_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
