use super::super::super::support::{assert_contains_all, runtime_numbered_archive_sources};

#[test]
fn runtime_15_runtime_07_owner_budget_status_index_anchors_are_locked() {
    let archive_source = runtime_numbered_archive_sources();
    let output_anchors = include_str!(
        "../../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_output_anchors.py"
    );
    let runtime_07_owner_budget_status_row_data = include_str!(
        "../../status_output_tables/expected_status_row_data/runtime_06_09/runtime_07/owner_budget.rs"
    );
    let runtime_15_status_row_data = include_str!(
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

    let runtime_07_owner_budget_index_anchors = [
        "Runtime 07 owner-budget 0-hotspot current audit sync",
        "`large_file_m1_gate_status = classified-and-clear`",
        "`large_file_hotspot_count = 0`",
        "`large_file_migration_debt_count = 0`",
        "`large_file_owner_class_count = 0`",
        "`large_file_unclassified_hotspot_count = 0`",
        "direct `performance_hotpath_boundary_audit` risks=0 / hotspots=0",
        "standalone `performance_hotspots.rs` exact owner-budget guards",
        "extract/ecs_query/performance profiling/FPS Cargo gates",
    ];
    assert_contains_all(
        "runtime plan-status output anchor inventory",
        output_anchors,
        &runtime_07_owner_budget_index_anchors,
    );
    assert_contains_all(
        "runtime numbered archives",
        &archive_source,
        &runtime_07_owner_budget_index_anchors,
    );

    let owner_budget_current_row = runtime_07_owner_budget_status_row_data
        .split_once("\"Runtime 07 owner-budget 0-hotspot current audit sync\"")
        .expect("Runtime 07 owner-budget row-data should keep the 0-hotspot current row")
        .1
        .split_once("),")
        .expect("Runtime 07 owner-budget 0-hotspot current row should end as a tuple")
        .0;
    assert_contains_all(
        "Runtime 07 owner-budget current row data",
        owner_budget_current_row,
        &runtime_07_owner_budget_index_anchors[1..8],
    );

    let status_anchors = [
        "Runtime 15 M3 Runtime 07 owner-budget status anchor sync",
        "runtime_15_runtime_07_owner_budget_status_anchor_sync_static_passed_cargo_deferred",
        "runtime_15_runtime_07_owner_budget_status_index_anchors_are_locked",
    ];
    for (label, source) in [
        ("runtime numbered archives", archive_source.as_str()),
        ("Runtime 15 status row data", runtime_15_status_row_data),
        ("Runtime 15 expected status map", status_map.as_str()),
        ("Runtime 15 expected date map", date_map.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "Runtime 15 status row data",
        runtime_15_status_row_data,
        &runtime_07_owner_budget_index_anchors,
    );
}
