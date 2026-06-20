use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 04 Asset pipeline 镜像文档守卫",
        [
            "runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts",
            "asset_pipeline_boundary",
            "standalone rustc 1/1",
            "broader asset::/worker_pool Cargo gates pending",
        ],
    ),
    (
        "Runtime 04 Asset pipeline 行为测试锚审计同步",
        [
            "behavior_test_anchor_count = 20",
            "missing_behavior_test_anchors = []",
            "standalone asset_pipeline 1/1",
            "broader asset::/worker_pool Cargo gates pending",
        ],
    ),
    (
        "Runtime 04 worker-pool manager frame sampler entry",
        [
            "spawn_worker_pool_with_frame_sampler",
            "project_asset_manager_spawns_worker_pool_with_frame_sampler",
            "behavior_test_anchor_count = 20",
            "broader `asset::` / `worker_pool` Cargo filters",
        ],
    ),
    (
        "Runtime 04 asset worker request entry hard-cutover",
        [
            "AssetWorkerPool::request_sender",
            "AssetWorkerPool::request(...)",
            "retired_worker_request_sender_references = []",
            "standalone `asset_pipeline` 1/1 与 `asset_worker_policy` 1/1",
        ],
    ),
    (
        "Runtime 04 Asset pipeline current audit recheck",
        [
            "asset_pipeline_current_audit_static_passed_cargo_pending",
            "source files 22/22",
            "standalone `asset_pipeline.rs` 1/1",
            "broader `asset::` / `worker_pool` Cargo filters",
        ],
    ),
];
