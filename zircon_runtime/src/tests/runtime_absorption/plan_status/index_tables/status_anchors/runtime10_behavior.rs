use super::super::super::support::{assert_contains_all, runtime_numbered_archive_sources};

#[test]
fn runtime_15_runtime_10_behavior_status_index_anchors_are_locked() {
    let archive_source = runtime_numbered_archive_sources();
    let output_anchors = include_str!(
        "../../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_output_anchors.py"
    );
    let runtime_10_status_row_data = include_str!(
        "../../status_output_tables/expected_status_row_data/runtime_10_13/runtime_10/dynamic_api.rs"
    );
    let runtime_05_status_row_data = include_str!(
        "../../status_output_tables/expected_status_row_data/runtime_05/cross_runtime_rows.rs"
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

    let runtime_10_behavior_index_anchors = [
        "Runtime 10 Dynamic API 行为测试锚审计同步",
        "Runtime 05 status-output Runtime 10 behavior-test row",
        "behavior_test_anchor_count = 16",
        "missing_behavior_test_anchors = []",
        "standalone dynamic_api_session 9/9",
        "dynamic_api/app/UI gates pending",
    ];
    assert_contains_all(
        "runtime plan-status output anchor inventory",
        output_anchors,
        &runtime_10_behavior_index_anchors,
    );
    assert_contains_all(
        "runtime numbered archives",
        &archive_source,
        &runtime_10_behavior_index_anchors,
    );

    let runtime_10_behavior_row = runtime_10_status_row_data
        .split_once("\"Runtime 10 Dynamic API 行为测试锚审计同步\"")
        .expect("Runtime 10 row-data should keep the behavior-test anchor row")
        .1
        .split_once("),")
        .expect("Runtime 10 behavior-test anchor row should end as a tuple")
        .0;
    assert_contains_all(
        "Runtime 10 behavior row data",
        runtime_10_behavior_row,
        &runtime_10_behavior_index_anchors[2..],
    );
    assert_contains_all(
        "Runtime 05 status-output behavior row data",
        runtime_05_status_row_data,
        &runtime_10_behavior_index_anchors[..3],
    );
    assert_contains_all(
        "Runtime 05 status-output behavior row data",
        runtime_05_status_row_data,
        &["standalone status-output 2/2"],
    );

    let status_anchors = [
        "Runtime 15 M3 Runtime 10 behavior status anchor sync",
        "runtime_15_runtime_10_behavior_status_anchor_sync_static_passed_cargo_deferred",
        "runtime_15_runtime_10_behavior_status_index_anchors_are_locked",
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
        &runtime_10_behavior_index_anchors,
    );
}
