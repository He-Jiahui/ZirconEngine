use super::*;

#[test]
fn runtime_15_review_guard_moved_row_status_mirror_scope_budgets_are_focused() {
    for (path, source) in moved_row_child_sources().into_iter().chain([(
        MOVED_ROWS_PARENT_PATH,
        read_runtime_src(MOVED_ROWS_PARENT_PATH),
    )]) {
        let line_count = source.lines().count();
        assert!(
            line_count < 400,
            "{path} should stay below the focused Runtime 15 review row-data moved-row guard budget; got {line_count} lines"
        );
    }

    for path in [
        FOUNDATION_GUARDS_PATH,
        REVIEW_GUARD_SPLITS_PATH,
        CODE_REVIEW_ROWS_PATH,
        REVIEW_GUARD_ROWS_PATH,
        PLUGIN_IMPORTER_ROWS_PATH,
        STRUCTURE_GUARD_ROWS_PATH,
        TYPED_ERROR_STRUCTURE_ROWS_PATH,
        TYPED_ERROR_ROWS_PATH,
    ] {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the focused Runtime 15 review row-data moved-row guard budget; got {line_count} lines"
        );
    }
}
