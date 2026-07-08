use super::*;

#[test]
fn runtime_15_m2_row_data_children_historical_status_is_current() {
    let row_data_owner = read_runtime_src(RUNTIME_15_M2_ROW_DATA_OWNER_PATH);
    let status_map = read_runtime_src(NAMING_BOUNDARY_STATUS_MAP_PATH);
    let date_map = read_runtime_src(NAMING_BOUNDARY_DATE_MAP_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    let historical_status_anchors = [
        ROW_DATA_OWNER_STATUS_NAME,
        ROW_DATA_OWNER_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m2.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m2/core_scene_asset_dynamic.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m2/render_graphics.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m2/ui_platform_editor.rs",
        ROW_DATA_OWNER_GUARD_NAME,
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        ("Runtime 15 M2 status row data", row_data_owner.as_str()),
        (
            "Runtime 15 naming-boundary expected status map",
            status_map.as_str(),
        ),
        (
            "Runtime 15 naming-boundary expected date map",
            date_map.as_str(),
        ),
    ] {
        assert_contains_all(label, source, &historical_status_anchors);
    }
}
