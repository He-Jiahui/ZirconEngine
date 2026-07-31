#[test]
fn asset_worker_pool_matches_runtime_04_and_11_decisions() {
    let worker_pool_source = include_str!("../../../asset/pipeline/worker_pool.rs");
    let worker_pool_diagnostics =
        include_str!("../../../asset/pipeline/worker_pool/diagnostics.rs");
    let worker_pool_completion = include_str!("../../../asset/pipeline/worker_pool/completion.rs");
    let worker_pool_options = include_str!("../../../asset/pipeline/worker_pool/options.rs");
    let worker_pool_sources = format!(
        "{worker_pool_source}\n{worker_pool_diagnostics}\n{worker_pool_completion}\n{worker_pool_options}"
    );
    let worker_pool_tests = include_str!("../../../asset/tests/pipeline/worker_pool.rs");
    let worker_pool_internal_tests = include_str!("../../../asset/pipeline/worker_pool/tests.rs");
    let project_asset_manager_construction =
        include_str!("../../../asset/pipeline/manager/project_asset_manager/construction.rs");
    let worker_pool_doc = include_str!("../../../../../docs/zircon_runtime/asset/worker_pool.md");
    let runtime_04_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md"
    );
    let runtime_11_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/11-job-system-task-model.md"
    );
    let runtime_04_output = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/04/2026-07-09-asset-pipeline-alignment-output-records.md"
    );
    let runtime_11_output = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/11/2026-07-09-job-system-task-model-output-records.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");

    for required_source_anchor in [
        "pub fn new(task_pool: TaskPool, options: AssetWorkerPoolOptions) -> Self",
        "pub struct AssetWorkerPoolOptions",
        "pub queue_depth: Option<usize>",
        "task_pool.spawn(move ||",
        "scheduled_jobs: usize",
        "pub struct AssetWorkerCompletionTicket",
        "Arc<CpuAssetPayload>",
        "pub completion_entry_capacity: usize",
        "pub completion_byte_capacity: usize",
        "pub request_max_age: Duration",
        "pub completion_max_age: Duration",
        "mod diagnostics;",
        "pub payload_clone_bytes: u64",
        "pub queue_age_total: Duration",
        "pub cancel_wall_total: Duration",
        "pub drop_wall_total: Duration",
        "TaskTimerSubscription",
        "Self::ProcessDefault => TaskTimer::process_default()?",
        "ASSET_WORKER_BUDGETED_THREADS_DIAGNOSTIC",
        "\"asset.worker.budgeted_threads\"",
        "AssetWorkerPoolFrameSampler",
        "ASSET_WORKER_FRAME_COMPLETED_DIAGNOSTIC",
        "\"asset.worker.frame_completed\"",
        "AssetWorkerThreadBudgetSource::TaskPoolIo",
    ] {
        assert!(
            worker_pool_sources.contains(required_source_anchor),
            "asset worker pool source should keep Runtime 04/11 anchor `{required_source_anchor}`"
        );
    }

    for required_manager_anchor in [
        "pub fn spawn_worker_pool_with_frame_sampler(",
        "AssetWorkerPoolFrameSampler::from_pool(&pool)",
        "TaskPools::default().io().clone()",
    ] {
        assert!(
            project_asset_manager_construction.contains(required_manager_anchor),
            "ProjectAssetManager worker construction should keep sampler anchor `{required_manager_anchor}`"
        );
    }

    let worker_pool_impl = worker_pool_source
        .split("impl AssetWorkerPool {")
        .nth(1)
        .expect("AssetWorkerPool implementation block should stay present");
    assert!(
        worker_pool_sources.contains("impl AssetWorkerPoolOptions"),
        "AssetWorkerPoolOptions should remain the queue configuration owner"
    );
    for retired_anchor in [
        "spawn_named_thread",
        "zircon-asset-",
        "crossbeam_channel::unbounded",
        "new_without_workers_for_test",
        "AssetWorkerPoolOptions::from_task_pool_options",
        "AssetWorkerThreadBudgetSource::Explicit",
        "request_sender",
        "completion_tx",
        "completion_rx",
        "pending_jobs: Mutex<usize>",
        "pending_jobs_changed: Condvar",
        "in_flight: Arc<Mutex<HashMap<AssetRequest, usize>>>",
        "for _ in 0..waiter_count",
    ] {
        assert!(
            !worker_pool_impl.contains(retired_anchor)
                && !worker_pool_sources.contains(retired_anchor),
            "asset worker pool should not retain retired anchor `{retired_anchor}`"
        );
    }

    for required_test_anchor in [
        "worker_pool_default_budgets_are_hard_limits",
        "project_asset_manager_uses_the_injected_runtime_io_pool",
        "project_asset_manager_defaults_share_the_process_io_pool",
        "worker_pool_bounded_queue_rejects_overflow_with_explicit_error",
        "cancelled_queued_work_keeps_admission_charged_until_its_closure_exits",
        "concurrent_requests_for_same_asset_share_one_immutable_payload_owner",
        "duplicate_waiter_budget_remains_hard_at_one_one_thousand_and_one_hundred_thousand",
        "worker_pool_source_uses_shared_ticket_results_not_a_completion_channel",
        "worker_pool_diagnostics_track_in_flight_and_failure_counts",
        "worker_pool_diagnostics_record_queue_age_clone_and_cancel_wall",
        "worker_pool_frame_sampler_records_per_job_completion_deltas",
        "completion_entry_budget_rejects_unharvested_payload_without_blocking_worker",
        "completion_age_expiry_is_observable_and_removes_unharvested_payload",
        "completion_deadline_replaces_the_pending_request_deadline",
        "completion_deadline_transition_reuses_a_full_timer_slot",
        "dropping_worker_pool_cancels_pending_jobs_without_synchronous_wait",
        "dropping_worker_pool_preserves_cancelled_ticket_after_armed_deadline",
        "dropping_worker_pool_on_its_io_worker_cancels_its_queued_ticket",
    ] {
        assert!(
            worker_pool_tests.contains(required_test_anchor),
            "asset worker pool tests should keep Runtime 04/11 evidence `{required_test_anchor}`"
        );
    }
    for required_internal_test_anchor in [
        "payload_size_matrix_keeps_one_owner_and_rejects_oversize_retention",
        "dropping_pool_records_a_nonblocking_drop_wall_measurement",
        "#[ignore = \"the Runtime11 256 MiB RSS matrix is an explicit pressure validation\"]\nfn payload_256_mib_matrix_rejects_oversize_retention",
        "runtime11_pressure_matrix_records_shared_completion_backpressure",
    ] {
        assert!(
            worker_pool_internal_tests.contains(required_internal_test_anchor),
            "asset worker pool internal tests should keep Runtime 11 evidence `{required_internal_test_anchor}`"
        );
    }

    for required_doc_anchor in [
        "Thread Budget",
        "Backpressure",
        "Request De-Duplication",
        "Diagnostics",
        "Runtime 11 M2.4",
        "ProjectAssetManager::default()",
        "spawn_worker_pool_with_frame_sampler",
        "asset.worker.budgeted_threads",
        "asset.worker.frame_completed",
        "asset.worker.queue_age_total_ms",
        "asset.worker.payload_clone_bytes",
        "asset.worker.cancel_wall_total_ms",
        "asset.worker.drop_wall_total_ms",
        "runtime11_pressure_matrix_records_shared_completion_backpressure",
        "only public request entry",
        "process-wide task owner",
        "does not create dedicated threads",
        "`--ignored`",
        "not counted as ordinary focused-test evidence",
    ] {
        assert!(
            worker_pool_doc.contains(required_doc_anchor),
            "asset worker pool doc should record `{required_doc_anchor}`"
        );
    }

    for required_plan_anchor in [
        "worker pool 原始缺口基线（已由 M2/M11-M2.4 收束）",
        "AssetWorkerPoolOptions",
        "Runtime 11 M2.4",
        "asset.worker.budgeted_threads",
        "asset.worker.frame_completed",
        "project_asset_manager_uses_the_injected_runtime_io_pool",
        "project_asset_manager_defaults_share_the_process_io_pool",
    ] {
        assert!(
            runtime_04_plan.contains(required_plan_anchor)
                || runtime_11_plan.contains(required_plan_anchor)
                || runtime_04_output.contains(required_plan_anchor)
                || runtime_11_output.contains(required_plan_anchor)
                || runtime_index.contains(required_plan_anchor),
            "runtime worker-pool plans should record `{required_plan_anchor}`"
        );
    }

    assert!(
        !runtime_04_plan.contains("worker pool 真实缺口确认"),
        "Runtime 04 should not present the original worker-pool gap as the current state"
    );
    assert!(
        !project_asset_manager_construction.contains("pub fn spawn_worker_pool(&self)"),
        "ProjectAssetManager must not retain the no-sampler worker pool construction entry"
    );
}
