use super::super::super::super::*;
use super::super::source_inventory::CodeReviewFindingsSources;

#[path = "root_parent/backflow.rs"]
mod backflow;
#[path = "root_parent/budgets.rs"]
mod budgets;
#[path = "root_parent/delegation.rs"]
mod delegation;
#[path = "root_parent/parent_mounts.rs"]
mod parent_mounts;
#[path = "root_parent/status_mirrors.rs"]
mod status_mirrors;

pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions.rs";
pub(super) const ROOT_PARENT_DIRECT_ASSERTIONS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/root_parent.rs";
pub(super) const ROOT_PARENT_DIRECT_ASSERTIONS_DELEGATION_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/root_parent/delegation.rs";
pub(super) const ROOT_PARENT_DIRECT_ASSERTIONS_PARENT_MOUNTS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/root_parent/parent_mounts.rs";
pub(super) const ROOT_PARENT_DIRECT_ASSERTIONS_BACKFLOW_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/root_parent/backflow.rs";
pub(super) const ROOT_PARENT_DIRECT_ASSERTIONS_BUDGETS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/root_parent/budgets.rs";
pub(super) const ROOT_PARENT_DIRECT_ASSERTIONS_STATUS_MIRRORS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/root_parent/status_mirrors.rs";
pub(super) const ROOT_PARENT_DIRECT_ASSERTIONS_FOLDER_BACKED_SLICE: &str =
    "Runtime 15 M3 code review findings root-parent direct assertions guard folder-backed split";
pub(super) const ROOT_PARENT_DIRECT_ASSERTIONS_FOLDER_BACKED_STATUS: &str =
    "runtime_15_code_review_findings_root_parent_direct_assertions_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const ROOT_PARENT_DIRECT_ASSERTIONS_FOLDER_BACKED_DATE: &str = "2026-07-04";
pub(super) const ROOT_PARENT_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD: &str =
    "runtime_15_code_review_findings_root_parent_direct_assertions_guard_is_folder_backed";
pub(super) const ROOT_PARENT_DIRECT_ASSERTIONS_STATUS_GUARD: &str =
    "runtime_15_code_review_findings_root_parent_direct_assertions_guard_folder_backed_status_is_current";
pub(super) const ROOT_PARENT_DIRECT_ASSERTIONS_BUDGET_GUARD: &str =
    "runtime_15_code_review_findings_root_parent_direct_assertions_children_line_budgets_are_current";
pub(super) const CODE_REVIEW_FINDINGS_LINE_BUDGET: usize = 800;

const REVIEW_GUARD_STATUS_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/direct_assertion_rows.rs";
const REVIEW_GUARD_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs";
const REVIEW_GUARD_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs";

pub(super) const ROOT_PARENT_DIRECT_ASSERTIONS_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        ROOT_PARENT_DIRECT_ASSERTIONS_DELEGATION_CHILD,
        "runtime_15_code_review_findings_root_parent_direct_assertions_are_child_owner",
    ),
    (
        "parent_mounts",
        ROOT_PARENT_DIRECT_ASSERTIONS_PARENT_MOUNTS_CHILD,
        "assert_code_review_root_parent_mounts_are_folder_backed",
    ),
    (
        "backflow",
        ROOT_PARENT_DIRECT_ASSERTIONS_BACKFLOW_CHILD,
        ROOT_PARENT_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD,
    ),
    (
        "budgets",
        ROOT_PARENT_DIRECT_ASSERTIONS_BUDGETS_CHILD,
        ROOT_PARENT_DIRECT_ASSERTIONS_BUDGET_GUARD,
    ),
    (
        "status_mirrors",
        ROOT_PARENT_DIRECT_ASSERTIONS_STATUS_MIRRORS_CHILD,
        ROOT_PARENT_DIRECT_ASSERTIONS_STATUS_GUARD,
    ),
];

pub(super) fn assert_code_review_root_parent_is_folder_backed(sources: &CodeReviewFindingsSources) {
    parent_mounts::assert_code_review_root_parent_mounts_are_folder_backed(sources);
    backflow::assert_code_review_root_parent_moved_tests_do_not_backflow(sources);
}

pub(super) fn root_parent_direct_assertion_child_sources() -> Vec<(&'static str, String)> {
    ROOT_PARENT_DIRECT_ASSERTIONS_GUARD_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn root_parent_direct_assertion_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in root_parent_direct_assertion_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
