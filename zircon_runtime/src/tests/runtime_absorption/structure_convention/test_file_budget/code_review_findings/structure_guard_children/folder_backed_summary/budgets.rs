use super::super::super::super::*;
use super::*;

pub(super) fn assert_folder_backed_summary_structure_line_budgets() {
    for (path, source) in [
        (
            STRUCTURE_GUARD_CHILD_OWNER,
            read_runtime_src(STRUCTURE_GUARD_CHILD_OWNER),
        ),
        (
            FOLDER_BACKED_SUMMARY_STRUCTURE_CHILD_OWNER,
            read_runtime_src(FOLDER_BACKED_SUMMARY_STRUCTURE_CHILD_OWNER),
        ),
        (
            FOLDER_BACKED_SUMMARY_CHILD_OWNER,
            read_runtime_src(FOLDER_BACKED_SUMMARY_CHILD_OWNER),
        ),
        (
            FOLDER_BACKED_SUMMARY_DELEGATION_CHILD_OWNER,
            read_runtime_src(FOLDER_BACKED_SUMMARY_DELEGATION_CHILD_OWNER),
        ),
        (
            FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_CHILD_OWNER,
            read_runtime_src(FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_CHILD_OWNER),
        ),
        (
            FOLDER_BACKED_SUMMARY_STATUS_MIRRORS_CHILD_OWNER,
            read_runtime_src(FOLDER_BACKED_SUMMARY_STATUS_MIRRORS_CHILD_OWNER),
        ),
        (
            FOLDER_BACKED_SUMMARY_DIRECT_ASSERTIONS_CHILD_OWNER,
            read_runtime_src(FOLDER_BACKED_SUMMARY_DIRECT_ASSERTIONS_CHILD_OWNER),
        ),
        (
            FOLDER_BACKED_SUMMARY_SOURCE_INVENTORY_CHILD_OWNER,
            read_runtime_src(FOLDER_BACKED_SUMMARY_SOURCE_INVENTORY_CHILD_OWNER),
        ),
    ]
    .into_iter()
    .chain(folder_backed_summary_structure_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}

#[test]
fn runtime_15_code_review_findings_structure_guard_folder_backed_summary_children_line_budgets_are_current(
) {
    assert_folder_backed_summary_structure_line_budgets();
}
