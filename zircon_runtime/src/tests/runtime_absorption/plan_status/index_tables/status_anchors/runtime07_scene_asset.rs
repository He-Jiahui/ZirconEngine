use super::super::super::support::{assert_contains_all, runtime_numbered_archive_sources};

#[test]
fn runtime_15_runtime_07_scene_asset_status_index_anchors_are_locked() {
    let archive_source = runtime_numbered_archive_sources();
    let output_anchors = include_str!(
        "../../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_output_anchors.py"
    );
    let runtime_07_scene_status_row_data = include_str!(
        "../../status_output_tables/expected_status_row_data/runtime_06_09/runtime_07/scene_asset.rs"
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

    let runtime_07_scene_index_anchors = [
        "Runtime 07 scene asset owner split",
        "Runtime 07 scene asset split-drift repair",
        "Runtime 07 scene asset folder-split public-surface guard",
        "Runtime 07 scene asset guard 纳入 performance_hotpath_boundary",
        "mirror_docs_static_passed_cargo_pending",
        "split_drift_static_passed_cargo_deferred_active_lanes",
        "folder_split_guard_static_passed_cargo_deferred_active_lanes",
        "boundary_guard_anchor_static_passed_cargo_deferred_active_lanes",
        "hotspot_guard_anchor_count = 20",
        "standalone `rustc --edition 2021 --test ...performance_hotspots.rs` 通过 6/6",
        "`scene_asset` / Runtime 07 Cargo gates",
    ];
    assert_contains_all(
        "runtime plan-status output anchor inventory",
        output_anchors,
        &runtime_07_scene_index_anchors,
    );
    assert_contains_all(
        "runtime numbered archives",
        &archive_source,
        &runtime_07_scene_index_anchors,
    );

    let runtime_07_scene_guard_anchors = [
        "Runtime 07 scene asset owner split",
        "Runtime 07 scene asset split-drift repair",
        "Runtime 07 scene asset folder-split public-surface guard",
        "Runtime 07 scene asset guard 纳入 performance_hotpath_boundary",
        "split_drift_static_passed_cargo_deferred_active_lanes",
        "folder_split_guard_static_passed_cargo_deferred_active_lanes",
        "boundary_guard_anchor_static_passed_cargo_deferred_active_lanes",
        "hotspot_guard_anchor_count = 20",
        "standalone `rustc --edition 2021 --test ...performance_hotspots.rs` 通过 6/6",
        "`scene_asset` / Runtime 07 Cargo gates",
    ];
    assert_contains_all(
        "Runtime 07 scene status row data",
        runtime_07_scene_status_row_data,
        &runtime_07_scene_guard_anchors,
    );

    let status_anchors = [
        "Runtime 15 M3 Runtime 07 scene asset status anchor sync",
        "runtime_15_runtime_07_scene_asset_status_anchor_sync_static_passed_cargo_deferred",
        "runtime_15_runtime_07_scene_asset_status_index_anchors_are_locked",
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
        &runtime_07_scene_guard_anchors[4..],
    );
}
