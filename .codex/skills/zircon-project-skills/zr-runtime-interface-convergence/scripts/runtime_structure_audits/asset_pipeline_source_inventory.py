from __future__ import annotations


EXPECTED_SOURCE_FILE_COUNT = 26
EXPECTED_GUARD_FILE_COUNT = 22
EXPECTED_WORKER_DIAGNOSTIC_COUNT = 7
EXPECTED_ARTIFACT_STORE_ROUNDTRIP_COUNT = 4
EXPECTED_WATCHER_TEST_COUNT = 7

RUNTIME_04_SOURCE_FILES = (
    "zircon_runtime/src/asset/facade/handle.rs",
    "zircon_runtime/src/asset/facade/assets.rs",
    "zircon_runtime/src/asset/facade/load_state.rs",
    "zircon_runtime/src/asset/facade/manager.rs",
    "zircon_runtime/src/asset/facade/event.rs",
    "zircon_runtime/src/asset/pipeline/worker_pool.rs",
    "zircon_runtime/src/asset/pipeline/worker_pool/options.rs",
    "zircon_runtime/src/asset/pipeline/worker_pool/completion.rs",
    "zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs",
    "zircon_runtime/src/asset/pipeline/manager/project_asset_manager/project_asset_manager.rs",
    "zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs",
    "zircon_runtime/src/asset/pipeline/manager/resource_sync/register_project_resource.rs",
    "zircon_runtime/src/asset/watch/asset_watcher.rs",
    "zircon_runtime/src/asset/watch/watch_loop.rs",
    "zircon_runtime/src/asset/watch/asset_watch_error.rs",
    "zircon_runtime/src/asset/artifact/cache_payload.rs",
    "zircon_runtime/src/asset/artifact/cache_payload/json_value.rs",
    "zircon_runtime/src/asset/artifact/cache_payload/mesh.rs",
    "zircon_runtime/src/asset/artifact/cache_payload/scene.rs",
    "zircon_runtime/src/asset/artifact/cache_payload/toml_value.rs",
    "zircon_runtime/src/asset/artifact/chunk_residency.rs",
    "zircon_runtime/src/asset/artifact/store.rs",
    "zircon_runtime/src/asset/module.rs",
    "zircon_runtime/crates/zr_resource/src/manager/registry_ops.rs",
    "zircon_runtime/crates/zr_resource/src/manager/commit.rs",
    "zircon_runtime_interface/src/resource/resource_record.rs",
)

RUNTIME_04_GUARD_FILES = (
    "zircon_runtime/src/asset/tests/facade/handle_lifecycle.rs",
    "zircon_runtime/src/asset/tests/facade/failure_reason.rs",
    "zircon_runtime/src/asset/tests/facade/hot_reload.rs",
    "zircon_runtime/src/asset/tests/pipeline/worker_pool.rs",
    "zircon_runtime/src/asset/tests/pipeline/worker_pool/diagnostics.rs",
    "zircon_runtime/src/asset/tests/pipeline/worker_pool/single_flight.rs",
    "zircon_runtime/src/asset/tests/pipeline/worker_pool/task_pool.rs",
    "zircon_runtime/src/asset/tests/watcher.rs",
    "zircon_runtime/src/asset/tests/assets/artifact_store.rs",
    "zircon_runtime/src/asset/tests/assets/artifact_store/bounded_read.rs",
    "zircon_runtime/src/asset/tests/assets/artifact_store/lazy_residency.rs",
    "zircon_runtime/src/asset/tests/assets/artifact_store/scene_components.rs",
    "zircon_runtime/src/asset/tests/assets/artifact_store/scene_script.rs",
    "zircon_runtime/crates/zr_resource/src/tests.rs",
    "zircon_runtime/src/tests/runtime_absorption/asset_surface.rs",
    "zircon_runtime/src/tests/runtime_absorption/asset_surface/facade_query.rs",
    "zircon_runtime/src/tests/runtime_absorption/asset_worker_policy.rs",
    "zircon_runtime/src/tests/runtime_absorption/asset_worker_policy/worker_pool.rs",
    "zircon_runtime/src/tests/runtime_absorption/asset_pipeline.rs",
    "zircon_runtime/src/tests/runtime_absorption/asset_pipeline/cargo_gate.rs",
    "zircon_runtime/src/tests/runtime_absorption/asset_pipeline/mirror_docs.rs",
    "zircon_runtime/src/tests/runtime_absorption/asset_pipeline/split_layout.rs",
)

RUNTIME_04_ARTIFACT_STORE_ROUNDTRIP_FILES = (
    "zircon_runtime/src/asset/tests/assets/artifact_store/scene_components.rs",
    "zircon_runtime/src/asset/tests/assets/artifact_store/scene_script.rs",
)
