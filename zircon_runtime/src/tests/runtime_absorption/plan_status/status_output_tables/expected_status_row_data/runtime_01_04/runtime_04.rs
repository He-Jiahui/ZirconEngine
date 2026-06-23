use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 04 Asset pipeline 镜像文档守卫",
        &[
            "runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts",
            "asset_pipeline_boundary",
            "standalone rustc 1/1",
            "broader asset::/worker_pool Cargo gates pending",
        ],
    ),
    (
        "Runtime 04 Asset pipeline 行为测试锚审计同步",
        &[
            "behavior_test_anchor_count = 20",
            "missing_behavior_test_anchors = []",
            "standalone asset_pipeline 1/1",
            "broader asset::/worker_pool Cargo gates pending",
        ],
    ),
    (
        "Runtime 04 worker-pool manager frame sampler entry",
        &[
            "spawn_worker_pool_with_frame_sampler",
            "project_asset_manager_spawns_worker_pool_with_frame_sampler",
            "behavior_test_anchor_count = 20",
            "broader `asset::` / `worker_pool` Cargo filters",
        ],
    ),
    (
        "Runtime 04 asset worker request entry hard-cutover",
        &[
            "AssetWorkerPool::request_sender",
            "AssetWorkerPool::request(...)",
            "retired_worker_request_sender_references = []",
            "standalone `asset_pipeline` 1/1 与 `asset_worker_policy` 1/1",
        ],
    ),
    (
        "Runtime 04 Asset pipeline current audit recheck",
        &[
            "asset_pipeline_current_audit_static_passed_cargo_pending",
            "source files 22/22",
            "standalone `asset_pipeline.rs` 1/1",
            "broader `asset::` / `worker_pool` Cargo filters",
        ],
    ),
    (
        "Runtime 04 Asset pipeline inventory split",
        &[
            "asset_pipeline_inventory_split_static_passed_cargo_deferred_tests_deferred",
            "asset_pipeline_source_inventory.py",
            "asset_pipeline_anchor_inventory.py",
            "standalone `plan_status.rs` 33/33",
        ],
    ),
    (
        "Runtime 04 Asset pipeline Markdown renderer split",
        &[
            "asset_pipeline_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "asset_pipeline_markdown.py",
            "asset_pipeline_boundary.py` owns audit read, missing-anchor calculation, and risk aggregation at 328 lines",
            "standalone `plan_status.rs` 33/33",
        ],
    ),
    (
        "Runtime 04 F7 asset artifact/importer typed errors",
        &[
            "asset_artifact_importer_typed_errors_coremin_passed",
            "AssetImportError::Registry",
            "asset_import_error_preserves_registry_error_source",
            "review_f7_asset_artifact_errors_use_asset_import_error_sources",
        ],
    ),
    (
        "Runtime 04 F8 texture import settings apply API",
        &[
            "texture_import_settings_apply_api_coremin_check_passed",
            "review_f8_texture_import_settings_use_fallible_apply_not_with",
            "apply_import_settings",
            "old fallible with-entry absent",
        ],
    ),
];
