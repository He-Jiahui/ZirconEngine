use super::super::super::super::*;

use super::super::source_inventory::CodeReviewFindingsSources;

#[path = "render/budgets.rs"]
mod budgets;
#[path = "render/delegation.rs"]
mod delegation;
#[path = "render/review_guard.rs"]
mod review_guard;

pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions.rs";
pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/child_ownership.rs";
pub(super) const RENDER_DIRECT_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/render.rs";
pub(super) const RENDER_DIRECT_ASSERTIONS_DELEGATION_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/render/delegation.rs";
pub(super) const RENDER_DIRECT_ASSERTIONS_REVIEW_GUARD_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/render/review_guard.rs";
pub(super) const RENDER_DIRECT_ASSERTIONS_BUDGETS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/render/budgets.rs";
pub(super) const RENDER_DIRECT_ASSERTIONS_FOLDER_BACKED_SLICE: &str =
    "Runtime 15 M3 code review findings render direct assertions guard folder-backed split";
pub(super) const RENDER_DIRECT_ASSERTIONS_FOLDER_BACKED_STATUS: &str = "runtime_15_code_review_findings_render_direct_assertions_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const RENDER_DIRECT_ASSERTIONS_FOLDER_BACKED_DATE: &str = "2026-07-04";
pub(super) const RENDER_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD: &str =
    "runtime_15_code_review_findings_render_direct_assertions_guard_is_folder_backed";
pub(super) const RENDER_DIRECT_ASSERTIONS_BUDGET_GUARD: &str =
    "runtime_15_code_review_findings_render_direct_assertions_children_line_budgets_are_current";
pub(super) const CODE_REVIEW_FINDINGS_LINE_BUDGET: usize = 800;

pub(super) const RENDER_DIRECT_ASSERTIONS_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        RENDER_DIRECT_ASSERTIONS_DELEGATION_CHILD,
        "runtime_15_code_review_findings_render_direct_assertions_are_child_owner",
    ),
    (
        "review_guard",
        RENDER_DIRECT_ASSERTIONS_REVIEW_GUARD_CHILD,
        RENDER_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD,
    ),
    (
        "budgets",
        RENDER_DIRECT_ASSERTIONS_BUDGETS_CHILD,
        RENDER_DIRECT_ASSERTIONS_BUDGET_GUARD,
    ),
];

pub(super) fn assert_render_direct_sources_are_folder_backed(sources: &CodeReviewFindingsSources) {
    review_guard::assert_render_compiled_scene_review_guard_is_child_owned(sources);
}

pub(super) fn render_direct_assertion_child_sources() -> Vec<(&'static str, String)> {
    RENDER_DIRECT_ASSERTIONS_GUARD_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn render_direct_assertion_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in render_direct_assertion_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
