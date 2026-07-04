use super::super::super::super::super::*;
use super::*;

pub(super) fn assert_f12_dead_code_review_guard_is_child_owned(
    sources: &CodeReviewFindingsSources,
) {
    assert_contains_all(
        "F12 dead-code child owns production suppression review guard",
        &sources.f12_dead_code,
        &[
            "fn review_f12_runtime_production_dead_code_suppression_is_globally_gated",
            "runtime_15_production_sources_do_not_allow_dead_code_suppression",
            "runtime_15_f12_dead_code_review_status_sync_static_passed_cargo_deferred",
            "runtime_15_f12_dead_code_runtime_editor_boundary_status_static_passed_cargo_deferred",
            "Runtime production `allow(dead_code)` sweep is globally gated",
        ],
    );
}

#[test]
fn runtime_15_code_review_findings_f12_direct_assertions_guard_is_folder_backed() {
    let f12_parent = read_runtime_src(F12_DIRECT_ASSERTIONS_CHILD);
    let child_blob = f12_direct_assertion_child_source_blob();
    let sources = super::super::super::source_inventory::code_review_findings_sources();

    assert_f12_dead_code_review_guard_is_child_owned(&sources);
    budgets::assert_f12_direct_assertions_children_line_budgets_are_current();
    for (_, child_path, child_guard) in F12_DIRECT_ASSERTIONS_GUARD_CHILDREN {
        assert!(
            f12_parent.contains(child_path),
            "F12 direct assertions parent should inventory child path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "F12 direct assertions child source blob should contain child guard {child_guard}"
        );
    }
    assert!(
        !f12_parent.contains("F12 dead-code child owns production suppression review guard"),
        "f12.rs should delegate F12 dead-code review assertions to review_guard.rs"
    );
    assert_contains_all(
        "F12 direct assertions parent records folder-backed status",
        &f12_parent,
        &[
            F12_DIRECT_ASSERTIONS_FOLDER_BACKED_SLICE,
            F12_DIRECT_ASSERTIONS_FOLDER_BACKED_STATUS,
            F12_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD,
            F12_DIRECT_ASSERTIONS_STATUS_GUARD,
            F12_DIRECT_ASSERTIONS_BUDGET_GUARD,
        ],
    );
}
