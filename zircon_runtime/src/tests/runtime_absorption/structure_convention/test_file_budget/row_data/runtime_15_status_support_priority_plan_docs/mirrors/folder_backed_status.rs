use super::*;

#[test]
fn runtime_15_priority_plan_docs_folder_backed_status_rows_are_current() {
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/priority_plan_doc_maps/row_data_guard_maps.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/priority_plan_doc_maps/row_data_guard_maps.rs",
    );
    let production_guard_support = production_guard_support_priority_rows_source_blob();

    let folder_backed_status_anchors = [
        FOLDER_BACKED_STATUS_NAME,
        FOLDER_BACKED_STATUS_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs/delegation.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs/export_chain.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs/row_sources.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs/status_mirrors.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs/budgets.rs",
        FOLDER_BACKED_GUARD_NAME,
        HISTORICAL_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for (label, path) in [
        (
            "Runtime 15 plan",
            "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        ),
        (
            "runtime index",
            "docs/plans/zircon_runtime/runtime/index.md",
        ),
        (
            "structure convention plan",
            "docs/plans/engine-code-structure-convention.md",
        ),
        (
            "review findings plan",
            "docs/plans/engine-code-review-findings-2026-06.md",
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
        assert_contains_all(label, &source, &folder_backed_status_anchors);
    }
    assert_contains_all(
        "status-output Runtime 15 M3 production support row data records priority-plan-doc folder-backed status",
        &production_guard_support,
        &folder_backed_status_anchors,
    );
    assert_contains_all(
        "Runtime 15 status-support map owns priority-plan-doc row-data guard folder-backed split",
        &status_map,
        &[FOLDER_BACKED_STATUS_NAME, FOLDER_BACKED_STATUS_ID],
    );
    assert_contains_all(
        "Runtime 15 status-support date map owns priority-plan-doc row-data guard folder-backed split",
        &date_map,
        &[FOLDER_BACKED_STATUS_NAME, "2026-07-03"],
    );
}
