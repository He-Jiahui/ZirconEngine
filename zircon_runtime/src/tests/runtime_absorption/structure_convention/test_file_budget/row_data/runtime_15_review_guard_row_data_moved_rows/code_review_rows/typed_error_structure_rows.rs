use super::*;

pub(super) fn assert_moved_typed_error_structure_rows_are_child_owned() {
    let foundation_guards = read_runtime_src(FOUNDATION_GUARDS_PATH);
    let review_guard_splits = read_runtime_src(REVIEW_GUARD_SPLITS_PATH);
    let review_guard_code_review_rows = read_runtime_src(CODE_REVIEW_ROWS_PATH);
    let review_guard_code_review_typed_error_structure_rows = [
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/core_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/folder_backed.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/structure_assertion_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/top_level.rs",
    ]
    .iter()
    .map(|path| read_runtime_src(path))
    .collect::<Vec<_>>()
    .join("\n");

    for moved_row in [
        "Runtime 15 M3 code review findings typed-error structure guard child-owner split",
        "Runtime 15 M3 typed-error structure assertions guard child-owner split",
        "Runtime 15 M3 typed-error native plugin loader structure guard child-owner split",
        "Runtime 15 M3 typed-error structure moved-guard absence child-owner split",
    ] {
        assert!(
            !foundation_guards.contains(moved_row),
            "foundation_guards.rs should delegate typed-error structure row literal {moved_row}"
        );
        assert!(
            review_guard_code_review_typed_error_structure_rows.contains(moved_row),
            "review_guard_splits/code_review_rows/typed_error_structure_rows.rs should own moved row literal {moved_row}"
        );
        assert!(
            !review_guard_splits.contains(moved_row),
            "review_guard_splits.rs should only route typed-error structure row literal {moved_row}"
        );
        assert!(
            !review_guard_code_review_rows.contains(moved_row),
            "review_guard_splits/code_review_rows.rs should only route typed-error structure row literal {moved_row}"
        );
    }
}
