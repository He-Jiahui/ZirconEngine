use super::*;

pub(super) fn assert_root_parent_direct_assertions_children_line_budgets_are_current() {
    for (path, source) in [
        (
            DIRECT_REVIEW_ASSERTIONS_CHILD,
            read_runtime_src(DIRECT_REVIEW_ASSERTIONS_CHILD),
        ),
        (
            ROOT_PARENT_DIRECT_ASSERTIONS_CHILD,
            read_runtime_src(ROOT_PARENT_DIRECT_ASSERTIONS_CHILD),
        ),
        (
            ROOT_PARENT_DIRECT_ASSERTIONS_DELEGATION_CHILD,
            read_runtime_src(ROOT_PARENT_DIRECT_ASSERTIONS_DELEGATION_CHILD),
        ),
        (
            ROOT_PARENT_DIRECT_ASSERTIONS_PARENT_MOUNTS_CHILD,
            read_runtime_src(ROOT_PARENT_DIRECT_ASSERTIONS_PARENT_MOUNTS_CHILD),
        ),
        (
            ROOT_PARENT_DIRECT_ASSERTIONS_BACKFLOW_CHILD,
            read_runtime_src(ROOT_PARENT_DIRECT_ASSERTIONS_BACKFLOW_CHILD),
        ),
        (
            ROOT_PARENT_DIRECT_ASSERTIONS_BUDGETS_CHILD,
            read_runtime_src(ROOT_PARENT_DIRECT_ASSERTIONS_BUDGETS_CHILD),
        ),
        (
            ROOT_PARENT_DIRECT_ASSERTIONS_STATUS_MIRRORS_CHILD,
            read_runtime_src(ROOT_PARENT_DIRECT_ASSERTIONS_STATUS_MIRRORS_CHILD),
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
fn runtime_15_code_review_findings_root_parent_direct_assertions_children_line_budgets_are_current()
{
    assert_root_parent_direct_assertions_children_line_budgets_are_current();
}
