use super::*;

#[path = "module_layout_child_summaries/delegation.rs"]
mod delegation;
#[path = "module_layout_child_summaries/foundation_review.rs"]
mod foundation_review;
#[path = "module_layout_child_summaries/milestone_groups.rs"]
mod milestone_groups;
#[path = "module_layout_child_summaries/owner_budgets.rs"]
mod owner_budgets;

pub(super) const CHILD_SUMMARY_PARENT_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries.rs";

pub(super) const CHILD_SUMMARY_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries/delegation.rs",
        "runtime_15_status_output_row_data_module_layout_child_summaries_are_child_owner",
    ),
    (
        "foundation_review",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries/foundation_review.rs",
        "runtime_15_module_layout_child_summary_foundation_review_rows_are_child_owner",
    ),
    (
        "milestone_groups",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries/milestone_groups.rs",
        "runtime_15_module_layout_child_summary_milestone_groups_are_child_owner",
    ),
    (
        "owner_budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries/owner_budgets.rs",
        "runtime_15_module_layout_child_summary_guard_owner_budgets_are_child_owned",
    ),
];

pub(super) const CHILD_SUMMARY_STATUS_ANCHORS: &[&str] = &[
    "Runtime 15 M3 status output row-data module-layout child-summary guard child-owner split",
    "runtime_15_status_output_row_data_module_layout_child_summary_guard_child_owner_split_static_passed_cargo_deferred",
    "Runtime 15 M3 module-layout child-summary guard folder-backed split",
    "runtime_15_module_layout_child_summary_guard_folder_backed_static_passed_cargo_deferred",
];

pub(super) fn child_summary_child_sources() -> Vec<(&'static str, String)> {
    CHILD_SUMMARY_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn child_summary_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in child_summary_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
