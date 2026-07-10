from __future__ import annotations


STATUS_OUTPUT_TABLE_GUARD_ANCHORS = (
    "runtime_plan_status_output_tables_cover_index_and_all_subplans",
    "runtime_index_status_output_records_recent_cross_plan_slices",
    "all runtime index status rows",
    "full coverage guard",
)
RUNTIME_03_MODULE_DOC_STATUS_INDEX_ANCHORS = (
    "schedule_frame_loop_boundary",
    "runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts",
    "frame schedule module-doc anchors 3/3",
    "guard/test files 11/11",
    "Runtime 03 guard anchors 14/14",
    "ecs_schedule/tests::time::/session/schedule_parallel Cargo gates",
)
RUNTIME_03_MODULE_DOC_STATUS_GUARD_ANCHORS = (
    "Runtime 03 Schedule/frame-loop module-doc 镜像元数据",
    "Runtime 05 status-output Runtime 03 module-doc row",
    "frame schedule module-doc anchors 3/3",
    "guard/test files 11/11",
    "Runtime 03 guard anchors 14/14",
)
RUNTIME_03_MODULE_DOC_STATUS_DOC_ANCHORS = (
    "Runtime 03 module-doc status-output row",
    "frame schedule module-doc anchors 3/3",
    "guard/test files 11/11",
)
RUNTIME_07_SCENE_STATUS_INDEX_ANCHORS = (
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
)
RUNTIME_07_SCENE_STATUS_GUARD_ANCHORS = (
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
)
RUNTIME_07_SCENE_STATUS_DOC_ANCHORS = (
    "Runtime 07 scene asset owner-split",
    "split-drift",
    "public-surface",
    "performance_hotpath_boundary",
)
RUNTIME_07_OWNER_BUDGET_STATUS_INDEX_ANCHORS = (
    "Runtime 07 owner-budget 0-hotspot current audit sync",
    "`large_file_m1_gate_status = classified-and-clear`",
    "`large_file_hotspot_count = 0`",
    "`large_file_migration_debt_count = 0`",
    "`large_file_owner_class_count = 0`",
    "`large_file_unclassified_hotspot_count = 0`",
    "direct `performance_hotpath_boundary_audit` risks=0 / hotspots=0",
    "standalone `performance_hotspots.rs` exact owner-budget guards",
    "extract/ecs_query/performance profiling/FPS Cargo gates",
)
RUNTIME_07_OWNER_BUDGET_STATUS_GUARD_ANCHORS = (
    "Runtime 07 owner-budget 0-hotspot current audit sync",
    "`large_file_m1_gate_status = classified-and-clear`",
    "`large_file_hotspot_count = 0`",
    "`large_file_migration_debt_count = 0`",
    "`large_file_owner_class_count = 0`",
    "`large_file_unclassified_hotspot_count = 0`",
    "direct `performance_hotpath_boundary_audit` risks=0 / hotspots=0",
    "standalone `performance_hotspots.rs` exact owner-budget guards",
)
RUNTIME_07_OWNER_BUDGET_STATUS_DOC_ANCHORS = (
    "Runtime 07 owner-budget 0-hotspot current audit sync",
    "large_file_hotspot_count = 0",
    "large_file_m1_gate_status = classified-and-clear",
)
RUNTIME_02_GENERATED_STATUS_INDEX_ANCHORS = (
    "Runtime 02 generated template count 审计同步",
    "`template_file_count=10`",
    "generated export templates 10/10",
    "0 migration debt",
    "stale 9/9 scan",
    "Runtime 02 generated/export/app/editor/plugin Cargo gates",
)
RUNTIME_02_GENERATED_STATUS_GUARD_ANCHORS = (
    "Runtime 02 generated template count 审计同步",
    "`template_file_count=10`",
    "generated export templates 10/10",
    "0 migration debt",
    "stale 9/9 scan",
)
RUNTIME_02_GENERATED_STATUS_DOC_ANCHORS = (
    "Runtime 02 generated template count",
    "template_file_count=10",
    "generated template count audit-sync row",
)
RUNTIME_10_BEHAVIOR_STATUS_INDEX_ANCHORS = (
    "Runtime 10 Dynamic API 行为测试锚审计同步",
    "Runtime 05 status-output Runtime 10 behavior-test row",
    "behavior_test_anchor_count = 16",
    "missing_behavior_test_anchors = []",
    "standalone dynamic_api_session 9/9",
    "dynamic_api/app/UI gates pending",
)
RUNTIME_10_BEHAVIOR_STATUS_GUARD_ANCHORS = (
    "Runtime 10 Dynamic API 行为测试锚审计同步",
    "Runtime 05 status-output Runtime 10 behavior-test row",
    "behavior_test_anchor_count = 16",
    "standalone status-output 2/2",
)
RUNTIME_10_BEHAVIOR_STATUS_DOC_ANCHORS = (
    "Runtime 10 behavior status-output row",
    "behavior_test_anchor_count = 16",
    "runtime_10_behavior_status_guard_present = true",
)
CARGO_ATTEMPT_STATUS_ANCHORS = (
    "cargo_deferred_active_lane",
    "cargo_blocked_external_compile_drift",
    "cargo_recheck_blocked_external_ui_compile_drift",
    "cargo_recheck_timeout_no_result",
    "Runtime 14 Cargo 验证窗口探测",
    "Runtime 14 animation Cargo gate 尝试",
    "Runtime 14 animation Cargo gate 修复与复验阻塞",
    "Runtime 14 animation runtime-status focused recheck timeout",
)
CARGO_ATTEMPT_STATUS_EVIDENCE_ANCHORS = (
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
)
