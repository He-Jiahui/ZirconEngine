use super::*;

pub(super) fn assert_f12_direct_assertions_children_line_budgets_are_current() {
    for (path, source) in [
        (
            DIRECT_REVIEW_ASSERTIONS_CHILD,
            read_runtime_src(DIRECT_REVIEW_ASSERTIONS_CHILD),
        ),
        (
            DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_CHILD,
            read_runtime_src(DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_CHILD),
        ),
        (
            F12_DIRECT_ASSERTIONS_CHILD,
            read_runtime_src(F12_DIRECT_ASSERTIONS_CHILD),
        ),
        (
            F12_DIRECT_ASSERTIONS_DELEGATION_CHILD,
            read_runtime_src(F12_DIRECT_ASSERTIONS_DELEGATION_CHILD),
        ),
        (
            F12_DIRECT_ASSERTIONS_REVIEW_GUARD_CHILD,
            read_runtime_src(F12_DIRECT_ASSERTIONS_REVIEW_GUARD_CHILD),
        ),
        (
            F12_DIRECT_ASSERTIONS_BUDGETS_CHILD,
            read_runtime_src(F12_DIRECT_ASSERTIONS_BUDGETS_CHILD),
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
fn runtime_15_code_review_findings_f12_direct_assertions_children_line_budgets_are_current() {
    assert_f12_direct_assertions_children_line_budgets_are_current();
}
