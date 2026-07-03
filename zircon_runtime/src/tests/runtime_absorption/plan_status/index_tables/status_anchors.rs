use super::*;

#[test]
fn runtime_15_runtime_03_module_doc_status_index_anchors_are_locked() {
    let index_source = include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let structure_convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let module_convention =
        include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md");
    let session_note = include_str!(
        "../../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
    );
    let output_anchors = include_str!(
        "../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_output_anchors.py"
    );
    let status_row_data = include_str!(
        "../status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs"
    );
    let status_map = include_str!(
        "../status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs"
    );
    let date_map = include_str!(
        "../status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs"
    );

    let runtime_03_index_anchors = [
        "Runtime 03 Schedule/frame-loop module-doc 镜像元数据",
        "Runtime 05 status-output Runtime 03 module-doc row",
        "frame schedule module-doc anchors 3/3",
        "guard/test files 8/8",
        "Runtime 03 guard anchors 14/14",
        "ecs_schedule/time/session/schedule_parallel Cargo gates",
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
        ("Runtime 15 subplan", runtime_15_plan),
        ("runtime index", index_source),
        ("engine code structure convention", structure_convention),
        ("engine code review findings", review_findings),
        ("module convention doc", module_convention),
        ("runtime implementation session note", session_note),
        ("Runtime 15 status row data", status_row_data),
        ("Runtime 15 expected status map", status_map),
        ("Runtime 15 expected date map", date_map),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "Runtime 15 status row data",
        status_row_data,
        &runtime_03_index_anchors,
    );
}

#[test]
fn runtime_15_runtime_07_scene_asset_status_index_anchors_are_locked() {
    let index_source = include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let structure_convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let module_convention =
        include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md");
    let session_note = include_str!(
        "../../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
    );
    let output_anchors = include_str!(
        "../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_output_anchors.py"
    );
    let runtime_07_scene_status_row_data = include_str!(
        "../status_output_tables/expected_status_row_data/runtime_06_09/runtime_07/scene_asset.rs"
    );
    let runtime_15_status_row_data = include_str!(
        "../status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs"
    );
    let status_map = include_str!(
        "../status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs"
    );
    let date_map = include_str!(
        "../status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs"
    );

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
        "runtime index",
        index_source,
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
        ("Runtime 15 subplan", runtime_15_plan),
        ("runtime index", index_source),
        ("engine code structure convention", structure_convention),
        ("engine code review findings", review_findings),
        ("module convention doc", module_convention),
        ("runtime implementation session note", session_note),
        ("Runtime 15 status row data", runtime_15_status_row_data),
        ("Runtime 15 expected status map", status_map),
        ("Runtime 15 expected date map", date_map),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "Runtime 15 status row data",
        runtime_15_status_row_data,
        &runtime_07_scene_guard_anchors[4..],
    );
}

#[test]
fn runtime_15_runtime_07_owner_budget_status_index_anchors_are_locked() {
    let index_source = include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let structure_convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let module_convention =
        include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md");
    let session_note = include_str!(
        "../../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
    );
    let output_anchors = include_str!(
        "../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_output_anchors.py"
    );
    let runtime_07_owner_budget_status_row_data = include_str!(
        "../status_output_tables/expected_status_row_data/runtime_06_09/runtime_07/owner_budget.rs"
    );
    let runtime_15_status_row_data = include_str!(
        "../status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs"
    );
    let status_map = include_str!(
        "../status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs"
    );
    let date_map = include_str!(
        "../status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs"
    );

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
        "runtime index",
        index_source,
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
        ("Runtime 15 subplan", runtime_15_plan),
        ("runtime index", index_source),
        ("engine code structure convention", structure_convention),
        ("engine code review findings", review_findings),
        ("module convention doc", module_convention),
        ("runtime implementation session note", session_note),
        ("Runtime 15 status row data", runtime_15_status_row_data),
        ("Runtime 15 expected status map", status_map),
        ("Runtime 15 expected date map", date_map),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "Runtime 15 status row data",
        runtime_15_status_row_data,
        &runtime_07_owner_budget_index_anchors,
    );
}

#[test]
fn runtime_15_runtime_02_generated_status_index_anchors_are_locked() {
    let index_source = include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let structure_convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let module_convention =
        include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md");
    let session_note = include_str!(
        "../../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
    );
    let output_anchors = include_str!(
        "../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_output_anchors.py"
    );
    let runtime_02_status_row_data = include_str!(
        "../status_output_tables/expected_status_row_data/runtime_01_04/runtime_02.rs"
    );
    let runtime_15_status_row_data = include_str!(
        "../status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs"
    );
    let status_map = include_str!(
        "../status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs"
    );
    let date_map = include_str!(
        "../status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs"
    );

    let runtime_02_generated_index_anchors = [
        "Runtime 02 generated template count 审计同步",
        "`template_file_count=10`",
        "generated export templates 10/10",
        "0 migration debt",
        "stale 9/9 scan",
        "Runtime 02 generated/export/app/editor/plugin Cargo gates",
    ];
    assert_contains_all(
        "runtime plan-status output anchor inventory",
        output_anchors,
        &runtime_02_generated_index_anchors,
    );
    assert_contains_all(
        "runtime index",
        index_source,
        &runtime_02_generated_index_anchors,
    );

    let runtime_02_generated_row = runtime_02_status_row_data
        .split_once("\"Runtime 02 generated template count 审计同步\"")
        .expect("Runtime 02 row-data should keep the generated template count row")
        .1
        .split_once("),")
        .expect("Runtime 02 generated template count row should end as a tuple")
        .0;
    assert_contains_all(
        "Runtime 02 generated current row data",
        runtime_02_generated_row,
        &runtime_02_generated_index_anchors[1..],
    );

    let status_anchors = [
        "Runtime 15 M3 Runtime 02 generated status anchor sync",
        "runtime_15_runtime_02_generated_status_anchor_sync_static_passed_cargo_deferred",
        "runtime_15_runtime_02_generated_status_index_anchors_are_locked",
    ];
    for (label, source) in [
        ("Runtime 15 subplan", runtime_15_plan),
        ("runtime index", index_source),
        ("engine code structure convention", structure_convention),
        ("engine code review findings", review_findings),
        ("module convention doc", module_convention),
        ("runtime implementation session note", session_note),
        ("Runtime 15 status row data", runtime_15_status_row_data),
        ("Runtime 15 expected status map", status_map),
        ("Runtime 15 expected date map", date_map),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "Runtime 15 status row data",
        runtime_15_status_row_data,
        &runtime_02_generated_index_anchors,
    );
}

#[test]
fn runtime_15_runtime_10_behavior_status_index_anchors_are_locked() {
    let index_source = include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let structure_convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let module_convention =
        include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md");
    let session_note = include_str!(
        "../../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
    );
    let output_anchors = include_str!(
        "../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_output_anchors.py"
    );
    let runtime_10_status_row_data = include_str!(
        "../status_output_tables/expected_status_row_data/runtime_10_13/runtime_10/dynamic_api.rs"
    );
    let runtime_05_status_row_data = include_str!(
        "../status_output_tables/expected_status_row_data/runtime_05/cross_runtime_rows.rs"
    );
    let runtime_15_status_row_data = include_str!(
        "../status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs"
    );
    let status_map = include_str!(
        "../status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs"
    );
    let date_map = include_str!(
        "../status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs"
    );

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
        "runtime index",
        index_source,
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
        ("Runtime 15 subplan", runtime_15_plan),
        ("runtime index", index_source),
        ("engine code structure convention", structure_convention),
        ("engine code review findings", review_findings),
        ("module convention doc", module_convention),
        ("runtime implementation session note", session_note),
        ("Runtime 15 status row data", runtime_15_status_row_data),
        ("Runtime 15 expected status map", status_map),
        ("Runtime 15 expected date map", date_map),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "Runtime 15 status row data",
        runtime_15_status_row_data,
        &runtime_10_behavior_index_anchors,
    );
}

#[test]
fn runtime_15_runtime_cargo_attempt_status_index_anchors_are_locked() {
    let index_source = include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let structure_convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let module_convention =
        include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md");
    let session_note = include_str!(
        "../../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
    );
    let output_anchors = include_str!(
        "../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_output_anchors.py"
    );
    let runtime_05_audit_metadata = include_str!(
        "../status_output_tables/expected_status_row_data/runtime_05/audit_metadata/plan_coverage_rows.rs"
    );
    let runtime_14_cargo_row_data =
        include_str!("../status_output_tables/expected_status_row_data/runtime_14/cargo_gates.rs");
    let runtime_15_status_row_data = include_str!(
        "../status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs"
    );
    let status_map = include_str!(
        "../status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs"
    );
    let date_map = include_str!(
        "../status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs"
    );

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
    assert_contains_all("runtime index", index_source, &cargo_attempt_status_anchors);
    assert_contains_all(
        "runtime index",
        index_source,
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
        ("Runtime 15 subplan", runtime_15_plan),
        ("runtime index", index_source),
        ("engine code structure convention", structure_convention),
        ("engine code review findings", review_findings),
        ("module convention doc", module_convention),
        ("runtime implementation session note", session_note),
        ("Runtime 15 status row data", runtime_15_status_row_data),
        ("Runtime 15 expected status map", status_map),
        ("Runtime 15 expected date map", date_map),
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
