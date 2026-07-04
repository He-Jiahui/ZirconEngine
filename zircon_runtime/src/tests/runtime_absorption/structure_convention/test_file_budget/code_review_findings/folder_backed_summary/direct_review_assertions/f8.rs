use super::super::super::super::*;

use super::super::source_inventory::CodeReviewFindingsSources;

#[path = "f8/budgets.rs"]
mod budgets;
#[path = "f8/delegation.rs"]
mod delegation;
#[path = "f8/parent_mounts.rs"]
mod parent_mounts;
#[path = "f8/review_children.rs"]
mod review_children;
#[path = "f8/status_mirrors.rs"]
mod status_mirrors;

pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions.rs";
pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/child_ownership.rs";
pub(super) const F8_DIRECT_ASSERTIONS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f8.rs";
pub(super) const F8_DIRECT_ASSERTIONS_DELEGATION_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f8/delegation.rs";
pub(super) const F8_DIRECT_ASSERTIONS_PARENT_MOUNTS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f8/parent_mounts.rs";
pub(super) const F8_DIRECT_ASSERTIONS_REVIEW_CHILDREN_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f8/review_children.rs";
pub(super) const F8_DIRECT_ASSERTIONS_BUDGETS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f8/budgets.rs";
pub(super) const F8_DIRECT_ASSERTIONS_STATUS_MIRRORS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f8/status_mirrors.rs";
pub(super) const F8_DIRECT_ASSERTIONS_FOLDER_BACKED_SLICE: &str =
    "Runtime 15 M3 code review findings F8 direct assertions guard folder-backed split";
pub(super) const F8_DIRECT_ASSERTIONS_FOLDER_BACKED_STATUS: &str =
    "runtime_15_code_review_findings_f8_direct_assertions_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const F8_DIRECT_ASSERTIONS_FOLDER_BACKED_DATE: &str = "2026-07-04";
pub(super) const F8_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD: &str =
    "runtime_15_code_review_findings_f8_direct_assertions_guard_is_folder_backed";
pub(super) const F8_DIRECT_ASSERTIONS_STATUS_GUARD: &str =
    "runtime_15_code_review_findings_f8_direct_assertions_guard_folder_backed_status_is_current";
pub(super) const F8_DIRECT_ASSERTIONS_BUDGET_GUARD: &str =
    "runtime_15_code_review_findings_f8_direct_assertions_children_line_budgets_are_current";
pub(super) const CODE_REVIEW_FINDINGS_LINE_BUDGET: usize = 800;

const REVIEW_GUARD_STATUS_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/direct_assertion_rows.rs";
const REVIEW_GUARD_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs";
const REVIEW_GUARD_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs";

pub(super) const F8_DIRECT_ASSERTIONS_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        F8_DIRECT_ASSERTIONS_DELEGATION_CHILD,
        "runtime_15_code_review_findings_f8_direct_assertions_are_child_owner",
    ),
    (
        "parent_mounts",
        F8_DIRECT_ASSERTIONS_PARENT_MOUNTS_CHILD,
        F8_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD,
    ),
    (
        "review_children",
        F8_DIRECT_ASSERTIONS_REVIEW_CHILDREN_CHILD,
        "assert_f8_review_children_are_folder_backed",
    ),
    (
        "budgets",
        F8_DIRECT_ASSERTIONS_BUDGETS_CHILD,
        F8_DIRECT_ASSERTIONS_BUDGET_GUARD,
    ),
    (
        "status_mirrors",
        F8_DIRECT_ASSERTIONS_STATUS_MIRRORS_CHILD,
        F8_DIRECT_ASSERTIONS_STATUS_GUARD,
    ),
];

pub(super) fn assert_f8_direct_sources_are_folder_backed(sources: &CodeReviewFindingsSources) {
    parent_mounts::assert_f8_api_convergence_parent_mounts_child_owners(sources);
    review_children::assert_f8_review_children_are_folder_backed(sources);
}

pub(super) fn f8_direct_assertion_child_sources() -> Vec<(&'static str, String)> {
    F8_DIRECT_ASSERTIONS_GUARD_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn f8_direct_assertion_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in f8_direct_assertion_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
