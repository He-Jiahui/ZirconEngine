use std::path::Path;

use super::inventory::{
    ASSET_PIPELINE_MIRROR_DOC_ANCHORS, EXPECTED_RUNTIME_04_BEHAVIOR_TEST_ANCHORS,
    EXPECTED_RUNTIME_04_GUARD_ANCHORS, EXPECTED_RUNTIME_04_GUARD_FILES,
    EXPECTED_RUNTIME_04_SOURCE_FILES, RUNTIME_11_SHARED_COMPLETION_TEST_ANCHORS,
    RUNTIME_11_WORKER_TIMER_TEST_ANCHORS,
};
use super::support::{assert_contains_all, assert_files_exist};

#[test]
fn runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts() {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = runtime_root
        .parent()
        .expect("zircon_runtime should live under the workspace root");

    assert_files_exist(
        runtime_root,
        EXPECTED_RUNTIME_04_SOURCE_FILES,
        "Runtime 04 audited source",
        "asset_pipeline_boundary before changing asset ownership",
    );
    assert_files_exist(
        runtime_root,
        EXPECTED_RUNTIME_04_GUARD_FILES,
        "Runtime 04 guard/test file",
        "asset_pipeline_boundary before changing guard ownership",
    );

    let guard_sources = [
        include_str!("../../../asset/tests/facade/handle_lifecycle.rs"),
        include_str!("../../../asset/tests/facade/failure_reason.rs"),
        include_str!("../../../asset/tests/facade/hot_reload.rs"),
        include_str!("../../../asset/tests/pipeline/worker_pool.rs"),
        include_str!("../../../asset/tests/pipeline/worker_pool/diagnostics.rs"),
        include_str!("../../../asset/tests/pipeline/worker_pool/single_flight.rs"),
        include_str!("../../../asset/tests/pipeline/worker_pool/task_pool.rs"),
        include_str!("../../../asset/tests/watcher.rs"),
        include_str!("../../../asset/tests/assets/artifact_store.rs"),
        include_str!("../../../asset/tests/assets/artifact_store/bounded_read.rs"),
        include_str!("../../../asset/tests/assets/artifact_store/lazy_residency.rs"),
        include_str!("../../../asset/tests/assets/artifact_store/scene_components.rs"),
        include_str!("../../../asset/tests/assets/artifact_store/scene_script.rs"),
        include_str!("../../../core/resource/tests.rs"),
        include_str!("../asset_surface.rs"),
        include_str!("../asset_surface/facade_query.rs"),
        include_str!("../asset_worker_policy.rs"),
        include_str!("../asset_worker_policy/worker_pool.rs"),
        include_str!("mirror_docs.rs"),
        include_str!("../../../asset/facade/load_state.rs"),
    ]
    .join("\n");
    let behavior_sources = [
        include_str!("../../../asset/tests/facade/handle_lifecycle.rs"),
        include_str!("../../../asset/tests/facade/failure_reason.rs"),
        include_str!("../../../asset/tests/facade/hot_reload.rs"),
        include_str!("../../../asset/tests/pipeline/worker_pool.rs"),
        include_str!("../../../asset/tests/pipeline/worker_pool/diagnostics.rs"),
        include_str!("../../../asset/tests/pipeline/worker_pool/single_flight.rs"),
        include_str!("../../../asset/tests/pipeline/worker_pool/task_pool.rs"),
        include_str!("../../../asset/tests/watcher.rs"),
        include_str!("../../../asset/tests/assets/artifact_store.rs"),
        include_str!("../../../asset/tests/assets/artifact_store/bounded_read.rs"),
        include_str!("../../../asset/tests/assets/artifact_store/lazy_residency.rs"),
        include_str!("../../../asset/tests/assets/artifact_store/scene_components.rs"),
        include_str!("../../../asset/tests/assets/artifact_store/scene_script.rs"),
        include_str!("../../../core/resource/tests.rs"),
        include_str!("../../../asset/facade/load_state.rs"),
    ]
    .join("\n");

    assert_contains_all(
        "Runtime 04 guard sources",
        &guard_sources,
        EXPECTED_RUNTIME_04_GUARD_ANCHORS,
    );
    assert_eq!(
        EXPECTED_RUNTIME_04_BEHAVIOR_TEST_ANCHORS.len(),
        24,
        "Runtime 04 behavior-test anchor count should mirror asset_pipeline_boundary"
    );
    assert_contains_all(
        "Runtime 04 behavior sources",
        &behavior_sources,
        EXPECTED_RUNTIME_04_BEHAVIOR_TEST_ANCHORS,
    );
    assert_contains_all(
        "Runtime 11 worker timer regression",
        include_str!("../../../asset/tests/pipeline/worker_pool.rs"),
        RUNTIME_11_WORKER_TIMER_TEST_ANCHORS,
    );
    assert_contains_all(
        "Runtime 11 shared completion payload matrix",
        include_str!("../../../asset/pipeline/worker_pool/tests.rs"),
        RUNTIME_11_SHARED_COMPLETION_TEST_ANCHORS,
    );
    let worker_pool_internal_tests = include_str!("../../../asset/pipeline/worker_pool/tests.rs");
    assert!(
        worker_pool_internal_tests.contains(
            "#[ignore = \"the Runtime11 256 MiB RSS matrix is an explicit pressure validation\"]\nfn payload_256_mib_matrix_rejects_oversize_retention"
        ),
        "Runtime 11 must keep the 256 MiB pressure matrix out of ordinary focused tests"
    );
    assert_contains_all(
        "Runtime 11 256 MiB pressure-matrix documentation",
        include_str!("../../../../../docs/zircon_runtime/asset/worker_pool.md"),
        &[
            "payload_256_mib_matrix_rejects_oversize_retention",
            "`--ignored`",
            "not counted as ordinary focused-test evidence",
        ],
    );

    let mirror_docs = [
        (
            "Runtime 04 plan",
            include_str!(
                "../../../../../docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md"
            ),
        ),
        (
            "runtime index",
            include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "asset facade doc",
            include_str!("../../../../../docs/zircon_runtime/asset/facade.md"),
        ),
        (
            "asset worker pool doc",
            include_str!("../../../../../docs/zircon_runtime/asset/worker_pool.md"),
        ),
        (
            "asset watcher doc",
            include_str!("../../../../../docs/zircon_runtime/asset/watcher.md"),
        ),
        (
            "asset artifact doc",
            include_str!("../../../../../docs/zircon_runtime/asset/artifact.md"),
        ),
        (
            "core resource doc",
            include_str!("../../../../../docs/zircon_runtime/core/resource.md"),
        ),
        (
            "M0 review",
            include_str!(
                "../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
            ),
        ),
        (
            "runtime-interface convergence",
            include_str!(
                "../../../../../docs/engine-architecture/runtime-interface-convergence.md"
            ),
        ),
    ];

    for (doc_name, doc_source) in mirror_docs {
        assert_contains_all(doc_name, doc_source, ASSET_PIPELINE_MIRROR_DOC_ANCHORS);
    }

    assert!(
        !workspace_root.join("zircon_asset/src/lib.rs").exists(),
        "Runtime 04 mirror guard assumes the standalone zircon_asset crate stays absorbed"
    );
    assert!(
        !include_str!("../../../asset/pipeline/worker_pool.rs").contains("pub fn request_sender"),
        "Runtime 04 hard-cutover requires AssetWorkerPool::request(...) to remain the only public request entry"
    );
}
