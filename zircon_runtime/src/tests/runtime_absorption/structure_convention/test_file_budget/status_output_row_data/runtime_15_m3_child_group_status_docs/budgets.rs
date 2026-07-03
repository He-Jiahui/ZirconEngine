use super::*;

#[test]
fn runtime_15_m3_child_group_status_docs_child_budgets_stay_focused() {
    for path in [
        CHILD_GROUPS_GUARD_PATH,
        STATUS_DOCS_GUARD_PATH,
        STATUS_ROW_DOCS_GUARD_PATH,
    ] {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < 180,
            "{path} should stay below the focused Runtime 15 M3 child-group status-doc guard budget; got {line_count} lines"
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
