use super::super::root_statuses::{
    EXPORT_CHAIN_CHILD_SPLIT_GUARD_NAME, EXPORT_CHAIN_CHILD_SPLIT_STATUS_ID,
    EXPORT_CHAIN_CHILD_SPLIT_STATUS_NAME,
};
use super::*;

#[test]
fn runtime_15_review_guard_direct_assertion_export_chain_status_is_current() {
    let row_data_owner_rows = read_runtime_src(DIRECT_ASSERTION_ROW_DATA_OWNER_ROWS_PATH);
    let status_support_expected_status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/row_data_maps/review_guard_row_data_maps.rs",
    );
    let status_support_expected_date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/row_data_maps/review_guard_row_data_maps.rs",
    );

    let status_anchors = [
        EXPORT_CHAIN_CHILD_SPLIT_STATUS_NAME,
        EXPORT_CHAIN_CHILD_SPLIT_STATUS_ID,
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/export_chain.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/export_chain/review_guard_row_data.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/export_chain/code_review_rows.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/export_chain/review_guard_splits.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/export_chain/runtime_aggregation.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/export_chain/status_current.rs",
        EXPORT_CHAIN_CHILD_SPLIT_GUARD_NAME,
        "scoped rustfmt/static scans passed",
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "direct-assertion export-chain child split is recorded in status rows",
        &row_data_owner_rows,
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
            "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
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
        "Runtime 15 status-support expected status map records export-chain child split",
        &status_support_expected_status_map,
        &[
            EXPORT_CHAIN_CHILD_SPLIT_STATUS_NAME,
            EXPORT_CHAIN_CHILD_SPLIT_STATUS_ID,
        ],
    );
    assert_contains_all(
        "Runtime 15 status-support expected date map records export-chain child split",
        &status_support_expected_date_map,
        &[EXPORT_CHAIN_CHILD_SPLIT_STATUS_NAME, "2026-07-05"],
    );
}
