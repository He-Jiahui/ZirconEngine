#[test]
fn asset_worker_pool_matches_runtime_04_and_11_decisions() {
    let worker_pool_source = include_str!("../../../asset/pipeline/worker_pool.rs");
    let worker_pool_tests = include_str!("../../../asset/tests/pipeline/worker_pool.rs");
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
        "pending_jobs: Mutex<usize>",
        "pending_jobs_changed: Condvar",
        "in_flight: Arc<Mutex<HashMap<AssetRequest, usize>>>",
        "if let Some(waiter_count) = in_flight.get_mut(&request)",
        "for _ in 0..waiter_count",
        "ASSET_WORKER_BUDGETED_THREADS_DIAGNOSTIC",
        "\"asset.worker.budgeted_threads\"",
        "AssetWorkerPoolFrameSampler",
        "ASSET_WORKER_FRAME_COMPLETED_DIAGNOSTIC",
        "\"asset.worker.frame_completed\"",
        "AssetWorkerThreadBudgetSource::TaskPoolIo",
    ] {
        assert!(
            worker_pool_source.contains(required_source_anchor),
            "asset worker pool source should keep Runtime 04/11 anchor `{required_source_anchor}`"
        );
    }

    for required_manager_anchor in [
        "pub fn spawn_worker_pool_with_frame_sampler(",
        "AssetWorkerPoolFrameSampler::from_pool(&pool)",
        "self.spawn_worker_pool_with_frame_sampler()",
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
        worker_pool_source.contains("impl AssetWorkerPoolOptions"),
        "AssetWorkerPoolOptions should remain the queue configuration owner"
    );
    for retired_anchor in [
        "spawn_named_thread",
        "zircon-asset-",
        "new_without_workers_for_test",
        "AssetWorkerPoolOptions::from_task_pool_options",
        "AssetWorkerThreadBudgetSource::Explicit",
        "request_sender",
    ] {
        assert!(
            !worker_pool_impl.contains(retired_anchor)
                && !worker_pool_source.contains(retired_anchor),
            "asset worker pool should not retain retired anchor `{retired_anchor}`"
        );
    }

    for required_test_anchor in [
        "worker_pool_unbounded_mode_is_explicit_opt_in",
        "project_asset_manager_uses_the_injected_runtime_io_pool",
        "project_asset_manager_defaults_share_the_process_io_pool",
        "worker_pool_bounded_queue_rejects_overflow_with_explicit_error",
        "concurrent_requests_for_same_asset_decode_once_and_notify_all",
        "worker_pool_diagnostics_track_in_flight_and_failure_counts",
        "worker_pool_frame_sampler_records_per_frame_completion_deltas",
        "dropping_worker_pool_waits_for_its_runtime_io_jobs",
        "dropping_worker_pool_on_its_io_worker_does_not_deadlock_pending_jobs",
    ] {
        assert!(
            worker_pool_tests.contains(required_test_anchor),
            "asset worker pool tests should keep Runtime 04/11 evidence `{required_test_anchor}`"
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
        "only public request entry",
        "process-wide task owner",
        "does not create dedicated threads",
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
}
