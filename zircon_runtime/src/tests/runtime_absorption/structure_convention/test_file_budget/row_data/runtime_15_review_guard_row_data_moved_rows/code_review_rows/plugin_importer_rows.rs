use super::*;

pub(super) fn assert_moved_plugin_importer_rows_are_child_owned() {
    let foundation_guards = read_runtime_src(FOUNDATION_GUARDS_PATH);
    let review_guard_splits = read_runtime_src(REVIEW_GUARD_SPLITS_PATH);
    let review_guard_code_review_rows = read_runtime_src(CODE_REVIEW_ROWS_PATH);
    let review_guard_plugin_importer_rows = read_runtime_src(PLUGIN_IMPORTER_ROWS_PATH);

    for moved_row in [
        "Runtime 15 M3 code review findings plugin-importer DX structure guard child-owner split",
        "Runtime 15 M3 plugin-importer DX review guard child-owner split",
        "Runtime 15 M3 plugin-importer D13 SDK review guard child-owner split",
    ] {
        assert!(
            !foundation_guards.contains(moved_row),
            "foundation_guards.rs should delegate plugin-importer row literal {moved_row}"
        );
        assert!(
            review_guard_plugin_importer_rows.contains(moved_row),
            "review_guard_splits/code_review_rows/plugin_importer_rows.rs should own moved row literal {moved_row}"
        );
        assert!(
            !review_guard_splits.contains(moved_row),
            "review_guard_splits.rs should only route plugin-importer row literal {moved_row}"
        );
        assert!(
            !review_guard_code_review_rows.contains(moved_row),
            "review_guard_splits/code_review_rows.rs should only route plugin-importer row literal {moved_row}"
        );
    }
}
