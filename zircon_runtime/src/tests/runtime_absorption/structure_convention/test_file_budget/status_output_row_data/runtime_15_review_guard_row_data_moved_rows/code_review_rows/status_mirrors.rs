use super::*;

#[test]
fn runtime_15_review_guard_moved_row_code_review_rows_child_split_status_is_current() {
    let status_rows = read_runtime_src(STATUS_SUPPORT_REVIEW_GUARD_ROWS_PATH);
    let status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);

    let status_anchors = [
        CODE_REVIEW_ROWS_CHILD_SPLIT_STATUS_NAME,
        CODE_REVIEW_ROWS_CHILD_SPLIT_STATUS_ID,
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/source_delegation.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/review_guard_rows.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/structure_guard_rows.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/typed_error_structure_rows.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/plugin_importer_rows.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/status_mirrors.rs",
        CODE_REVIEW_ROWS_CHILD_SPLIT_GUARD_NAME,
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "production guard review rows record moved-row code-review rows split",
        &status_rows,
        &status_anchors,
    );
    assert_contains_all(
        "M3 status-support map records moved-row code-review rows split",
        &status_map,
        &[
            CODE_REVIEW_ROWS_CHILD_SPLIT_STATUS_NAME,
            CODE_REVIEW_ROWS_CHILD_SPLIT_STATUS_ID,
        ],
    );
    assert_contains_all(
        "M3 status-support date map records moved-row code-review rows split",
        &date_map,
        &[CODE_REVIEW_ROWS_CHILD_SPLIT_STATUS_NAME, "2026-07-04"],
    );

    for (label, path) in [
        (
            "Runtime 15 plan",
            "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        ),
        (
            "Runtime index",
            "docs/plans/zircon_runtime/runtime/index.md",
        ),
        (
            "review findings",
            "docs/plans/engine-code-review-findings-2026-06.md",
        ),
        (
            "structure convention",
            "docs/plans/engine-code-structure-convention.md",
        ),
        (
            "module convention doc",
            "docs/zircon_runtime/structure/module-convention.md",
        ),
        (
            "runtime implementation session",
            ".codex/sessions/20260612-0847-runtime-architecture-implementation.md",
        ),
    ] {
        let source = read_repo(path);
        assert_contains_all(label, &source, &status_anchors);
    }
    assert_contains_all(
        "review-guard moved-row code-review rows child source blob reaches every child",
        &code_review_rows_child_source_blob(),
        &[
            "assert_moved_code_review_row_sources_are_delegated",
            "assert_moved_review_guard_rows_are_child_owned",
            "assert_moved_structure_guard_rows_are_child_owned",
            "assert_moved_typed_error_structure_rows_are_child_owned",
            "assert_moved_plugin_importer_rows_are_child_owned",
            "runtime_15_review_guard_moved_row_code_review_rows_child_split_status_is_current",
        ],
    );
}
