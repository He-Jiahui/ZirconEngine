use super::super::super::super::*;

use super::super::source_inventory::CodeReviewFindingsSources;

#[path = "f12/budgets.rs"]
mod budgets;
#[path = "f12/delegation.rs"]
mod delegation;
#[path = "f12/review_guard.rs"]
mod review_guard;

pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions.rs";
pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/child_ownership.rs";
pub(super) const F12_DIRECT_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f12.rs";
pub(super) const F12_DIRECT_ASSERTIONS_DELEGATION_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f12/delegation.rs";
pub(super) const F12_DIRECT_ASSERTIONS_REVIEW_GUARD_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f12/review_guard.rs";
pub(super) const F12_DIRECT_ASSERTIONS_BUDGETS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f12/budgets.rs";
pub(super) const F12_DIRECT_ASSERTIONS_FOLDER_BACKED_SLICE: &str =
    "Runtime 15 M3 code review findings F12 direct assertions guard folder-backed split";
pub(super) const F12_DIRECT_ASSERTIONS_FOLDER_BACKED_STATUS: &str = "runtime_15_code_review_findings_f12_direct_assertions_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const F12_DIRECT_ASSERTIONS_FOLDER_BACKED_DATE: &str = "2026-07-04";
pub(super) const F12_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD: &str =
    "runtime_15_code_review_findings_f12_direct_assertions_guard_is_folder_backed";
pub(super) const F12_DIRECT_ASSERTIONS_BUDGET_GUARD: &str =
    "runtime_15_code_review_findings_f12_direct_assertions_children_line_budgets_are_current";
pub(super) const CODE_REVIEW_FINDINGS_LINE_BUDGET: usize = 800;

pub(super) const F12_DIRECT_ASSERTIONS_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        F12_DIRECT_ASSERTIONS_DELEGATION_CHILD,
        "runtime_15_code_review_findings_f12_direct_assertions_are_child_owner",
    ),
    (
        "review_guard",
        F12_DIRECT_ASSERTIONS_REVIEW_GUARD_CHILD,
        F12_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD,
    ),
    (
        "budgets",
        F12_DIRECT_ASSERTIONS_BUDGETS_CHILD,
        F12_DIRECT_ASSERTIONS_BUDGET_GUARD,
    ),
];

pub(super) fn assert_f12_direct_sources_are_folder_backed(sources: &CodeReviewFindingsSources) {
    review_guard::assert_f12_dead_code_review_guard_is_child_owned(sources);
}

pub(super) fn f12_direct_assertion_child_sources() -> Vec<(&'static str, String)> {
    F12_DIRECT_ASSERTIONS_GUARD_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn f12_direct_assertion_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in f12_direct_assertion_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
