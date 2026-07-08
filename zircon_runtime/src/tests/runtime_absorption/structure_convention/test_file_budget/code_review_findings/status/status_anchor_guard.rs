use super::super::super::*;
use super::*;

#[path = "status_anchor_guard/budgets.rs"]
mod budgets;
#[path = "status_anchor_guard/child_ownership.rs"]
mod child_ownership;
#[path = "status_anchor_guard/folder_backing.rs"]
mod folder_backing;
#[path = "status_anchor_guard/status_mirrors.rs"]
mod status_mirrors;

pub(super) const STATUS_DOC_STATUS_ANCHOR_GUARD_CHILD_OWNERSHIP_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status/status_anchor_guard/child_ownership.rs";
pub(super) const STATUS_DOC_STATUS_ANCHOR_GUARD_FOLDER_BACKING_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status/status_anchor_guard/folder_backing.rs";
pub(super) const STATUS_DOC_STATUS_ANCHOR_GUARD_BUDGETS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status/status_anchor_guard/budgets.rs";
pub(super) const STATUS_DOC_STATUS_ANCHOR_GUARD_STATUS_MIRRORS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status/status_anchor_guard/status_mirrors.rs";
pub(super) const STATUS_DOC_STATUS_ANCHOR_GUARD_FOLDER_BACKED_SLICE: &str =
    "Runtime 15 M3 code review findings status-doc status anchor guard folder-backed split";
pub(super) const STATUS_DOC_STATUS_ANCHOR_GUARD_FOLDER_BACKED_STATUS: &str =
    "runtime_15_code_review_findings_status_docs_status_anchor_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const STATUS_DOC_STATUS_ANCHOR_GUARD_FOLDER_BACKED_DATE: &str = "2026-07-04";
pub(super) const STATUS_DOC_STATUS_ANCHOR_GUARD_FOLDER_BACKED_GUARD: &str =
    "runtime_15_code_review_findings_status_docs_status_anchor_guard_is_folder_backed";
pub(super) const STATUS_DOC_STATUS_ANCHOR_GUARD_STATUS_GUARD: &str =
    "runtime_15_code_review_findings_status_docs_status_anchor_guard_folder_backed_status_is_current";
pub(super) const STATUS_DOC_STATUS_ANCHOR_GUARD_BUDGET_GUARD: &str =
    "runtime_15_code_review_findings_status_docs_status_anchor_guard_children_line_budgets_are_current";

pub(super) const STATUS_DOC_STATUS_ANCHOR_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "child_ownership",
        STATUS_DOC_STATUS_ANCHOR_GUARD_CHILD_OWNERSHIP_CHILD,
        "runtime_15_code_review_findings_status_docs_status_anchors_are_child_owner",
    ),
    (
        "folder_backing",
        STATUS_DOC_STATUS_ANCHOR_GUARD_FOLDER_BACKING_CHILD,
        STATUS_DOC_STATUS_ANCHOR_GUARD_FOLDER_BACKED_GUARD,
    ),
    (
        "budgets",
        STATUS_DOC_STATUS_ANCHOR_GUARD_BUDGETS_CHILD,
        STATUS_DOC_STATUS_ANCHOR_GUARD_BUDGET_GUARD,
    ),
    (
        "status_mirrors",
        STATUS_DOC_STATUS_ANCHOR_GUARD_STATUS_MIRRORS_CHILD,
        STATUS_DOC_STATUS_ANCHOR_GUARD_STATUS_GUARD,
    ),
];

pub(super) fn status_anchor_guard_child_sources() -> Vec<(&'static str, String)> {
    STATUS_DOC_STATUS_ANCHOR_GUARD_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn status_anchor_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in status_anchor_guard_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
