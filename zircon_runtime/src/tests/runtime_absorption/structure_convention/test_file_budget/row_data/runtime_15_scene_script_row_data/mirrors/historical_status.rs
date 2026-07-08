use super::*;

#[test]
fn runtime_15_scene_script_row_data_historical_status_is_current() {
    let row_data_owner = read_runtime_src(SCENE_SCRIPT_ROW_DATA_OWNER_PATH);
    let status_map = read_runtime_src(M3_STRUCTURE_SUPPORT_STATUS_MAP_PATH);
    let date_map = read_runtime_src(M3_STRUCTURE_SUPPORT_DATE_MAP_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    let child_owner_status_anchors = [
        CHILD_OWNER_STATUS_NAME,
        CHILD_OWNER_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/runtime_07_performance.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/script_vm_runtime.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/plugin_extension_tests.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/script_vm_gameplay_shader.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/scene_ecs_tests.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/scene_asset_world.rs",
        CHILD_OWNER_GUARD_NAME,
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        (
            "Runtime 15 scene-script status row data",
            row_data_owner.as_str(),
        ),
        (
            "Runtime 15 M3 structure expected status map",
            status_map.as_str(),
        ),
        (
            "Runtime 15 M3 structure expected date map",
            date_map.as_str(),
        ),
    ] {
        assert_contains_all(label, source, &child_owner_status_anchors);
    }
}
