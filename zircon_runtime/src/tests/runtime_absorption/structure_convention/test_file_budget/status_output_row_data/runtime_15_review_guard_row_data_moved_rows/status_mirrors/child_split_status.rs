use super::*;

#[test]
fn runtime_15_review_guard_moved_row_status_mirror_children_are_child_owned() {
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs",
    );
    let status_support_expected_status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs",
    );
    let status_support_expected_date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs",
    );

    let status_anchors = [
        STATUS_MIRROR_CHILD_SPLIT_NAME,
        STATUS_MIRROR_CHILD_SPLIT_ID,
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/status_mirrors.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/status_mirrors/child_split_status.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/status_mirrors/moved_row_status.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/status_mirrors/folder_backed_status.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/status_mirrors/scope_budgets.rs",
        "runtime_15_review_guard_moved_row_status_mirror_children_are_child_owned",
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "production support row data records moved-row status-mirror child split",
        &status_rows,
        &status_anchors,
    );
    assert_contains_all(
        "Runtime 15 status-support expected status map records moved-row status-mirror child split",
        &status_support_expected_status_map,
        &[STATUS_MIRROR_CHILD_SPLIT_NAME, STATUS_MIRROR_CHILD_SPLIT_ID],
    );
    assert_contains_all(
        "Runtime 15 status-support expected date map records moved-row status-mirror child split",
        &status_support_expected_date_map,
        &[STATUS_MIRROR_CHILD_SPLIT_NAME, "2026-07-04"],
    );

    let doc_anchors = [
        STATUS_MIRROR_CHILD_SPLIT_NAME,
        STATUS_MIRROR_CHILD_SPLIT_ID,
        "runtime_15_review_guard_moved_row_status_mirror_children_are_child_owned",
    ];
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
        assert_contains_all(label, &source, &doc_anchors);
    }
}
