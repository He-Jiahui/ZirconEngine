type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 P0 F1/F2/F4 top-row closed status sync",
        &[
            "p0_f1_f2_f4_top_row_closed_status_static_passed_cargo_deferred",
            "docs/plans/engine-code-review-findings-2026-06.md",
            "tests/runtime_absorption/code_review_findings/p0_robustness.rs",
            "review_f1_native_host_callbacks_catch_unwind_before_crossing_ffi",
            "review_f2_scene_eventbus_locks_recover_after_poison",
            "review_f4_render_submit_capability_gaps_return_typed_errors",
            "| F1 | **原生插件入站 host 回调必须在跨 FFI 前截断 panic**",
            "| F2 | **scene/EventBus shared locks recover after poison**",
            "| F4 | **render submit viewport/provider capability gaps return typed errors**",
            "| Runtime 15 + Runtime 06 + Plugins 11 / review closed |",
            "| Runtime 15 + Runtime 07 / review closed |",
            "| Runtime 07 + render index / review closed |",
            "F1/F2/F4 已有闭合守卫",
            "完整 Runtime 07 FPS/profiling/full gate 仍 pending",
            "cargo deferred",
        ],
    ),
    (
        "Runtime 15 M3 D7 core workspace dependency top-row closed status sync",
        &[
            "d7_core_workspace_dependency_top_row_closed_status_static_passed_cargo_deferred",
            "docs/plans/engine-code-review-findings-2026-06.md",
            "tests/runtime_absorption/code_review_findings/p0_robustness.rs",
            "review_priority_recommendation_tracks_current_remaining_work",
            "| D7 | core workspace dependency inheritance 已完成首轮全局 guard",
            "| M2 / closed |",
            "d7_core_workspace_dependency_inheritance_guard_static_passed_cargo_deferred",
            "core_workspace_dependency_status = core-workspace-deps-clean",
            "core_workspace_dependency_violation_count = 0",
            "插件间 path 依赖仍归后续切片",
            "cargo deferred",
        ],
    ),
];
