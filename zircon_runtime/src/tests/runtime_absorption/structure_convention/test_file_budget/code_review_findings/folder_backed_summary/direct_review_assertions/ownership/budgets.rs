use super::*;

pub(super) fn assert_direct_assertions_child_ownership_children_line_budgets_are_current() {
    for (path, source) in [
        (
            DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_CHILD,
            read_runtime_src(DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_CHILD),
        ),
        (
            DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_DELEGATION_CHILD,
            read_runtime_src(DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_DELEGATION_CHILD),
        ),
        (
            DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_PARENT_ABSENCE_CHILD,
            read_runtime_src(DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_PARENT_ABSENCE_CHILD),
        ),
        (
            DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_ENTRY_POINTS_CHILD,
            read_runtime_src(DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_ENTRY_POINTS_CHILD),
        ),
        (
            DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_BUDGETS_CHILD,
            read_runtime_src(DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_BUDGETS_CHILD),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < CODE_REVIEW_FINDINGS_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}

#[test]
fn runtime_15_code_review_findings_direct_assertions_child_ownership_children_line_budgets_are_current(
) {
    assert_direct_assertions_child_ownership_children_line_budgets_are_current();
}
