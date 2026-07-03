use super::*;

#[path = "module_layout/budgets.rs"]
mod budgets;
#[path = "module_layout/child_summaries.rs"]
mod child_summaries;
#[path = "module_layout/delegation.rs"]
mod delegation;
#[path = "module_layout/status_mirrors.rs"]
mod status_mirrors;

pub(super) const STATUS_OUTPUT_ROW_DATA_PARENT_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs";
pub(super) const MODULE_LAYOUT_PARENT_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout.rs";
pub(super) const MODULE_LAYOUT_CHILD_SUMMARIES_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries.rs";
pub(super) const MODULE_LAYOUT_CHILD_SUMMARY_DELEGATION_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries/delegation.rs";
pub(super) const MODULE_LAYOUT_CHILD_SUMMARY_FOUNDATION_REVIEW_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries/foundation_review.rs";
pub(super) const MODULE_LAYOUT_CHILD_SUMMARY_MILESTONE_GROUPS_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries/milestone_groups.rs";
pub(super) const MODULE_LAYOUT_CHILD_SUMMARY_OWNER_BUDGETS_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries/owner_budgets.rs";
pub(super) const MODULE_LAYOUT_CHILD_SUMMARY_STATUS_DOCS_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summary_status_docs.rs";
pub(super) const MODULE_LAYOUT_STATUS_DOCS_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_status_docs.rs";

pub(super) const HISTORICAL_STATUS_NAME: &str =
    "Runtime 15 M3 status output row-data guard child-owner split";
pub(super) const HISTORICAL_STATUS_ID: &str =
    "runtime_15_status_output_row_data_guard_child_owner_split_static_passed_cargo_deferred";
pub(super) const HISTORICAL_GUARD_NAME: &str =
    "runtime_15_status_output_row_data_guard_child_owner_split";
pub(super) const FOLDER_BACKED_STATUS_NAME: &str =
    "Runtime 15 M3 status-output row-data module-layout guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS_ID: &str =
    "runtime_15_status_output_row_data_module_layout_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_GUARD_NAME: &str =
    "runtime_15_status_output_row_data_module_layout_guard_is_folder_backed";

pub(super) const MODULE_LAYOUT_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout/delegation.rs",
        HISTORICAL_GUARD_NAME,
    ),
    (
        "child_summaries",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout/child_summaries.rs",
        "runtime_15_status_output_row_data_module_layout_child_summaries_stay_delegated",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout/status_mirrors.rs",
        "runtime_15_status_output_row_data_module_layout_status_mirrors_are_current",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout/budgets.rs",
        "runtime_15_status_output_row_data_module_layout_children_stay_focused",
    ),
];

pub(super) const MODULE_LAYOUT_GUARD_OWNER_PATHS: &[(&str, &str, usize)] = &[
    (
        "status-output row-data parent",
        STATUS_OUTPUT_ROW_DATA_PARENT_PATH,
        400,
    ),
    ("module-layout parent", MODULE_LAYOUT_PARENT_PATH, 160),
    (
        "module-layout delegation child",
        MODULE_LAYOUT_CHILDREN[0].1,
        180,
    ),
    (
        "module-layout child-summary child",
        MODULE_LAYOUT_CHILDREN[1].1,
        160,
    ),
    (
        "module-layout status-mirror child",
        MODULE_LAYOUT_CHILDREN[2].1,
        180,
    ),
    (
        "module-layout budget child",
        MODULE_LAYOUT_CHILDREN[3].1,
        80,
    ),
    (
        "module-layout child-summary parent",
        MODULE_LAYOUT_CHILD_SUMMARIES_PATH,
        400,
    ),
    (
        "module-layout child-summary status-doc child",
        MODULE_LAYOUT_CHILD_SUMMARY_STATUS_DOCS_PATH,
        400,
    ),
    (
        "module-layout status-doc child",
        MODULE_LAYOUT_STATUS_DOCS_PATH,
        400,
    ),
];

pub(super) fn module_layout_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in MODULE_LAYOUT_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
