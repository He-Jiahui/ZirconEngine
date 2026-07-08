use super::super::super::super::*;
use super::*;

#[path = "ownership/budgets.rs"]
mod budgets;
#[path = "ownership/delegation.rs"]
mod delegation;
#[path = "ownership/entry_points.rs"]
mod entry_points;
#[path = "ownership/parent_absence.rs"]
mod parent_absence;
#[path = "ownership/status_mirrors.rs"]
mod status_mirrors;

pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_DELEGATION_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/ownership/delegation.rs";
pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_PARENT_ABSENCE_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/ownership/parent_absence.rs";
pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_ENTRY_POINTS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/ownership/entry_points.rs";
pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_BUDGETS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/ownership/budgets.rs";
pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_STATUS_MIRRORS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/ownership/status_mirrors.rs";
pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_SLICE: &str =
    "Runtime 15 M3 code review findings direct assertions child-ownership guard folder-backed split";
pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_STATUS: &str =
    "runtime_15_code_review_findings_direct_assertions_child_ownership_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_DATE: &str = "2026-07-04";
pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_GUARD: &str =
    "runtime_15_code_review_findings_direct_assertions_child_ownership_guard_is_folder_backed";
pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_STATUS_GUARD: &str =
    "runtime_15_code_review_findings_direct_assertions_child_ownership_guard_folder_backed_status_is_current";
pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_BUDGET_GUARD: &str =
    "runtime_15_code_review_findings_direct_assertions_child_ownership_children_line_budgets_are_current";

pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_DELEGATION_CHILD,
        "runtime_15_code_review_findings_direct_assertions_children_are_child_owned",
    ),
    (
        "parent_absence",
        DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_PARENT_ABSENCE_CHILD,
        DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_GUARD,
    ),
    (
        "entry_points",
        DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_ENTRY_POINTS_CHILD,
        "assert_direct_review_child_entry_points_are_current",
    ),
    (
        "budgets",
        DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_BUDGETS_CHILD,
        DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_BUDGET_GUARD,
    ),
    (
        "status_mirrors",
        DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_STATUS_MIRRORS_CHILD,
        DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_STATUS_GUARD,
    ),
];

pub(super) fn direct_assertion_child_ownership_child_sources() -> Vec<(&'static str, String)> {
    DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_GUARD_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn direct_assertion_child_ownership_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in direct_assertion_child_ownership_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
