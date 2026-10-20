use crate::tests::runtime_absorption::structure_convention::fixed_history_archive_source as history;

pub(super) fn assert_worker_pool_doc_evidence() {
    let worker_pool_doc =
        include_str!("../../../../../../docs/crates/zircon_runtime/asset/worker_pool.md");
    let runtime_04_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md"
    );
    let runtime_11_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/11-job-system-task-model.md"
    );
    let runtime_04_output = history("docs/plans/zircon_runtime/runtime/04/2026-07-09-asset-pipeline-alignment-output-records.md");
    let runtime_11_output = history(
        "docs/plans/zircon_runtime/runtime/11/2026-07-09-job-system-task-model-output-records.md",
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");

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
        "project_asset_manager_uses_the_injected_runtime_io_pool",
        "project_asset_manager_defaults_share_the_process_io_pool",
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
