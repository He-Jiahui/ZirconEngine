use super::*;

pub(super) fn assert_moved_code_review_row_sources_are_delegated() {
    let moved_rows_guard = read_runtime_src(MOVED_ROWS_PARENT_PATH);
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
}
