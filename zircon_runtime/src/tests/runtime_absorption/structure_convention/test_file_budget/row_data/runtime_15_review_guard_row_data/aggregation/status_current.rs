use super::*;

#[test]
fn runtime_15_review_guard_row_data_aggregation_status_is_current() {
    let row_data_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/review_guard/row_data_rows.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/row_data_maps/review_guard_row_data_maps.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/row_data_maps/review_guard_row_data_maps.rs",
    );

    let status_anchors = [
        AGGREGATION_CHILD_SPLIT_STATUS_NAME,
        AGGREGATION_CHILD_SPLIT_STATUS_ID,
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/aggregation.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/aggregation/top_level_rows.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/aggregation/runtime_15_root.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/aggregation/runtime_15_m3_root.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/aggregation/review_guard_splits.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/aggregation/status_current.rs",
        AGGREGATION_CHILD_SPLIT_GUARD_NAME,
        "scoped rustfmt/static scans passed",
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "review-guard row-data aggregation child split is recorded in status rows",
        &row_data_rows,
        &status_anchors,
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
            "Frameworks 02 plan",
            "docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md",
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
            "session note",
            ".codex/sessions/20260612-0847-runtime-architecture-implementation.md",
        ),
    ] {
        let source = read_repo(path);
        assert_contains_all(label, &source, &status_anchors);
    }
    assert_contains_all(
        "Runtime 15 status-support expected status map records aggregation child split",
        &status_map,
        &[
            AGGREGATION_CHILD_SPLIT_STATUS_NAME,
            AGGREGATION_CHILD_SPLIT_STATUS_ID,
        ],
    );
    assert_contains_all(
        "Runtime 15 status-support expected date map records aggregation child split",
        &date_map,
        &[AGGREGATION_CHILD_SPLIT_STATUS_NAME, "2026-07-05"],
    );
}
