use super::super::super::*;
use super::*;

#[path = "ownership/budgets.rs"]
mod budgets;
#[path = "ownership/delegation.rs"]
mod delegation;
#[path = "ownership/direct_assertions.rs"]
mod direct_assertions;
#[path = "ownership/parent_absence.rs"]
mod parent_absence;
#[path = "ownership/source_inventory.rs"]
mod source_inventory_checks;
#[path = "ownership/status_mirrors.rs"]
mod status_mirrors;

pub(super) const FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/child_ownership.rs";
pub(super) const FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_DELEGATION_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/ownership/delegation.rs";
pub(super) const FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_PARENT_ABSENCE_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/ownership/parent_absence.rs";
pub(super) const FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_DIRECT_ASSERTIONS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/ownership/direct_assertions.rs";
pub(super) const FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_SOURCE_INVENTORY_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/ownership/source_inventory.rs";
pub(super) const FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_BUDGETS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/ownership/budgets.rs";
pub(super) const FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_STATUS_MIRRORS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/ownership/status_mirrors.rs";
pub(super) const FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_SLICE: &str =
    "Runtime 15 M3 code review findings folder-backed summary child-ownership guard folder-backed split";
pub(super) const FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_STATUS: &str =
    "runtime_15_code_review_findings_folder_backed_summary_child_ownership_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_DATE: &str = "2026-07-04";
pub(super) const FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_GUARD: &str =
    "runtime_15_code_review_findings_folder_backed_summary_child_ownership_guard_is_folder_backed";
pub(super) const FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_STATUS_GUARD: &str =
    "runtime_15_code_review_findings_folder_backed_summary_child_ownership_guard_folder_backed_status_is_current";
pub(super) const FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_BUDGET_GUARD: &str =
    "runtime_15_code_review_findings_folder_backed_summary_child_ownership_children_line_budgets_are_current";

pub(super) const FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_DELEGATION_CHILD,
        "runtime_15_code_review_findings_folder_backed_summary_children_are_child_owned",
    ),
    (
        "parent_absence",
        FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_PARENT_ABSENCE_CHILD,
        FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_GUARD,
    ),
    (
        "direct_assertions",
        FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_DIRECT_ASSERTIONS_CHILD,
        "assert_folder_backed_direct_review_assertion_children_are_current",
    ),
    (
        "source_inventory",
        FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_SOURCE_INVENTORY_CHILD,
        "assert_folder_backed_source_inventory_child_is_current",
    ),
    (
        "budgets",
        FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_BUDGETS_CHILD,
        FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_BUDGET_GUARD,
    ),
    (
        "status_mirrors",
        FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_STATUS_MIRRORS_CHILD,
        FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_STATUS_GUARD,
    ),
];

pub(super) fn folder_backed_summary_child_ownership_child_sources() -> Vec<(&'static str, String)> {
    FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_GUARD_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn folder_backed_summary_child_ownership_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in folder_backed_summary_child_ownership_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
