use super::*;

#[test]
fn runtime_15_review_guard_status_support_folder_backed_guard_budgets_are_current() {
    for (path, budget) in [
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
    ] {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < budget,
            "{path} should stay under its focused status-support folder-backed budget of {budget}; got {line_count} lines"
        );
    }
}
