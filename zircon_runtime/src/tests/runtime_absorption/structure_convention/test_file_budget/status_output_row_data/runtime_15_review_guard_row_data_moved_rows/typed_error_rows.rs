use super::*;

#[test]
fn runtime_15_review_guard_moved_row_typed_error_rows_are_child_owned() {
    let moved_rows_guard = read_runtime_src(MOVED_ROWS_PARENT_PATH);
    let foundation_guards = read_runtime_src(FOUNDATION_GUARDS_PATH);
    let review_guard_splits = read_runtime_src(REVIEW_GUARD_SPLITS_PATH);
    let review_guard_typed_error_rows = read_runtime_src(TYPED_ERROR_ROWS_PATH);

    for moved_row_source in [
        concat!("let typed_error_", "review_guard_rows ="),
        concat!("for moved_row in typed_error_", "review_guard_rows"),
    ] {
        assert!(
            !moved_rows_guard.contains(moved_row_source),
            "review-guard moved-row parent should delegate source {moved_row_source}"
        );
    }

    for moved_row in [
        "Runtime 15 M3 native plugin loader typed-error review guard child-owner split",
        "Runtime 15 M3 scene world typed-error review guard child-owner split",
        "Runtime 15 M3 script host typed-error review guard child-owner split",
        "Runtime 15 M3 asset loader typed-error review guard child-owner split",
        "Runtime 15 M3 asset records typed-error review guard child-owner split",
        "Runtime 15 M3 shader prewarm CLI typed-error review guard child-owner split",
        "Runtime 15 M3 native ABI surfaces typed-error review guard child-owner split",
        "Runtime 15 M3 native plugin descriptor ABI typed-error review guard child-owner split",
        "Runtime 15 M3 UI input typed-error review guard child-owner split",
        "Runtime 15 M3 native manifest sources typed-error review guard child-owner split",
        "Runtime 15 M3 native live-host typed-error review guard child-owner split",
        "Runtime 15 M3 native live-host lifecycle-paths typed-error review guard child-owner split",
        "Runtime 15 M3 native live-host replay-runtime typed-error review guard child-owner split",
    ] {
        assert!(
            !foundation_guards.contains(moved_row),
            "foundation_guards.rs should delegate typed-error review row literal {moved_row}"
        );
        assert!(
            review_guard_typed_error_rows.contains(moved_row),
            "review_guard_splits/typed_error_rows.rs should own moved row literal {moved_row}"
        );
        assert!(
            !review_guard_splits.contains(moved_row),
            "review_guard_splits.rs should only route typed-error row literal {moved_row}"
        );
    }
}
