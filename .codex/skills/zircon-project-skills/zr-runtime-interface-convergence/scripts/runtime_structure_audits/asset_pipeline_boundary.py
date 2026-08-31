from __future__ import annotations

from pathlib import Path

from .asset_pipeline_anchor_inventory import (
    ARTIFACT_CACHE_ANCHORS,
    CARGO_GATE_ANCHORS,
    HANDLE_STATE_ANCHORS,
    MIRROR_DOCS_GUARD,
    RESOURCE_RELOAD_ANCHORS,
    RUNTIME_04_BEHAVIOR_TEST_ANCHORS,
    RUNTIME_04_DOC_ANCHORS,
    RUNTIME_04_TEST_ANCHORS,
    WATCHER_ANCHORS,
    WORKER_DIAGNOSTIC_ANCHORS,
    WORKER_POOL_ANCHORS,
)
from .asset_pipeline_source_inventory import (
    EXPECTED_ARTIFACT_STORE_ROUNDTRIP_COUNT,
    EXPECTED_GUARD_FILE_COUNT,
    EXPECTED_SOURCE_FILE_COUNT,
    EXPECTED_WATCHER_TEST_COUNT,
    EXPECTED_WORKER_DIAGNOSTIC_COUNT,
    RUNTIME_04_ARTIFACT_STORE_ROUNDTRIP_FILES,
    RUNTIME_04_GUARD_FILES,
    RUNTIME_04_SOURCE_FILES,
)


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _file_line_count(path: Path) -> int:
    return len(_read_text(path).splitlines())


def _file_entries(root: Path, files: tuple[str, ...]) -> tuple[list[dict[str, object]], list[str]]:
    entries: list[dict[str, object]] = []
    missing: list[str] = []
    for file_name in files:
        path = root / file_name
        if not path.exists():
            missing.append(file_name)
            continue
        entries.append({"path": file_name, "lines": _file_line_count(path)})
    return entries, missing


def _missing_snippets(sources: tuple[str, ...], snippets: tuple[str, ...]) -> list[str]:
    return [
        snippet
        for snippet in snippets
        if not any(snippet in source for source in sources)
    ]


def _line_references(root: Path, path: Path, needle: str) -> list[dict[str, object]]:
    if not path.exists():
        return []

    references: list[dict[str, object]] = []
    relative = path.relative_to(root).as_posix()
    for line_no, line in enumerate(_read_text(path).splitlines(), start=1):
        if needle in line:
            references.append(
                {"path": relative, "line": line_no, "snippet": line.strip()}
            )
    return references


def asset_pipeline_boundary_audit(root: Path) -> dict[str, object]:
    handle = root / "zircon_runtime/src/asset/facade/handle.rs"
    assets = root / "zircon_runtime/src/asset/facade/assets.rs"
    load_state = root / "zircon_runtime/src/asset/facade/load_state.rs"
    facade_manager = root / "zircon_runtime/src/asset/facade/manager.rs"
    resource_record = root / "zircon_runtime_interface/src/resource/resource_record.rs"
    registry_ops = root / "zircon_runtime/crates/zr_resource/src/manager/registry_ops.rs"
    resource_commit = root / "zircon_runtime/crates/zr_resource/src/manager/commit.rs"
    resource_sync = (
        root
        / "zircon_runtime/src/asset/pipeline/manager/resource_sync/register_project_resource.rs"
    )
    worker_pool = root / "zircon_runtime/src/asset/pipeline/worker_pool.rs"
    worker_pool_options = (
        root / "zircon_runtime/src/asset/pipeline/worker_pool/options.rs"
    )
    worker_pool_completion = (
        root / "zircon_runtime/src/asset/pipeline/worker_pool/completion.rs"
    )
    worker_pool_diagnostics = (
        root / "zircon_runtime/src/asset/pipeline/worker_pool/diagnostics.rs"
    )
    construction = (
        root
        / "zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs"
    )
    watcher = root / "zircon_runtime/src/asset/watch/asset_watcher.rs"
    watch_loop = root / "zircon_runtime/src/asset/watch/watch_loop.rs"
    watch_error = root / "zircon_runtime/src/asset/watch/asset_watch_error.rs"
    project_asset_manager = (
        root
        / "zircon_runtime/src/asset/pipeline/manager/project_asset_manager/project_asset_manager.rs"
    )
    project_asset_runtime = (
        root
        / "zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs"
    )
    artifact_mod = root / "zircon_runtime/src/asset/artifact/mod.rs"
    artifact_store = root / "zircon_runtime/src/asset/artifact/store.rs"
    artifact_chunk_residency = (
        root / "zircon_runtime/src/asset/artifact/chunk_residency.rs"
    )
    cache_payload = root / "zircon_runtime/src/asset/artifact/cache_payload.rs"
    cache_json = root / "zircon_runtime/src/asset/artifact/cache_payload/json_value.rs"
    cache_mesh = root / "zircon_runtime/src/asset/artifact/cache_payload/mesh.rs"
    cache_scene = root / "zircon_runtime/src/asset/artifact/cache_payload/scene.rs"
    cache_toml = root / "zircon_runtime/src/asset/artifact/cache_payload/toml_value.rs"
    runtime_04_plan = (
        root / "docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md"
    )
    runtime_index = root / "docs/plans/zircon_runtime/runtime/index.md"
    asset_docs = (
        root / "docs/zircon_runtime/asset/facade.md",
        root / "docs/zircon_runtime/asset/worker_pool.md",
        root / "docs/zircon_runtime/asset/watcher.md",
        root / "docs/zircon_runtime/asset/artifact.md",
        root / "docs/zircon_runtime/core/resource.md",
    )
    review = root / "docs/engine-architecture/runtime-architecture-review-m0.md"
    convergence = root / "docs/engine-architecture/runtime-interface-convergence.md"

    handle_state_sources = tuple(
        _read_text(path)
        for path in (handle, assets, load_state, facade_manager, resource_record)
        if path.exists()
    )
    resource_reload_sources = tuple(
        _read_text(path)
        for path in (registry_ops, resource_commit, resource_sync)
        if path.exists()
    )
    worker_sources = tuple(
        _read_text(path)
        for path in (
            worker_pool,
            worker_pool_options,
            worker_pool_completion,
            worker_pool_diagnostics,
            construction,
        )
        if path.exists()
    )
    watcher_sources = tuple(
        _read_text(path)
        for path in (
            watcher,
            watch_loop,
            watch_error,
            project_asset_manager,
            project_asset_runtime,
        )
        if path.exists()
    )
    artifact_sources = tuple(
        _read_text(path)
        for path in (
            artifact_mod,
            artifact_store,
            artifact_chunk_residency,
            cache_payload,
            cache_json,
            cache_mesh,
            cache_scene,
            cache_toml,
            root / "zircon_runtime/src/asset/artifact/cache_payload/scene/script.rs",
        )
        if path.exists()
    )
    guard_sources = tuple(
        _read_text(root / file_name)
        for file_name in RUNTIME_04_GUARD_FILES
        if (root / file_name).exists()
    ) + ((_read_text(load_state),) if load_state.exists() else tuple())
    artifact_store_roundtrip_sources = tuple(
        _read_text(root / file_name)
        for file_name in RUNTIME_04_ARTIFACT_STORE_ROUNDTRIP_FILES
        if (root / file_name).exists()
    )
    behavior_test_sources = tuple(
        _read_text(root / file_name)
        for file_name in (
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
            "zircon_runtime/crates/zr_resource/src/tests.rs",
        )
        if (root / file_name).exists()
    ) + artifact_store_roundtrip_sources + ((_read_text(load_state),) if load_state.exists() else tuple())
    doc_sources = tuple(
        _read_text(path)
        for path in (
            runtime_04_plan,
            runtime_index,
            *asset_docs,
            review,
            convergence,
        )
        if path.exists()
    )

    source_files, missing_source_files = _file_entries(root, RUNTIME_04_SOURCE_FILES)
    guard_files, missing_guard_files = _file_entries(root, RUNTIME_04_GUARD_FILES)

    worker_diagnostic_count = sum(
        1 for anchor in WORKER_DIAGNOSTIC_ANCHORS if anchor in "\n".join(worker_sources)
    )
    artifact_store_roundtrip_count = sum(
        "\n".join(artifact_store_roundtrip_sources).count(anchor)
        for anchor in (
            "artifact_store_roundtrips_scene_assets_with_mesh_references",
            "artifact_store_roundtrips_scene_assets_with_camera_targets",
            "artifact_store_roundtrips_scene_assets_with_physics_components",
            "artifact_store_roundtrips_scene_assets_with_script_binding_json_values",
        )
    )
    watcher_acceptance_doc_references = _line_references(
        root,
        runtime_04_plan,
        "watcher` 7/7",
    )
    artifact_acceptance_doc_references = _line_references(
        root,
        runtime_04_plan,
        "artifact_store_roundtrips_scene_assets_with",
    )
    retired_worker_new_references = [
        reference
        for reference in _line_references(root, worker_pool, "AssetWorkerPool::new(worker_count)")
        if "retired" not in reference["snippet"]
    ]
    retired_worker_request_sender_references = _line_references(
        root,
        worker_pool,
        "request_sender",
    )
    old_watch_debounce_references = _line_references(
        root,
        watch_loop,
        "const WATCH_DEBOUNCE",
    )

    missing_handle_state_anchors = _missing_snippets(
        handle_state_sources,
        HANDLE_STATE_ANCHORS,
    )
    missing_resource_reload_anchors = _missing_snippets(
        resource_reload_sources,
        RESOURCE_RELOAD_ANCHORS,
    )
    missing_worker_pool_anchors = _missing_snippets(worker_sources, WORKER_POOL_ANCHORS)
    missing_watcher_anchors = _missing_snippets(watcher_sources, WATCHER_ANCHORS)
    missing_artifact_cache_anchors = _missing_snippets(
        artifact_sources,
        ARTIFACT_CACHE_ANCHORS,
    )
    missing_test_anchors = _missing_snippets(guard_sources, RUNTIME_04_TEST_ANCHORS)
    missing_behavior_test_anchors = _missing_snippets(
        behavior_test_sources,
        RUNTIME_04_BEHAVIOR_TEST_ANCHORS,
    )
    missing_doc_anchors = _missing_snippets(doc_sources, RUNTIME_04_DOC_ANCHORS)
    missing_cargo_gate_anchors = _missing_snippets(doc_sources, CARGO_GATE_ANCHORS)
    mirror_docs_guard_present = not _missing_snippets(
        doc_sources + guard_sources,
        (MIRROR_DOCS_GUARD,),
    )

    risks: list[str] = []
    if missing_source_files:
        risks.append("Runtime 04 asset pipeline source owner files are missing.")
    if missing_guard_files:
        risks.append("Runtime 04 asset pipeline guard/test files are missing.")
    if missing_handle_state_anchors:
        risks.append("Runtime 04 handle/load-state/failure-reason anchors are missing.")
    if missing_resource_reload_anchors:
        risks.append("Runtime 04 resource reload transition anchors are missing.")
    if missing_worker_pool_anchors:
        risks.append("Runtime 04 worker-pool policy anchors are missing.")
    if worker_diagnostic_count != EXPECTED_WORKER_DIAGNOSTIC_COUNT:
        risks.append("Runtime 04 worker diagnostics count changed without audit sync.")
    if missing_watcher_anchors:
        risks.append("Runtime 04 watcher debounce/error anchors are missing.")
    if missing_artifact_cache_anchors:
        risks.append("Runtime 04 artifact cache wire anchors are missing.")
    if artifact_store_roundtrip_count != EXPECTED_ARTIFACT_STORE_ROUNDTRIP_COUNT:
        risks.append("Runtime 04 artifact-store roundtrip guard count changed.")
    if len(watcher_acceptance_doc_references) < 1:
        risks.append("Runtime 04 watcher 7/7 acceptance evidence is missing from the plan.")
    if len(artifact_acceptance_doc_references) < 1:
        risks.append("Runtime 04 artifact-store 4/4 evidence is missing from the plan.")
    if missing_test_anchors:
        risks.append("Runtime 04 named guard/test anchors are missing.")
    if missing_behavior_test_anchors:
        risks.append("Runtime 04 named behavior-test anchors are missing.")
    if missing_doc_anchors:
        risks.append("Runtime 04 plan or mirror docs are missing required status anchors.")
    if missing_cargo_gate_anchors:
        risks.append("Runtime 04 managed Cargo gate anchors are missing from docs.")
    if not mirror_docs_guard_present:
        risks.append("Runtime 04 mirror-doc guard anchor is missing from docs or guards.")
    if retired_worker_new_references:
        risks.append("Runtime 04 retired worker_count constructor reappeared in source.")
    if retired_worker_request_sender_references:
        risks.append("Runtime 04 retired worker request_sender bypass reappeared in source.")
    if old_watch_debounce_references:
        risks.append("Runtime 04 old WATCH_DEBOUNCE constant reappeared in watch_loop.")

    return {
        "source_files": source_files,
        "expected_source_file_count": EXPECTED_SOURCE_FILE_COUNT,
        "missing_source_files": missing_source_files,
        "guard_files": guard_files,
        "expected_guard_file_count": EXPECTED_GUARD_FILE_COUNT,
        "missing_guard_files": missing_guard_files,
        "missing_handle_state_anchors": missing_handle_state_anchors,
        "missing_resource_reload_anchors": missing_resource_reload_anchors,
        "missing_worker_pool_anchors": missing_worker_pool_anchors,
        "worker_diagnostic_count": worker_diagnostic_count,
        "expected_worker_diagnostic_count": EXPECTED_WORKER_DIAGNOSTIC_COUNT,
        "missing_watcher_anchors": missing_watcher_anchors,
        "missing_artifact_cache_anchors": missing_artifact_cache_anchors,
        "artifact_store_roundtrip_count": artifact_store_roundtrip_count,
        "expected_artifact_store_roundtrip_count": EXPECTED_ARTIFACT_STORE_ROUNDTRIP_COUNT,
        "watcher_acceptance_reference_count": len(watcher_acceptance_doc_references),
        "expected_watcher_acceptance_count": EXPECTED_WATCHER_TEST_COUNT,
        "artifact_acceptance_reference_count": len(artifact_acceptance_doc_references),
        "missing_test_anchors": missing_test_anchors,
        "test_anchor_count": len(RUNTIME_04_TEST_ANCHORS),
        "missing_behavior_test_anchors": missing_behavior_test_anchors,
        "behavior_test_anchor_count": len(RUNTIME_04_BEHAVIOR_TEST_ANCHORS),
        "missing_doc_anchors": missing_doc_anchors,
        "missing_cargo_gate_anchors": missing_cargo_gate_anchors,
        "retired_worker_new_references": retired_worker_new_references,
        "retired_worker_request_sender_references": retired_worker_request_sender_references,
        "old_watch_debounce_references": old_watch_debounce_references,
        "mirror_docs_guard_present": mirror_docs_guard_present,
        "risks": risks,
    }
