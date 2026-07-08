use super::*;

pub(super) fn assert_moved_review_guard_rows_are_child_owned() {
    let foundation_guards = read_runtime_src(FOUNDATION_GUARDS_PATH);
    let review_guard_splits = read_runtime_src(REVIEW_GUARD_SPLITS_PATH);
    let review_guard_code_review_rows = read_runtime_src(CODE_REVIEW_ROWS_PATH);
    let review_guard_code_review_review_guard_rows = read_runtime_src(REVIEW_GUARD_ROWS_PATH);

    for moved_row in [
        "Runtime 15 M3 code review findings test folder split",
        "Runtime 15 M3 P0 robustness review guard child-owner split",
        "Runtime 15 M3 F8 API convergence review guard child-owner split",
        "Runtime 15 M3 F8 descriptor review guard child-owner split",
        "Runtime 15 M3 late API cleanup review guard child-owner split",
    ] {
        assert!(
            !foundation_guards.contains(moved_row),
            "foundation_guards.rs should delegate review-guard row literal {moved_row}"
        );
        assert!(
            review_guard_code_review_review_guard_rows.contains(moved_row),
            "review_guard_splits/code_review_rows/review_guard_rows.rs should own moved row literal {moved_row}"
        );
        assert!(
            !review_guard_splits.contains(moved_row),
            "review_guard_splits.rs should only route code-review row literal {moved_row}"
        );
        assert!(
            !review_guard_code_review_rows.contains(moved_row),
            "review_guard_splits/code_review_rows.rs should only route code-review row literal {moved_row}"
        );
    }
}
