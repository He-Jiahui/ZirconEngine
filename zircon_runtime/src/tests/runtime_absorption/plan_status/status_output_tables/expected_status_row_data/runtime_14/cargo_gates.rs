use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 14 Module family Cargo gate 审计元数据",
        &[
            "module_family_boundary",
            "cargo_gate_anchor_count = 5",
            "missing_cargo_gate_anchors = []",
            "cargo test -p zircon_runtime --lib engine_module --locked",
        ],
    ),
    (
        "Runtime 14 Cargo 验证窗口探测",
        &[
            "cargo test -p zircon_runtime --lib --no-default-features --features core-min",
            "tree_view_pointer",
            "runtime_14_module_family_cargo_gate_stays_visible_until_filters_pass",
            "animation` / `navigation` / `diagnostic_log` / `engine_module` / full lib Cargo gates pending",
        ],
    ),
    (
        "Runtime 14 animation Cargo gate 尝试",
        &[
            "cargo_blocked_external_compile_drift",
            "cargo test -p zircon_runtime --lib animation --locked",
            "共享 lib-test 编译层",
            "SKINNED_MESH_MAX_JOINT_MATRICES",
            "ViewportCameraSnapshot.temporal_jitter",
        ],
    ),
    (
        "Runtime 14 animation Cargo gate 修复与复验阻塞",
        &[
            "cargo_recheck_blocked_external_ui_compile_drift",
            "31 passed; 3 failed",
            "AnimationPlayerRuntimeStatus::sanitized_snapshot",
            "UiInputDispatchDiagnostics.capture_started",
            "default_interactions/table.rs:257",
        ],
    ),
    (
        "Runtime 14 animation runtime-status focused recheck timeout",
        &[
            "cargo_recheck_timeout_no_result",
            "runtime_status_reports_player_rig_and_gpu_readiness",
            "904s",
            "无测试结果",
            "cargo/rustc processes were stopped",
            "animation` / `navigation` / `diagnostic_log` / `engine_module` / full lib Cargo gates",
        ],
    ),
];
