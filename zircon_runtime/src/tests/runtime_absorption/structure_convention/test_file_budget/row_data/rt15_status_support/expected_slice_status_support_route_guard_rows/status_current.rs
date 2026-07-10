use super::*;

pub(super) fn assert_route_guard_rows_status_is_current(child_blob: &str) {
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let frameworks_02 = read_repo(
        "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
    );
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let status_anchors = [
        ROUTE_GUARD_ROWS_ROW_DATA_OWNER_STATUS_NAME,
        ROUTE_GUARD_ROWS_ROW_DATA_OWNER_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/status_support_maps/route_guard_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/status_support_maps/route_guard_rows/runtime_index_anchor_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/status_support_maps/route_guard_rows/expected_slice_route_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/status_support_maps/route_guard_rows/route_input_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/status_support_maps/route_guard_rows/row_data_owner.rs",
        ROUTE_GUARD_ROWS_ROW_DATA_OWNER_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("status route guard row children", child_blob),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("Frameworks 02", frameworks_02.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "status map records route guard row-data owner split",
        &read_runtime_src(STATUS_SUPPORT_ROW_DATA_STATUS_MAP_PATH),
        &[
            ROUTE_GUARD_ROWS_ROW_DATA_OWNER_STATUS_NAME,
            ROUTE_GUARD_ROWS_ROW_DATA_OWNER_STATUS_ID,
        ],
    );
    assert_contains_all(
        "date map records route guard row-data owner split",
        &read_runtime_src(STATUS_SUPPORT_ROW_DATA_DATE_MAP_PATH),
        &[ROUTE_GUARD_ROWS_ROW_DATA_OWNER_STATUS_NAME, "2026-07-07"],
    );
}
