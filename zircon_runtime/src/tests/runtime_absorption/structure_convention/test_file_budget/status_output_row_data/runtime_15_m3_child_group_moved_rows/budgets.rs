use super::*;

#[test]
fn runtime_15_m3_child_group_moved_row_child_budgets_stay_focused() {
    for (_, child_path, _) in MOVED_ROW_GUARD_CHILDREN {
        let source = read_runtime_src(child_path);
        let line_count = source.lines().count();
        assert!(
            line_count < 180,
            "{child_path} should stay focused after M3 child-group moved-row folder-backed split; got {line_count} lines"
        );
    }

    for (path, budget) in [
        (CHILD_GROUPS_GUARD_PATH, 180),
        (MOVED_ROWS_GUARD_PATH, 70),
        (ROOT_PATHS_PATH, 120),
        (ROOT_STATUSES_PATH, 80),
        (ROOT_CHILD_ROWS_PATH, 140),
        (ROOT_SOURCE_BLOBS_PATH, 60),
        (ROOT_INVENTORY_GUARD_PATH, 100),
        (FOUNDATION_GUARDS_ROWS_PATH, 800),
        (LOCK_POISON_STATUS_ROWS_PATH, 800),
        (MODULE_CONVENTION_STATUS_ROWS_PATH, 800),
        (REVIEW_STATUS_SYNC_ROWS_PATH, 800),
        (STATUS_SUPPORT_STATUS_DOCS_ROWS_PATH, 280),
    ] {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < budget,
            "{path} should stay below the Runtime 15 M3 moved-row guard support budget of {budget} lines; got {line_count}"
        );
    }
}
