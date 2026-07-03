use super::*;

#[test]
fn runtime_15_review_guard_moved_row_code_review_rows_are_child_owned() {
    let moved_rows_guard = read_runtime_src(MOVED_ROWS_PARENT_PATH);
    let foundation_guards = read_runtime_src(FOUNDATION_GUARDS_PATH);
    let review_guard_splits = read_runtime_src(REVIEW_GUARD_SPLITS_PATH);
    let review_guard_code_review_rows = read_runtime_src(CODE_REVIEW_ROWS_PATH);
    let review_guard_code_review_review_guard_rows = read_runtime_src(REVIEW_GUARD_ROWS_PATH);
    let review_guard_plugin_importer_rows = read_runtime_src(PLUGIN_IMPORTER_ROWS_PATH);
    let review_guard_code_review_structure_guard_rows = read_runtime_src(STRUCTURE_GUARD_ROWS_PATH);
    let review_guard_code_review_structure_guard_child_rows = [
        read_runtime_src(STRUCTURE_GUARD_ROOT_AND_CHILDREN_PATH),
        read_runtime_src(STRUCTURE_GUARD_STATUS_DOCS_PATH),
        read_runtime_src(STRUCTURE_GUARD_FOLDER_BACKED_SUMMARY_PATH),
        read_runtime_src(STRUCTURE_GUARD_TYPED_ERROR_PATH),
        read_runtime_src(STRUCTURE_GUARD_ROW_DATA_OWNER_PATH),
    ]
    .concat();
    let review_guard_code_review_typed_error_structure_rows =
        read_runtime_src(TYPED_ERROR_STRUCTURE_ROWS_PATH);

    for moved_row_source in [
        concat!("let moved_code_review_", "review_guard_rows ="),
        concat!("let moved_code_review_", "structure_guard_rows ="),
        concat!("let moved_code_review_", "typed_error_structure_rows ="),
        concat!("let moved_", "plugin_importer_rows ="),
    ] {
        assert!(
            !moved_rows_guard.contains(moved_row_source),
            "review-guard moved-row parent should delegate source {moved_row_source}"
        );
    }

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
