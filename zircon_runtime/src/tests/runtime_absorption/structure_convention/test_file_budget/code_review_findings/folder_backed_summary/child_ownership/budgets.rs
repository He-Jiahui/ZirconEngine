use super::*;

pub(super) fn assert_folder_backed_summary_child_ownership_children_line_budgets_are_current() {
    for (path, source) in [
        (
            FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_CHILD,
            read_runtime_src(FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_CHILD),
        ),
        (
            FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_DELEGATION_CHILD,
            read_runtime_src(FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_DELEGATION_CHILD),
        ),
        (
            FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_PARENT_ABSENCE_CHILD,
            read_runtime_src(FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_PARENT_ABSENCE_CHILD),
        ),
        (
            FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_DIRECT_ASSERTIONS_CHILD,
            read_runtime_src(FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_DIRECT_ASSERTIONS_CHILD),
        ),
        (
            FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_SOURCE_INVENTORY_CHILD,
            read_runtime_src(FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_SOURCE_INVENTORY_CHILD),
        ),
        (
            FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_BUDGETS_CHILD,
            read_runtime_src(FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_BUDGETS_CHILD),
        ),
        (
            FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_STATUS_MIRRORS_CHILD,
            read_runtime_src(FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_STATUS_MIRRORS_CHILD),
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
fn runtime_15_code_review_findings_folder_backed_summary_child_ownership_children_line_budgets_are_current(
) {
    assert_folder_backed_summary_child_ownership_children_line_budgets_are_current();
}
