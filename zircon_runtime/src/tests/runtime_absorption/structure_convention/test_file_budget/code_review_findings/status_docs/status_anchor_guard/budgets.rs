use super::*;

pub(super) fn assert_status_anchor_guard_children_line_budgets_are_current() {
    for (path, source) in [
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchor_guard.rs",
            read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchor_guard.rs"),
        ),
        (
            STATUS_DOC_STATUS_ANCHOR_GUARD_CHILD_OWNERSHIP_CHILD,
            read_runtime_src(STATUS_DOC_STATUS_ANCHOR_GUARD_CHILD_OWNERSHIP_CHILD),
        ),
        (
            STATUS_DOC_STATUS_ANCHOR_GUARD_FOLDER_BACKING_CHILD,
            read_runtime_src(STATUS_DOC_STATUS_ANCHOR_GUARD_FOLDER_BACKING_CHILD),
        ),
        (
            STATUS_DOC_STATUS_ANCHOR_GUARD_BUDGETS_CHILD,
            read_runtime_src(STATUS_DOC_STATUS_ANCHOR_GUARD_BUDGETS_CHILD),
        ),
        (
            STATUS_DOC_STATUS_ANCHOR_GUARD_STATUS_MIRRORS_CHILD,
            read_runtime_src(STATUS_DOC_STATUS_ANCHOR_GUARD_STATUS_MIRRORS_CHILD),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 400,
            "{path} should stay below the focused status-anchor guard budget; got {line_count} lines"
        );
    }
}

#[test]
fn runtime_15_code_review_findings_status_docs_status_anchor_guard_children_line_budgets_are_current(
) {
    assert_status_anchor_guard_children_line_budgets_are_current();
}
