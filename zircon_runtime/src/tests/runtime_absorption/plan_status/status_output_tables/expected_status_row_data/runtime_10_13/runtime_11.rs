use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 11 JobSystem 镜像文档守卫",
        &[
            "runtime_11_job_system_mirror_docs_match_structure_audit_counts",
            "job_system_boundary",
            "standalone rustc 1/1",
            "tasks/ecs_schedule/worker_pool/rayon Cargo gates pending",
        ],
    ),
    (
        "Runtime 11 JobSystem 行为测试锚审计同步",
        &[
            "behavior_test_anchor_count = 12",
            "missing_behavior_test_anchors = []",
            "scheduler_wait_all_waits_for_all_handles_and_records_sync_time",
            "tasks/ecs_schedule/worker_pool/rayon Cargo gates pending",
        ],
    ),
    (
        "Runtime 11 scheduler wait_all 同步点",
        &[
            "JobScheduler::wait_all",
            "scheduler_wait_all_waits_for_all_handles_and_records_sync_time",
            "behavior_test_anchor_count = 12",
            "tasks/ecs_schedule/worker_pool/rayon Cargo gates pending",
        ],
    ),
    (
        "Runtime 11 panic-safe handle completion",
        &[
            "panic_safe_completion_static_passed_cargo_deferred",
            "job_handle_wait_reports_task_panic_without_leaking_completion",
            "schedule_after_propagates_dependency_panic_without_running_dependent_task",
            "tasks/ecs_schedule/worker_pool/rayon Cargo gates",
        ],
    ),
    (
        "Runtime 11 graphics frustum rayon cutover",
        &[
            "runtime_11_m2_1_graphics_frustum_rayon_cutover_static_passed_cargo_pending",
            "parallel_frustum.rs",
            "direct_rayon_paths = 2",
            "tasks/ecs_schedule/worker_pool/rayon Cargo gates pending",
        ],
    ),
    (
        "Runtime 11 JobSystem 2026-06-20 验证窗口探测",
        &[
            "cargo test -p zircon_runtime --lib tasks --locked",
            "1200s",
            "standalone `asset_worker_policy.rs` 1/1",
            "tasks/ecs_schedule/worker_pool/rayon Cargo gates pending",
        ],
    ),
    (
        "Runtime 11 JobSystem core-min 验证窗口探测",
        &[
            "core_min_cargo_recheck_timeout_static_guards_passed",
            "--no-default-features --features core-min",
            "无 `zircon_runtime*.exe` test binary",
            "tasks/ecs_schedule/worker_pool/rayon Cargo gates pending",
        ],
    ),
    (
        "Runtime 11 JobSystem current audit recheck",
        &[
            "job_system_current_audit_static_passed_cargo_pending",
            "task owner modules 9/9",
            "standalone `rayon_boundary.rs` 3/3",
            "tasks/ecs_schedule/worker_pool/rayon Cargo gates",
        ],
    ),
    (
        "Runtime 11 JobSystem inventory split",
        &[
            "job_system_inventory_split_static_passed_cargo_deferred_tests_deferred",
            "job_system_source_inventory.py",
            "job_system_anchor_inventory.py",
            "standalone `plan_status.rs` 33/33",
        ],
    ),
    (
        "Runtime 11 JobSystem Markdown renderer split",
        &[
            "job_system_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "job_system_markdown.py",
            "job_system_boundary.py` now owns audit read, missing-anchor calculation, and risk aggregation at 193 lines",
            "standalone `plan_status.rs` 33/33",
        ],
    ),
    (
        "Runtime 11 worker wait-assist",
        &[
            "worker_wait_assist_static_passed_cargo_deferred",
            "worker_thread_wait_does_not_deadlock_scheduler",
            "assist_current_thread_once",
            "behavior_test_anchor_count = 13",
        ],
    ),
    (
        "Runtime 11 worker wait-assist core-min 验证窗口探测",
        &[
            "worker_wait_assist_core_min_cargo_timeout_no_result_residual_stopped",
            "worker_thread_wait_does_not_deadlock_scheduler",
            "1800s 工具窗口超时",
            "无 `zircon_runtime*.exe` 测试二进制",
        ],
    ),
    (
        "Runtime 11 worker wait-assist core-min test binary 验证",
        &[
            "worker_wait_assist_core_min_test_binary_passed_cargo_gate_pending",
            "zircon_runtime-coremin-workerwait.exe",
            "1 passed; 0 failed; 4687 filtered out",
            "tasks/ecs_schedule/worker_pool/rayon",
        ],
    ),
    (
        "Runtime 11 core-min test binary task/guard batch",
        &[
            "runtime_11_core_min_test_binary_task_guard_batch_passed_cargo_gate_pending",
            "tests::tasks::` 为 `18 passed; 0 failed; 4670 filtered out",
            "worker_pool` 为 `10 passed; 0 failed; 4678 filtered out",
            "rayon` 为 `4 passed; 0 failed; 4684 filtered out",
        ],
    ),
    (
        "Runtime 11 ecs_schedule source-guard lifetime anchor repair",
        &[
            "runtime_11_ecs_schedule_lifetime_guard_anchor_static_passed_rebuild_pending",
            "74 passed; 1 failed; 4613 filtered out",
            "native_steps: &'a [Self]",
            "重建 test binary 后复跑",
        ],
    ),
    (
        "Runtime 11 ecs_schedule core-min Cargo 复验",
        &[
            "runtime_11_core_min_ecs_schedule_cargo_passed_remaining_gates_pending",
            "75 passed; 0 failed; 4616 filtered out",
            "cargo test -p zircon_runtime --lib ecs_schedule --no-default-features --features core-min",
            "tasks`/`worker_pool`/`rayon",
        ],
    ),
    (
        "Runtime 11 tasks core-min Cargo 复验",
        &[
            "runtime_11_core_min_tasks_cargo_passed_remaining_gates_pending",
            "19 passed; 0 failed; 4673 filtered out",
            "cargo test -p zircon_runtime --lib tasks --no-default-features --features core-min",
            "worker_pool`/`rayon",
        ],
    ),
    (
        "Runtime 11 worker_pool core-min Cargo 复验",
        &[
            "runtime_11_core_min_worker_pool_cargo_passed_remaining_gates_pending",
            "10 passed; 0 failed; 4682 filtered out",
            "cargo test -p zircon_runtime --lib worker_pool --no-default-features --features core-min",
            "剩余 `rayon` Cargo gate",
        ],
    ),
    (
        "Runtime 11 rayon core-min Cargo 复验",
        &[
            "runtime_11_core_min_rayon_cargo_passed_broader_gates_pending",
            "4 passed; 0 failed; 4688 filtered out",
            "cargo test -p zircon_runtime --lib rayon --no-default-features --features core-min",
            "默认/更宽配置 Cargo gate",
        ],
    ),
    (
        "Runtime 11 tasks default Cargo 复验",
        &[
            "runtime_11_default_tasks_cargo_passed_remaining_default_gates_pending",
            "19 passed; 0 failed; 4673 filtered out",
            "cargo test -p zircon_runtime --lib tasks --locked",
            "worker_pool`/`rayon`/`ecs_schedule",
        ],
    ),
    (
        "Runtime 11 worker_pool default Cargo 复验",
        &[
            "runtime_11_default_worker_pool_cargo_passed_remaining_default_gates_pending",
            "10 passed; 0 failed; 4683 filtered out",
            "cargo test -p zircon_runtime --lib worker_pool --locked",
            "默认/更宽配置 `rayon`/`ecs_schedule",
        ],
    ),
    (
        "Runtime 11 rayon default Cargo 复验",
        &[
            "runtime_11_default_rayon_cargo_passed_full_lib_gate_pending",
            "4 passed; 0 failed; 4690 filtered out",
            "cargo test -p zircon_runtime --lib rayon --locked",
            "收尾全量 `cargo test -p zircon_runtime --lib --locked`",
        ],
    ),
    (
        "Runtime 11 ecs_schedule default Cargo 复验",
        &[
            "runtime_11_default_ecs_schedule_cargo_passed_full_lib_gate_pending",
            "75 passed; 0 failed; 4619 filtered out",
            "cargo test -p zircon_runtime --lib ecs_schedule --locked",
            "core-min 与默认配置 `tasks/ecs_schedule/worker_pool/rayon`",
        ],
    ),
    (
        "Runtime 11 full-lib default Cargo closeout attempt",
        &[
            "runtime_11_full_lib_cargo_timeout_with_broader_failures_observed",
            "1200s 工具窗口超时",
            "58 条 `... FAILED`",
            "未出现最终 `test result:`",
        ],
    ),
    (
        "Runtime 11 core runtime full-lib triage recheck",
        &[
            "runtime_11_core_runtime_tests_passed_full_lib_gate_broader_failures_pending",
            "core::runtime::tests::",
            "82 passed; 0 failed; 4613 filtered out",
            "asset/dynamic_api/graphics broader gates",
        ],
    ),
    (
        "Runtime 11 asset broader failure triage core-min 复验",
        &[
            "runtime_11_asset_tests_passed_full_lib_gate_dynamic_graphics_pending",
            "asset::tests::",
            "363 passed; 0 failed; 4334 filtered out",
            "dynamic_api/graphics",
        ],
    ),
    (
        "Runtime 11 full-lib default after asset triage recheck",
        &[
            "runtime_11_full_lib_after_asset_recheck_blocked_graphics_compile_timeout",
            "full_lib_default_after_asset_20260621.log",
            "RenderExposureReadbackReport",
            "graphics_execution_record_exposure_default_20260621.log",
        ],
    ),
    (
        "Runtime 11 full-lib default after graphics exposure retry",
        &[
            "runtime_11_full_lib_after_graphics_exposure_retry_timeout_104_broader_failures",
            "graphics_execution_record_exposure_default_retry_20260621.log",
            "104 条 `... FAILED`",
            "full_lib_default_after_graphics_exposure_retry_20260621.log",
        ],
    ),
];
