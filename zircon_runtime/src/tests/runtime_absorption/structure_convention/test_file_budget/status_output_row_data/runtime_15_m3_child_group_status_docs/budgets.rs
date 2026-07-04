use super::*;

#[test]
fn runtime_15_m3_child_group_status_docs_child_budgets_stay_focused() {
    for (path, budget) in [
        (CHILD_GROUPS_GUARD_PATH, 180),
        (STATUS_DOCS_GUARD_PATH, 70),
        (STATUS_ROW_DOCS_GUARD_PATH, 180),
        (ROOT_PATHS_PATH, 90),
        (ROOT_STATUSES_PATH, 90),
        (ROOT_CHILD_ROWS_PATH, 120),
        (ROOT_SOURCE_BLOBS_PATH, 80),
        (ROOT_INVENTORY_GUARD_PATH, 100),
        (STATUS_SUPPORT_STATUS_DOCS_ROWS_PATH, 260),
    ] {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < budget,
            "{path} should stay below the focused Runtime 15 M3 child-group status-doc guard budget of {budget} lines; got {line_count} lines"
        );
    }

    for (_, child_path, _) in STATUS_DOCS_GUARD_CHILDREN {
        let source = read_runtime_src(child_path);
        let line_count = source.lines().count();
        assert!(
            line_count < 130,
            "{child_path} should stay focused after M3 child-group status-doc folder-backed split; got {line_count} lines"
        );
    }
}
