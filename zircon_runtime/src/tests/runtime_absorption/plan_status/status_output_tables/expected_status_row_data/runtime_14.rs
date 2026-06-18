use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 14 Module family 镜像文档守卫",
        [
            "runtime_14_module_family_mirror_docs_match_structure_audit_counts",
            "module_family_boundary",
            "standalone rustc 13/13",
            "module-family Cargo/rustc gates pending",
        ],
    ),
    (
        "Runtime 14 animation family 28-file audit sync",
        [
            "animation = 28",
            "navigation = 9",
            "module_family_boundary",
            "module_family_source_count_static_passed_cargo_pending",
        ],
    ),
    (
        "Runtime 14 navigation fallback runtime owner split",
        [
            "navigation_runtime_owner_split_static_passed_cargo_pending",
            "folder-backed runtime owner split",
            "navigation = 9",
            "runtime/avoidance.rs",
        ],
    ),
    (
        "Runtime 14 Module family guard anchors 审计同步",
        [
            "module_family_guard_anchor_count = 7",
            "missing_module_family_guard_anchors = []",
            "standalone root_entries 13/13",
            "module-family Cargo/rustc gates pending",
        ],
    ),
    (
        "Runtime 14 animation runtime-status JSON 边界守卫",
        [
            "runtime_animation_status_json_boundary_sanitizes_non_finite_values",
            "animation_status_json_guard_present = true",
            "animation_status_json_anchor_count = 8",
            "missing_animation_status_json_anchors = []",
        ],
    ),
    (
        "Runtime 14 Module family Cargo gate 审计元数据",
        [
            "module_family_boundary",
            "cargo_gate_anchor_count = 5",
            "missing_cargo_gate_anchors = []",
            "cargo test -p zircon_runtime --lib engine_module --locked",
        ],
    ),
    (
        "Runtime 14 Cargo 验证窗口探测",
        [
            "cargo test -p zircon_runtime --lib --no-default-features --features core-min",
            "tree_view_pointer",
            "runtime_14_module_family_cargo_gate_stays_visible_until_filters_pass",
            "animation` / `navigation` / `diagnostic_log` / `engine_module` / full lib Cargo gates pending",
        ],
    ),
    (
        "Runtime 14 animation Cargo gate 尝试",
        [
            "cargo test -p zircon_runtime --lib animation --locked",
            "共享 lib-test 编译层",
            "SKINNED_MESH_MAX_JOINT_MATRICES",
            "ViewportCameraSnapshot.temporal_jitter",
        ],
    ),
    (
        "Runtime 14 animation Cargo gate 修复与复验阻塞",
        [
            "31 passed; 3 failed",
            "AnimationPlayerRuntimeStatus::sanitized_snapshot",
            "UiInputDispatchDiagnostics.capture_started",
            "default_interactions/table.rs:257",
        ],
    ),
    (
        "Runtime 14 animation runtime-status focused recheck timeout",
        [
            "runtime_status_reports_player_rig_and_gpu_readiness",
            "904s",
            "无测试结果",
            "animation` / `navigation` / `diagnostic_log` / `engine_module` / full lib Cargo gates",
        ],
    ),
];
