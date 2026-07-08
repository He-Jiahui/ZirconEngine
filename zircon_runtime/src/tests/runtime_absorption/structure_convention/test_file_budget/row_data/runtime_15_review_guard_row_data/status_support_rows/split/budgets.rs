use super::*;

#[test]
fn runtime_15_review_guard_status_support_rows_guard_budgets_are_current() {
    for (path, budget) in [
        (STATUS_SUPPORT_ROWS_GUARD_PATH, 30),
        (STATUS_SUPPORT_ROWS_FOLDER_BACKED_CHILD_PATH, 35),
        (STATUS_SUPPORT_ROWS_FOLDER_BACKED_ROW_LAYOUT_CHILD_PATH, 80),
        (
            STATUS_SUPPORT_ROWS_FOLDER_BACKED_STATUS_CURRENT_CHILD_PATH,
            95,
        ),
        (
            STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_CHILD_PATH,
            40,
        ),
        (
            STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_ROUTE_MOUNTS_CHILD_PATH,
            95,
        ),
        (
            STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_STATUS_CURRENT_CHILD_PATH,
            120,
        ),
        (
            STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_BUDGETS_CHILD_PATH,
            75,
        ),
        (
            STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_SPLIT_LAYOUT_CHILD_PATH,
            170,
        ),
        (STATUS_SUPPORT_ROWS_ANCHOR_MIRROR_CLEANUP_CHILD_PATH, 35),
        (
            STATUS_SUPPORT_ROWS_ANCHOR_MIRROR_CLEANUP_ROW_CLEANUP_CHILD_PATH,
            75,
        ),
        (
            STATUS_SUPPORT_ROWS_ANCHOR_MIRROR_CLEANUP_STATUS_CURRENT_CHILD_PATH,
            95,
        ),
        (
            STATUS_SUPPORT_ROWS_ANCHOR_MIRROR_CLEANUP_SPLIT_LAYOUT_CHILD_PATH,
            160,
        ),
        (STATUS_SUPPORT_ROWS_SPLIT_LAYOUT_CHILD_PATH, 40),
        (STATUS_SUPPORT_ROWS_SPLIT_LAYOUT_ROUTE_MOUNTS_CHILD_PATH, 70),
        (
            STATUS_SUPPORT_ROWS_SPLIT_LAYOUT_STATUS_CURRENT_CHILD_PATH,
            95,
        ),
        (STATUS_SUPPORT_ROWS_SPLIT_LAYOUT_BUDGETS_CHILD_PATH, 70),
        (
            STATUS_SUPPORT_ROWS_SPLIT_LAYOUT_SPLIT_LAYOUT_CHILD_PATH,
            170,
        ),
    ] {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < budget,
            "{path} should stay under its focused status-support row-data guard budget of {budget}; got {line_count} lines"
        );
    }
}
