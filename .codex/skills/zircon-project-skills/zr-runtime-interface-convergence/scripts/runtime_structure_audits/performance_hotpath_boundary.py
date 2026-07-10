from __future__ import annotations

from pathlib import Path

from runtime_structure_audits.large_file_ownership import large_file_ownership_gate
from runtime_structure_audits.module_inventory import runtime_inventory
from runtime_structure_audits.performance_hotpath_anchor_inventory import (
    ANIMATION_SCENE_ANCHORS,
    ASSET_WORKER_ANCHORS,
    CARGO_GATE_ANCHORS,
    CHANGE_COUNTER_ANCHORS,
    EXTRACT_COUNTER_ANCHORS,
    FRAME_SPAN_ANCHORS,
    HOTSPOT_GUARD_ANCHORS,
    MIRROR_DOCS_GUARD,
    PROFILE_COUNTER_HOTSPOT_ANCHORS,
    QUERY_COUNTER_ANCHORS,
    RUNTIME_07_DOC_ANCHORS,
    RUNTIME_07_TEST_ANCHORS,
)
from runtime_structure_audits.performance_hotpath_source_inventory import (
    EXPECTED_LARGE_FILE_OWNER_CLASSES,
    EXPECTED_SOURCE_FILE_COUNT,
    EXPECTED_TEST_FILE_COUNT,
    LARGE_FILE_HOTSPOT_THRESHOLD,
    RUNTIME_07_SOURCE_FILES,
    RUNTIME_07_TEST_FILES,
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


def _large_file_gate(root: Path, gate: dict[str, object] | None) -> dict[str, object]:
    if gate is not None:
        return gate
    inventory = runtime_inventory(root, LARGE_FILE_HOTSPOT_THRESHOLD)
    return large_file_ownership_gate(inventory.hotspots, LARGE_FILE_HOTSPOT_THRESHOLD)


def performance_hotpath_boundary_audit(
    root: Path,
    large_file_gate: dict[str, object] | None = None,
) -> dict[str, object]:
    session = root / "zircon_runtime/src/dynamic_api/session.rs"
    session_extract = root / "zircon_runtime/src/dynamic_api/session/extract.rs"
    session_extract_cache = (
        root / "zircon_runtime/src/dynamic_api/session/extract_cache.rs"
    )
    session_extract_stats = root / "zircon_runtime/src/dynamic_api/session/extract_stats.rs"
    runtime_loop = root / "zircon_runtime/src/dynamic_api/runtime_loop.rs"
    schedule_runner = root / "zircon_runtime/src/scene/ecs/schedule_runner.rs"
    profiling_macros = (
        root / "zircon_runtime/src/core/runtime/diagnostics/profiling/macros.rs"
    )
    query_state_root = root / "zircon_runtime/src/scene/ecs/query/query_state/mod.rs"
    query_state_cache = root / "zircon_runtime/src/scene/ecs/query/query_state/cache.rs"
    query_state_read_only_cached = (
        root / "zircon_runtime/src/scene/ecs/query/query_state/read_only_cached.rs"
    )
    query_state_stats = root / "zircon_runtime/src/scene/ecs/query/query_state/stats.rs"
    query_state_system_param = (
        root / "zircon_runtime/src/scene/ecs/query/query_state/system_param.rs"
    )
    system_param = root / "zircon_runtime/src/scene/ecs/system/system_param.rs"
    system_state = root / "zircon_runtime/src/scene/ecs/system/system_state.rs"
    param_set = root / "zircon_runtime/src/scene/ecs/system/param_set.rs"
    query_filter = root / "zircon_runtime/src/scene/ecs/query/query_filter.rs"
    query_iter = root / "zircon_runtime/src/scene/ecs/query/query_iter.rs"
    query_many_iter = root / "zircon_runtime/src/scene/ecs/query/query_many_iter.rs"
    world_performance_diagnostics = (
        root / "zircon_runtime/src/scene/world/performance_diagnostics.rs"
    )
    world_driver = root / "zircon_runtime/src/scene/module/world_driver.rs"
    change_detection_stats = root / "zircon_runtime/src/scene/ecs/change_detection/stats.rs"
    worker_pool = root / "zircon_runtime/src/asset/pipeline/worker_pool.rs"
    project_asset_manager_construction = (
        root
        / "zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs"
    )
    animation_scene_diagnostics = (
        root / "zircon_runtime/src/animation/scene_hook/diagnostics.rs"
    )
    animation_scene_events = root / "zircon_runtime/src/animation/scene_hook/events.rs"
    animation_scene_node_pose = (
        root / "zircon_runtime/src/animation/scene_hook/node_pose.rs"
    )
    animation_scene_pending = root / "zircon_runtime/src/animation/scene_hook/pending.rs"
    animation_scene_scan = root / "zircon_runtime/src/animation/scene_hook/scan.rs"
    animation_scene_tick = root / "zircon_runtime/src/animation/scene_hook/tick.rs"
    profiling_counter_hotspot = (
        root / "zircon_runtime/src/core/runtime/diagnostics/profiling/counter_hotspot.rs"
    )
    profiling_export = (
        root / "zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs"
    )
    profiling_mod = root / "zircon_runtime/src/core/runtime/diagnostics/profiling/mod.rs"
    interface_profiling = root / "zircon_runtime_interface/src/profiling.rs"

    source_files, missing_source_files = _file_entries(root, RUNTIME_07_SOURCE_FILES)
    test_files, missing_test_files = _file_entries(root, RUNTIME_07_TEST_FILES)

    frame_span_sources = tuple(
        _read_text(path)
        for path in (
            session,
            session_extract,
            runtime_loop,
            schedule_runner,
            profiling_macros,
        )
        if path.exists()
    )
    query_counter_sources = tuple(
        _read_text(path)
        for path in (
            query_state_root,
            query_state_cache,
            query_state_read_only_cached,
            query_state_stats,
            query_state_system_param,
            query_filter,
            query_iter,
            query_many_iter,
            system_param,
            system_state,
            param_set,
            world_performance_diagnostics,
            world_driver,
        )
        if path.exists()
    )
    change_counter_sources = tuple(
        _read_text(path)
        for path in (change_detection_stats, query_filter)
        if path.exists()
    )
    extract_counter_sources = tuple(
        _read_text(path)
        for path in (session, session_extract, session_extract_cache, session_extract_stats)
        if path.exists()
    )
    asset_worker_sources = tuple(
        _read_text(path)
        for path in (worker_pool, project_asset_manager_construction)
        if path.exists()
    )
    animation_scene_sources = tuple(
        _read_text(path)
        for path in (
            animation_scene_diagnostics,
            animation_scene_events,
            animation_scene_node_pose,
            animation_scene_pending,
            animation_scene_scan,
            animation_scene_tick,
        )
        if path.exists()
    )
    profile_counter_hotspot_sources = tuple(
        _read_text(path)
        for path in (
            interface_profiling,
            profiling_counter_hotspot,
            profiling_export,
            profiling_mod,
        )
        if path.exists()
    )
    test_sources = tuple(
        _read_text(root / file_name)
        for file_name in RUNTIME_07_TEST_FILES
        if (root / file_name).exists()
    )
    doc_paths = (
        root / "docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md",
        root
        / "docs/plans/zircon_runtime/runtime/07/2026-07-09-runtime-performance-hotpath-output-records.md",
        root / "docs/plans/zircon_runtime/runtime/index.md",
        root / "docs/zircon_runtime/performance/hotspot_inventory.md",
        root / "docs/zircon_runtime/dynamic_api/session.md",
        root / "docs/zircon_runtime/scene/ecs.md",
        root / "docs/zircon_runtime/animation/runtime.md",
        root / "docs/zircon_runtime/core/diagnostics.md",
        root / "docs/engine-architecture/runtime-interface-convergence.md",
        root / "docs/engine-architecture/runtime-architecture-review-m0.md",
    )
    doc_sources = tuple(_read_text(path) for path in doc_paths if path.exists())

    missing_frame_span_anchors = _missing_snippets(
        frame_span_sources,
        FRAME_SPAN_ANCHORS,
    )
    missing_query_counter_anchors = _missing_snippets(
        query_counter_sources,
        QUERY_COUNTER_ANCHORS,
    )
    missing_change_counter_anchors = _missing_snippets(
        change_counter_sources,
        CHANGE_COUNTER_ANCHORS,
    )
    missing_extract_counter_anchors = _missing_snippets(
        extract_counter_sources,
        EXTRACT_COUNTER_ANCHORS,
    )
    missing_asset_worker_anchors = _missing_snippets(
        asset_worker_sources,
        ASSET_WORKER_ANCHORS,
    )
    missing_animation_scene_anchors = _missing_snippets(
        animation_scene_sources,
        ANIMATION_SCENE_ANCHORS,
    )
    missing_profile_counter_hotspot_anchors = _missing_snippets(
        profile_counter_hotspot_sources,
        PROFILE_COUNTER_HOTSPOT_ANCHORS,
    )
    missing_hotspot_guard_anchors = _missing_snippets(
        test_sources + doc_sources,
        HOTSPOT_GUARD_ANCHORS,
    )
    missing_test_anchors = _missing_snippets(test_sources, RUNTIME_07_TEST_ANCHORS)
    missing_doc_anchors = _missing_snippets(doc_sources, RUNTIME_07_DOC_ANCHORS)
    missing_cargo_gate_anchors = _missing_snippets(doc_sources, CARGO_GATE_ANCHORS)
    mirror_docs_guard_present = not _missing_snippets(
        doc_sources + test_sources,
        (MIRROR_DOCS_GUARD,),
    )
    stale_hotspot_placeholder_present = any(
        "热点清单 top3：__" in source for source in doc_sources
    )
    large_file_gate_report = _large_file_gate(root, large_file_gate)
    large_file_hotspot_count = int(large_file_gate_report["hotspot_count"])
    large_file_owner_classes = sorted(
        str(owner_class)
        for owner_class in large_file_gate_report["classification_counts"].keys()
    )
    missing_large_file_owner_classes = (
        []
        if large_file_hotspot_count == 0
        else [
            owner_class
            for owner_class in EXPECTED_LARGE_FILE_OWNER_CLASSES
            if owner_class not in large_file_owner_classes
        ]
    )

    risks: list[str] = []
    if missing_source_files:
        risks.append("Runtime 07 performance hotpath source files are missing.")
    if missing_test_files:
        risks.append("Runtime 07 performance hotpath guard/test files are missing.")
    if missing_frame_span_anchors:
        risks.append("Runtime 07 frame profiling span anchors are missing.")
    if missing_query_counter_anchors:
        risks.append("Runtime 07 QueryState telemetry anchors are missing.")
    if missing_change_counter_anchors:
        risks.append("Runtime 07 change-detection telemetry anchors are missing.")
    if missing_extract_counter_anchors:
        risks.append("Runtime 07 extract telemetry anchors are missing.")
    if missing_asset_worker_anchors:
        risks.append("Runtime 07 asset-worker candidate telemetry anchors are missing.")
    if missing_animation_scene_anchors:
        risks.append("Runtime 07 animation scene telemetry anchors are missing.")
    if missing_profile_counter_hotspot_anchors:
        risks.append("Runtime 07 profiling counter hotspot export anchors are missing.")
    if missing_hotspot_guard_anchors:
        risks.append("Runtime 07 hotspot inventory guard anchors are missing.")
    if missing_test_anchors:
        risks.append("Runtime 07 named counter assertion anchors are missing.")
    if missing_doc_anchors:
        risks.append("Runtime 07 plan or mirror docs are missing required status anchors.")
    if missing_cargo_gate_anchors:
        risks.append("Runtime 07 pending Cargo/profiling/FPS gate anchors are missing from docs.")
    if not mirror_docs_guard_present:
        risks.append("Runtime 07 mirror-doc guard anchor is missing from docs or guards.")
    if stale_hotspot_placeholder_present:
        risks.append("Runtime 07 hotspot inventory still contains the old top3 placeholder.")
    if missing_large_file_owner_classes:
        risks.append(
            "Runtime 07 owner-budgeted optimization gate is missing expected "
            "large-file owner classes."
        )
    if large_file_gate_report["unclassified_hotspot_count"]:
        risks.append(
            "Runtime 07 owner-budgeted optimization gate has unclassified "
            "large production-file hotspots."
        )

    return {
        "source_files": source_files,
        "expected_source_file_count": EXPECTED_SOURCE_FILE_COUNT,
        "missing_source_files": missing_source_files,
        "test_files": test_files,
        "expected_test_file_count": EXPECTED_TEST_FILE_COUNT,
        "missing_test_files": missing_test_files,
        "frame_span_anchor_count": len(FRAME_SPAN_ANCHORS),
        "missing_frame_span_anchors": missing_frame_span_anchors,
        "query_counter_anchor_count": len(QUERY_COUNTER_ANCHORS),
        "missing_query_counter_anchors": missing_query_counter_anchors,
        "change_counter_anchor_count": len(CHANGE_COUNTER_ANCHORS),
        "missing_change_counter_anchors": missing_change_counter_anchors,
        "extract_counter_anchor_count": len(EXTRACT_COUNTER_ANCHORS),
        "missing_extract_counter_anchors": missing_extract_counter_anchors,
        "asset_worker_anchor_count": len(ASSET_WORKER_ANCHORS),
        "missing_asset_worker_anchors": missing_asset_worker_anchors,
        "animation_scene_anchor_count": len(ANIMATION_SCENE_ANCHORS),
        "missing_animation_scene_anchors": missing_animation_scene_anchors,
        "profile_counter_hotspot_anchor_count": len(PROFILE_COUNTER_HOTSPOT_ANCHORS),
        "missing_profile_counter_hotspot_anchors": missing_profile_counter_hotspot_anchors,
        "hotspot_guard_anchor_count": len(HOTSPOT_GUARD_ANCHORS),
        "missing_hotspot_guard_anchors": missing_hotspot_guard_anchors,
        "test_anchor_count": len(RUNTIME_07_TEST_ANCHORS),
        "missing_test_anchors": missing_test_anchors,
        "doc_anchor_count": len(RUNTIME_07_DOC_ANCHORS),
        "missing_doc_anchors": missing_doc_anchors,
        "cargo_gate_anchor_count": len(CARGO_GATE_ANCHORS),
        "missing_cargo_gate_anchors": missing_cargo_gate_anchors,
        "mirror_docs_guard": MIRROR_DOCS_GUARD,
        "mirror_docs_guard_present": mirror_docs_guard_present,
        "stale_hotspot_placeholder_present": stale_hotspot_placeholder_present,
        "large_file_hotspot_threshold": LARGE_FILE_HOTSPOT_THRESHOLD,
        "large_file_hotspot_count": large_file_hotspot_count,
        "large_file_m1_gate_status": large_file_gate_report["m1_gate_status"],
        "large_file_migration_debt_count": large_file_gate_report[
            "large_file_migration_debt_count"
        ],
        "large_file_owner_class_count": large_file_gate_report["classification_count"],
        "large_file_owner_classes": large_file_owner_classes,
        "missing_large_file_owner_classes": missing_large_file_owner_classes,
        "large_file_unclassified_hotspot_count": large_file_gate_report[
            "unclassified_hotspot_count"
        ],
        "risks": risks,
    }
