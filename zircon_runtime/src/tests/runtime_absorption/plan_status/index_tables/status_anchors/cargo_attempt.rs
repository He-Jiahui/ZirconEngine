use super::super::super::support::{assert_contains_all, runtime_numbered_archive_sources};

#[test]
fn runtime_15_runtime_cargo_attempt_status_index_anchors_are_locked() {
    let archive_source = runtime_numbered_archive_sources();
    let output_anchors = include_str!(
        "../../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_output_anchors.py"
    );
    let runtime_05_audit_metadata = include_str!(
        "../../status_output_tables/expected_status_row_data/runtime_05/audit_metadata/plan_coverage_rows.rs"
    );
    let runtime_14_cargo_row_data = include_str!(
        "../../status_output_tables/expected_status_row_data/runtime_14/cargo_gates.rs"
    );
    let runtime_15_status_row_data = include_str!(
        "../../status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors/cargo_attempt.rs"
    );
    let status_map = [
        include_str!(
            "../../status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps.rs"
        ),
        include_str!(
            "../../status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps/cargo_attempt_maps.rs"
        ),
    ]
    .join("\n");
    let date_map = [
        include_str!(
            "../../status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps.rs"
        ),
        include_str!(
            "../../status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps/cargo_attempt_maps.rs"
        ),
    ]
    .join("\n");

    let cargo_attempt_status_anchors = [
        "cargo_deferred_active_lane",
        "cargo_blocked_external_compile_drift",
        "cargo_recheck_blocked_external_ui_compile_drift",
        "cargo_recheck_timeout_no_result",
        "Runtime 14 Cargo 验证窗口探测",
        "Runtime 14 animation Cargo gate 尝试",
        "Runtime 14 animation Cargo gate 修复与复验阻塞",
        "Runtime 14 animation runtime-status focused recheck timeout",
    ];
    let cargo_attempt_evidence_anchors = [
        "cargo test -p zircon_runtime --lib animation --locked",
        "runtime_status_reports_player_rig_and_gpu_readiness",
        "共享 lib-test 编译层",
        "SKINNED_MESH_MAX_JOINT_MATRICES",
        "ViewportCameraSnapshot.temporal_jitter",
        "31 passed; 3 failed",
        "AnimationPlayerRuntimeStatus::sanitized_snapshot",
        "UiInputDispatchDiagnostics.capture_started",
        "default_interactions/table.rs:257",
        "904s",
        "无测试结果",
        "cargo/rustc processes were stopped",
    ];
    assert_contains_all(
        "runtime plan-status output anchor inventory",
        output_anchors,
        &cargo_attempt_status_anchors,
    );
    assert_contains_all(
        "runtime plan-status output anchor inventory",
        output_anchors,
        &cargo_attempt_evidence_anchors,
    );
    assert_contains_all(
        "runtime numbered archives",
        &archive_source,
        &cargo_attempt_status_anchors,
    );
    assert_contains_all(
        "runtime numbered archives",
        &archive_source,
        &cargo_attempt_evidence_anchors,
    );
    assert_contains_all(
        "Runtime 14 cargo row data",
        runtime_14_cargo_row_data,
        &cargo_attempt_status_anchors[1..],
    );
    assert_contains_all(
        "Runtime 14 cargo row data",
        runtime_14_cargo_row_data,
        &cargo_attempt_evidence_anchors,
    );
    assert_contains_all(
        "Runtime 05 audit metadata row data",
        runtime_05_audit_metadata,
        &[
            "cargo_attempt_status_anchor_count = 20",
            "cargo_attempt_status_guard_present = true",
            "cargo_blocked_external_compile_drift",
            "cargo_recheck_blocked_external_ui_compile_drift",
            "cargo_recheck_timeout_no_result",
            "cargo/rustc processes were stopped",
        ],
    );

    let status_anchors = [
        "Runtime 15 M3 Runtime Cargo attempt status anchor sync",
        "runtime_15_runtime_cargo_attempt_status_anchor_sync_static_passed_cargo_deferred",
        "runtime_15_runtime_cargo_attempt_status_index_anchors_are_locked",
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
        &cargo_attempt_status_anchors,
    );
    assert_contains_all(
        "Runtime 15 status row data",
        runtime_15_status_row_data,
        &cargo_attempt_evidence_anchors,
    );
}
