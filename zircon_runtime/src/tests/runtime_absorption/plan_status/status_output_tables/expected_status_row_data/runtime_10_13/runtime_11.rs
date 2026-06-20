use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 11 JobSystem 镜像文档守卫",
        [
            "runtime_11_job_system_mirror_docs_match_structure_audit_counts",
            "job_system_boundary",
            "standalone rustc 1/1",
            "tasks/ecs_schedule/worker_pool/rayon Cargo gates pending",
        ],
    ),
    (
        "Runtime 11 JobSystem 行为测试锚审计同步",
        [
            "behavior_test_anchor_count = 12",
            "missing_behavior_test_anchors = []",
            "scheduler_wait_all_waits_for_all_handles_and_records_sync_time",
            "tasks/ecs_schedule/worker_pool/rayon Cargo gates pending",
        ],
    ),
    (
        "Runtime 11 scheduler wait_all 同步点",
        [
            "JobScheduler::wait_all",
            "scheduler_wait_all_waits_for_all_handles_and_records_sync_time",
            "behavior_test_anchor_count = 12",
            "tasks/ecs_schedule/worker_pool/rayon Cargo gates pending",
        ],
    ),
    (
        "Runtime 11 panic-safe handle completion",
        [
            "panic_safe_completion_static_passed_cargo_deferred",
            "job_handle_wait_reports_task_panic_without_leaking_completion",
            "schedule_after_propagates_dependency_panic_without_running_dependent_task",
            "tasks/ecs_schedule/worker_pool/rayon Cargo gates",
        ],
    ),
    (
        "Runtime 11 graphics frustum rayon cutover",
        [
            "runtime_11_m2_1_graphics_frustum_rayon_cutover_static_passed_cargo_pending",
            "parallel_frustum.rs",
            "direct_rayon_paths = 2",
            "tasks/ecs_schedule/worker_pool/rayon Cargo gates pending",
        ],
    ),
    (
        "Runtime 11 JobSystem 2026-06-20 验证窗口探测",
        [
            "cargo test -p zircon_runtime --lib tasks --locked",
            "1200s",
            "standalone `asset_worker_policy.rs` 1/1",
            "tasks/ecs_schedule/worker_pool/rayon Cargo gates pending",
        ],
    ),
    (
        "Runtime 11 JobSystem core-min 验证窗口探测",
        [
            "core_min_cargo_recheck_timeout_static_guards_passed",
            "--no-default-features --features core-min",
            "无 `zircon_runtime*.exe` test binary",
            "tasks/ecs_schedule/worker_pool/rayon Cargo gates pending",
        ],
    ),
    (
        "Runtime 11 JobSystem current audit recheck",
        [
            "job_system_current_audit_static_passed_cargo_pending",
            "task owner modules 9/9",
            "standalone `rayon_boundary.rs` 3/3",
            "tasks/ecs_schedule/worker_pool/rayon Cargo gates",
        ],
    ),
];
