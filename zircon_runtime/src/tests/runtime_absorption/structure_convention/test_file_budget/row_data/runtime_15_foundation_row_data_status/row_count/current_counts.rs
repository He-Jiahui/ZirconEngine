use super::*;

#[test]
fn runtime_15_foundation_row_data_docs_record_current_row_count() {
    let foundation_core_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation/core_rows.rs",
    );
    let foundation_typed_error_runtime_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation/typed_error_runtime_rows.rs",
    );
    let foundation_typed_error_plugin_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation/typed_error_plugin_rows.rs",
    );
    let foundation_typed_error_scene_asset_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation/typed_error_scene_asset_rows.rs",
    );
    let status_rows = read_runtime_src(STATUS_SUPPORT_ROWS_PATH);
    let expected_status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let expected_date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    let topic_counts = [
        runtime_15_row_count(&foundation_core_rows),
        runtime_15_row_count(&foundation_typed_error_runtime_rows),
        runtime_15_row_count(&foundation_typed_error_plugin_rows),
        runtime_15_row_count(&foundation_typed_error_scene_asset_rows),
    ];
    assert_eq!(
        [21, 23, 18, 11],
        topic_counts,
        "foundation topic row-data docs should mirror the actual current row distribution"
    );
    assert_eq!(
        73,
        topic_counts.iter().sum::<usize>(),
        "foundation topic child owners should preserve all 73 current Runtime 15 foundation status rows"
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("Runtime 15 status rows", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                ROW_COUNT_SYNC_NAME,
                ROW_COUNT_SYNC_ID,
                "21/23/18/11",
                "73",
                "runtime_15_foundation_row_data_docs_record_current_row_count",
            ],
        );
    }
    assert_contains_all(
        "Runtime 15 status map",
        &expected_status_map,
        &[ROW_COUNT_SYNC_NAME, ROW_COUNT_SYNC_ID],
    );
    assert_contains_all(
        "Runtime 15 date map",
        &expected_date_map,
        &[ROW_COUNT_SYNC_NAME, "2026-07-01"],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        for stale_count in [
            "70-row total",
            "20/23/18/9",
            "70 条 foundation",
            "70 条 Runtime 15 foundation",
            "70 条 row",
            "21/23/18/9",
            "71-row",
            "71 Runtime 15 foundation status rows",
            "合计 71 条",
            "合计 71 条 Runtime 15 foundation",
            "71 条 foundation row 总数",
            "21/23/18/10",
            "72 Runtime 15 foundation status rows",
            "合计 72 条",
            "72 条 Runtime 15 foundation",
            "72 条 foundation row 总数",
        ] {
            assert!(
                !source.contains(stale_count),
                "{label} should not retain stale Runtime 15 foundation row-data count {stale_count}"
            );
        }
    }
}
