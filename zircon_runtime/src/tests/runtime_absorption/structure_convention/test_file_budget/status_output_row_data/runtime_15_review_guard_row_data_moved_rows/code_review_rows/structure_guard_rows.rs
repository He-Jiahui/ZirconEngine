use super::*;

pub(super) fn assert_moved_structure_guard_rows_are_child_owned() {
    let foundation_guards = read_runtime_src(FOUNDATION_GUARDS_PATH);
    let review_guard_splits = read_runtime_src(REVIEW_GUARD_SPLITS_PATH);
    let review_guard_code_review_rows = read_runtime_src(CODE_REVIEW_ROWS_PATH);
    let review_guard_code_review_structure_guard_rows = read_runtime_src(STRUCTURE_GUARD_ROWS_PATH);
    let review_guard_code_review_structure_guard_child_rows = [
        read_runtime_src(STRUCTURE_GUARD_ROOT_AND_CHILDREN_PATH),
        read_runtime_src(STRUCTURE_GUARD_STATUS_DOCS_PATH),
        read_runtime_src(STRUCTURE_GUARD_FOLDER_BACKED_SUMMARY_PATH),
        read_runtime_src(STRUCTURE_GUARD_TYPED_ERROR_PATH),
        read_runtime_src(STRUCTURE_GUARD_ROW_DATA_OWNER_PATH),
    ]
    .concat();

    for moved_row in [
        "Runtime 15 M3 code review findings structure guard child-owner split",
        "Runtime 15 M3 code review findings status-doc guard child-owner split",
        "Runtime 15 M3 code review findings status-doc source anchors child-owner split",
        "Runtime 15 M3 code review findings status-doc status anchors child-owner split",
        "Runtime 15 M3 code review findings folder-backed summary child-owner split",
        "Runtime 15 M3 code review findings source inventory child-owner split",
        "Runtime 15 M3 code review findings structure guard folder-backed summary child-owner split",
        "Runtime 15 M3 code review findings structure guard typed-error child-owner split",
    ] {
        assert!(
            !foundation_guards.contains(moved_row),
            "foundation_guards.rs should delegate structure-guard row literal {moved_row}"
        );
        assert!(
            !review_guard_code_review_structure_guard_rows.contains(moved_row),
            "review_guard_splits/code_review_rows/structure_guard_rows.rs should route structure-guard row literal {moved_row}"
        );
        assert!(
            review_guard_code_review_structure_guard_child_rows.contains(moved_row),
            "review_guard_splits/code_review_rows/structure_guard_rows/* should own moved row literal {moved_row}"
        );
        assert!(
            !review_guard_splits.contains(moved_row),
            "review_guard_splits.rs should only route structure-guard row literal {moved_row}"
        );
        assert!(
            !review_guard_code_review_rows.contains(moved_row),
            "review_guard_splits/code_review_rows.rs should only route structure-guard row literal {moved_row}"
        );
    }
}
