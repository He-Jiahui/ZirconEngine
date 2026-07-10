use super::super::super::support::assert_contains_all;

#[test]
fn runtime_15_runtime_03_module_doc_status_index_anchors_are_locked() {
    let index_source =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let runtime_03_output_records = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/03/2026-07-09-schedule-and-frame-loop-alignment-output-records.md"
    );
    let output_anchors = include_str!(
        "../../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_output_anchors.py"
    );
    let status_row_data = include_str!(
        "../../status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors/runtime_status_anchors.rs"
    );
    let status_map = [
        include_str!(
            "../../status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps.rs"
        ),
        include_str!(
            "../../status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps/runtime_status_anchor_maps.rs"
        ),
    ]
    .join("\n");
    let date_map = [
        include_str!(
            "../../status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps.rs"
        ),
        include_str!(
            "../../status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps/runtime_status_anchor_maps.rs"
        ),
    ]
    .join("\n");

    let runtime_03_index_anchors = [
        "schedule_frame_loop_boundary",
        "runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts",
        "frame schedule module-doc anchors 3/3",
        "guard/test files 11/11",
        "Runtime 03 guard anchors 14/14",
        "ecs_schedule/tests::time::/session/schedule_parallel Cargo gates",
    ];
    assert_contains_all(
        "runtime plan-status output anchor inventory",
        output_anchors,
        &runtime_03_index_anchors,
    );
    assert_contains_all("runtime index", index_source, &runtime_03_index_anchors);

    let status_anchors = [
        "Runtime 15 M3 Runtime 03 module-doc status index anchor sync",
        "runtime_15_runtime_03_module_doc_status_index_anchor_sync_static_passed_cargo_deferred",
        "runtime_15_runtime_03_module_doc_status_index_anchors_are_locked",
    ];
    for (label, source) in [
        ("Runtime 03 output records", runtime_03_output_records),
        ("Runtime 15 status row data", status_row_data),
        ("Runtime 15 expected status map", status_map.as_str()),
        ("Runtime 15 expected date map", date_map.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "Runtime 15 status row data",
        status_row_data,
        &runtime_03_index_anchors,
    );
}
