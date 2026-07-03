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

    for path in [
        CHILD_GROUPS_GUARD_PATH,
        MOVED_ROWS_GUARD_PATH,
        FOUNDATION_GUARDS_ROWS_PATH,
        LOCK_POISON_STATUS_ROWS_PATH,
        MODULE_CONVENTION_STATUS_ROWS_PATH,
        REVIEW_STATUS_SYNC_ROWS_PATH,
        PRODUCTION_GUARD_SUPPORT_ROWS_PATH,
    ] {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 M3 moved-row guard support budget; got {line_count} lines"
        );
    }
}
