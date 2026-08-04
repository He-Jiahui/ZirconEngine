use super::super::super::super::*;

use super::super::source_inventory::CodeReviewFindingsSources;

#[path = "p0/budgets.rs"]
mod budgets;
#[path = "p0/delegation.rs"]
mod delegation;
#[path = "p0/parent_mounts.rs"]
mod parent_mounts;
#[path = "p0/review_children.rs"]
mod review_children;

pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions.rs";
pub(super) const P0_DIRECT_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/p0.rs";
pub(super) const P0_DIRECT_ASSERTIONS_DELEGATION_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/p0/delegation.rs";
pub(super) const P0_DIRECT_ASSERTIONS_PARENT_MOUNTS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/p0/parent_mounts.rs";
pub(super) const P0_DIRECT_ASSERTIONS_REVIEW_CHILDREN_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/p0/review_children.rs";
pub(super) const P0_DIRECT_ASSERTIONS_BUDGETS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/p0/budgets.rs";
pub(super) const P0_DIRECT_ASSERTIONS_FOLDER_BACKED_SLICE: &str =
    "Runtime 15 M3 code review findings P0 direct assertions guard folder-backed split";
pub(super) const P0_DIRECT_ASSERTIONS_FOLDER_BACKED_STATUS: &str = "runtime_15_code_review_findings_p0_direct_assertions_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const P0_DIRECT_ASSERTIONS_FOLDER_BACKED_DATE: &str = "2026-07-04";
pub(super) const P0_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD: &str =
    "runtime_15_code_review_findings_p0_direct_assertions_guard_is_folder_backed";
pub(super) const P0_DIRECT_ASSERTIONS_BUDGET_GUARD: &str =
    "runtime_15_code_review_findings_p0_direct_assertions_children_line_budgets_are_current";
pub(super) const CODE_REVIEW_FINDINGS_LINE_BUDGET: usize = 800;

pub(super) const P0_DIRECT_ASSERTIONS_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        P0_DIRECT_ASSERTIONS_DELEGATION_CHILD,
        "runtime_15_code_review_findings_p0_direct_assertions_are_child_owner",
    ),
    (
        "parent_mounts",
        P0_DIRECT_ASSERTIONS_PARENT_MOUNTS_CHILD,
        P0_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD,
    ),
    (
        "review_children",
        P0_DIRECT_ASSERTIONS_REVIEW_CHILDREN_CHILD,
        "assert_p0_review_children_are_folder_backed",
    ),
    (
        "budgets",
        P0_DIRECT_ASSERTIONS_BUDGETS_CHILD,
        P0_DIRECT_ASSERTIONS_BUDGET_GUARD,
    ),
];

pub(super) fn assert_p0_direct_sources_are_folder_backed(sources: &CodeReviewFindingsSources) {
    parent_mounts::assert_p0_robustness_parent_mounts_child_owners(sources);
    review_children::assert_p0_review_children_are_folder_backed(sources);
}

pub(super) fn p0_direct_assertion_child_sources() -> Vec<(&'static str, String)> {
    P0_DIRECT_ASSERTIONS_GUARD_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn p0_direct_assertion_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in p0_direct_assertion_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
