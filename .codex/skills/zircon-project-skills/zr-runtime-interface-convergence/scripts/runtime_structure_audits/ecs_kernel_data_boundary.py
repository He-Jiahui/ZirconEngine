from __future__ import annotations

from pathlib import Path

from runtime_structure_audits.ecs_kernel_data_anchor_inventory import (
    ARCHETYPE_ANCHORS,
    CARGO_GATE_ANCHORS,
    CHANGE_TICK_ANCHORS,
    COMMAND_ANCHORS,
    COMPONENT_IDENTITY_ANCHORS,
    COMPONENT_STORAGE_PRIVATE_REEXPORT_ANCHORS,
    COMPONENT_STORAGE_PRIVATE_REEXPORT_FORBIDDEN_SNIPPETS,
    ENTITY_LIFECYCLE_ANCHORS,
    EVENT_MESSAGE_ANCHORS,
    OBSERVER_ANCHORS,
    RESOURCE_IDENTITY_ANCHORS,
    RUNTIME_08_BEHAVIOR_TEST_ANCHORS,
    RUNTIME_08_DOC_ANCHORS,
    RUNTIME_08_TEST_ANCHORS,
    STORAGE_ANCHORS,
)
from runtime_structure_audits.ecs_kernel_data_source_inventory import (
    EXPECTED_SOURCE_FILE_COUNT,
    EXPECTED_TEST_FILE_COUNT,
    MIRROR_DOCS_GUARD,
    RUNTIME_08_SOURCE_FILES,
    RUNTIME_08_TEST_FILES,
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


def _missing_file_snippets(
    root: Path,
    file_snippets: tuple[tuple[str, str], ...],
) -> list[str]:
    missing: list[str] = []
    for file_name, snippet in file_snippets:
        path = root / file_name
        if not path.exists() or snippet not in _read_text(path):
            missing.append(f"{file_name}: {snippet}")
    return missing


def _present_file_snippets(
    root: Path,
    file_snippets: tuple[tuple[str, str], ...],
) -> list[str]:
    present: list[str] = []
    for file_name, snippet in file_snippets:
        path = root / file_name
        if path.exists() and snippet in _read_text(path):
            present.append(f"{file_name}: {snippet}")
    return present


def ecs_kernel_data_boundary_audit(root: Path) -> dict[str, object]:
    archetype_sources = tuple(
        _read_text(root / file_name)
        for file_name in (
            "zircon_runtime/src/scene/ecs/archetype/mod.rs",
            "zircon_runtime/src/scene/ecs/archetype/id.rs",
            "zircon_runtime/src/scene/ecs/archetype/index.rs",
            "zircon_runtime/src/scene/ecs/archetype/move_result.rs",
            "zircon_runtime/src/scene/ecs/archetype/record.rs",
            "zircon_runtime/src/scene/ecs/archetype/signature.rs",
        )
        if (root / file_name).exists()
    )
    storage_sources = tuple(
        _read_text(root / file_name)
        for file_name in (
            "zircon_runtime/src/scene/ecs/storage_type.rs",
            "zircon_runtime/src/scene/ecs/storage/component_storage/mod.rs",
            "zircon_runtime/src/scene/ecs/storage/component_storage/entry.rs",
            "zircon_runtime/src/scene/ecs/storage/component_storage/location.rs",
            "zircon_runtime/src/scene/ecs/storage/component_storage/sparse.rs",
            "zircon_runtime/src/scene/ecs/storage/component_storage/sparse/locator.rs",
            "zircon_runtime/src/scene/ecs/storage/component_storage/store.rs",
            "zircon_runtime/src/scene/ecs/storage/component_storage/table.rs",
            "zircon_runtime/src/scene/ecs/storage/component_storage/component_results.rs",
        )
        if (root / file_name).exists()
    )
    component_identity_sources = tuple(
        _read_text(root / file_name)
        for file_name in (
            "zircon_runtime/src/scene/ecs/component/mod.rs",
            "zircon_runtime/src/scene/ecs/component/id.rs",
            "zircon_runtime/src/scene/ecs/component/marker.rs",
            "zircon_runtime/src/scene/ecs/component/registry.rs",
            "zircon_runtime/src/scene/ecs/component/registry/transferred.rs",
        )
        if (root / file_name).exists()
    )
    entity_sources = tuple(
        _read_text(root / file_name)
        for file_name in (
            "zircon_runtime/src/scene/ecs/entity/mod.rs",
            "zircon_runtime/src/scene/ecs/entity/despawned.rs",
            "zircon_runtime/src/scene/ecs/entity/error.rs",
            "zircon_runtime/src/scene/ecs/entity/internal.rs",
            "zircon_runtime/src/scene/ecs/entity/location.rs",
            "zircon_runtime/src/scene/ecs/entity/registry.rs",
            "zircon_runtime/src/scene/ecs/entity/slot.rs",
            "zircon_runtime/src/scene/ecs/entity/stable_location.rs",
            "zircon_runtime/src/scene/world/identity.rs",
        )
        if (root / file_name).exists()
    )
    observer_sources = tuple(
        _read_text(root / file_name)
        for file_name in (
            "zircon_runtime/src/scene/ecs/observer/mod.rs",
            "zircon_runtime/src/scene/ecs/observer/callbacks.rs",
            "zircon_runtime/src/scene/ecs/observer/entry.rs",
            "zircon_runtime/src/scene/ecs/observer/id.rs",
            "zircon_runtime/src/scene/ecs/observer/store.rs",
            "zircon_runtime/src/scene/ecs/observer/callback_registry.rs",
            "zircon_runtime/src/scene/world/observers.rs",
        )
        if (root / file_name).exists()
    )
    command_sources = tuple(
        _read_text(root / file_name)
        for file_name in (
            "zircon_runtime/src/scene/ecs/commands/command.rs",
            "zircon_runtime/src/scene/ecs/commands/command_queue.rs",
            "zircon_runtime/src/scene/ecs/commands/commands/mod.rs",
            "zircon_runtime/src/scene/ecs/commands/commands/entity_commands.rs",
            "zircon_runtime/src/scene/ecs/commands/commands/facade.rs",
            "zircon_runtime/src/scene/ecs/commands/commands/param.rs",
            "zircon_runtime/src/scene/world/commands.rs",
        )
        if (root / file_name).exists()
    )
    event_message_sources = tuple(
        _read_text(root / file_name)
        for file_name in (
            "zircon_runtime/src/scene/ecs/events/mod.rs",
            "zircon_runtime/src/scene/ecs/events/cursor.rs",
            "zircon_runtime/src/scene/ecs/events/id.rs",
            "zircon_runtime/src/scene/ecs/events/metrics.rs",
            "zircon_runtime/src/scene/ecs/events/queue.rs",
            "zircon_runtime/src/scene/ecs/events/store.rs",
            "zircon_runtime/src/scene/ecs/events/subscription.rs",
            "zircon_runtime/src/scene/ecs/messages/mod.rs",
            "zircon_runtime/src/scene/ecs/messages/cursor.rs",
            "zircon_runtime/src/scene/ecs/messages/id.rs",
            "zircon_runtime/src/scene/ecs/messages/queue.rs",
            "zircon_runtime/src/scene/ecs/messages/store.rs",
            "zircon_runtime/src/scene/world/events.rs",
            "zircon_runtime/src/scene/world/messages.rs",
        )
        if (root / file_name).exists()
    )
    resource_identity_sources = tuple(
        _read_text(root / file_name)
        for file_name in (
            "zircon_runtime/src/scene/ecs/resource/mod.rs",
            "zircon_runtime/src/scene/ecs/resource/id.rs",
            "zircon_runtime/src/core/framework/scene/resource.rs",
            "zircon_runtime/src/scene/ecs/resource/registry.rs",
        )
        if (root / file_name).exists()
    )
    change_tick_sources = tuple(
        _read_text(root / file_name)
        for file_name in (
            "zircon_runtime/src/scene/ecs/change_detection/mod.rs",
            "zircon_runtime/src/scene/ecs/change_detection/change_tick.rs",
            "zircon_runtime/src/scene/ecs/change_detection/change_tick_window.rs",
            "zircon_runtime/src/scene/ecs/change_detection/component_ticks.rs",
            "zircon_runtime/src/scene/ecs/change_detection/stats.rs",
            "zircon_runtime/src/scene/ecs/change_detection/wrappers.rs",
        )
        if (root / file_name).exists()
    )
    test_sources = tuple(
        _read_text(root / file_name)
        for file_name in RUNTIME_08_TEST_FILES
        if (root / file_name).exists()
    )
    doc_paths = (
        root / "docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md",
        root / "docs/plans/zircon_runtime/runtime/index.md",
        root / "docs/zircon_runtime/scene/ecs.md",
        root / "docs/engine-architecture/runtime-interface-convergence.md",
        root / "docs/engine-architecture/runtime-architecture-review-m0.md",
    )
    doc_sources = tuple(_read_text(path) for path in doc_paths if path.exists())

    source_files, missing_source_files = _file_entries(root, RUNTIME_08_SOURCE_FILES)
    test_files, missing_test_files = _file_entries(root, RUNTIME_08_TEST_FILES)

    missing_archetype_anchors = _missing_snippets(archetype_sources, ARCHETYPE_ANCHORS)
    missing_storage_anchors = _missing_snippets(storage_sources, STORAGE_ANCHORS)
    missing_component_storage_private_reexport_anchors = _missing_file_snippets(
        root,
        COMPONENT_STORAGE_PRIVATE_REEXPORT_ANCHORS,
    )
    unexpected_component_storage_private_reexports = _present_file_snippets(
        root,
        COMPONENT_STORAGE_PRIVATE_REEXPORT_FORBIDDEN_SNIPPETS,
    )
    missing_component_identity_anchors = _missing_snippets(
        component_identity_sources,
        COMPONENT_IDENTITY_ANCHORS,
    )
    missing_entity_lifecycle_anchors = _missing_snippets(
        entity_sources,
        ENTITY_LIFECYCLE_ANCHORS,
    )
    missing_observer_anchors = _missing_snippets(observer_sources, OBSERVER_ANCHORS)
    missing_command_anchors = _missing_snippets(command_sources, COMMAND_ANCHORS)
    missing_event_message_anchors = _missing_snippets(
        event_message_sources,
        EVENT_MESSAGE_ANCHORS,
    )
    missing_resource_identity_anchors = _missing_snippets(
        resource_identity_sources,
        RESOURCE_IDENTITY_ANCHORS,
    )
    missing_change_tick_anchors = _missing_snippets(
        change_tick_sources,
        CHANGE_TICK_ANCHORS,
    )
    missing_test_anchors = _missing_snippets(test_sources, RUNTIME_08_TEST_ANCHORS)
    missing_behavior_test_anchors = _missing_snippets(
        test_sources,
        RUNTIME_08_BEHAVIOR_TEST_ANCHORS,
    )
    missing_doc_anchors = _missing_snippets(doc_sources, RUNTIME_08_DOC_ANCHORS)
    missing_cargo_gate_anchors = _missing_snippets(doc_sources, CARGO_GATE_ANCHORS)

    risks: list[str] = []
    if missing_source_files:
        risks.append("Runtime 08 ECS data-kernel source files are missing.")
    if missing_test_files:
        risks.append("Runtime 08 ECS data-kernel guard/test files are missing.")
    if missing_archetype_anchors:
        risks.append("archetype identity, signature, and index semantics are no longer statically visible.")
    if missing_storage_anchors:
        risks.append("dual storage semantics are no longer statically visible.")
    if missing_component_storage_private_reexport_anchors:
        risks.append("component storage sibling-owner imports are no longer statically visible.")
    if unexpected_component_storage_private_reexports:
        risks.append("component storage parent private re-export drift has returned.")
    if missing_component_identity_anchors:
        risks.append("component identity semantics are no longer statically visible.")
    if missing_entity_lifecycle_anchors:
        risks.append("generational entity lifecycle semantics are no longer statically visible.")
    if missing_observer_anchors:
        risks.append("observer entry or clone-out dispatch semantics are no longer statically visible.")
    if missing_command_anchors:
        risks.append("deferred command reporting semantics are no longer statically visible.")
    if missing_event_message_anchors:
        risks.append("event/message split semantics are no longer statically visible.")
    if missing_resource_identity_anchors:
        risks.append("resource identity semantics are no longer statically visible.")
    if missing_change_tick_anchors:
        risks.append("wrap-aware change tick semantics are no longer statically visible.")
    if missing_test_anchors:
        risks.append("Runtime 08 behavior or cargo-gate test anchors are missing.")
    if missing_behavior_test_anchors:
        risks.append("Runtime 08 ECS behavior test anchors are missing.")
    if not any(MIRROR_DOCS_GUARD in source for source in test_sources):
        risks.append("Runtime 08 ECS data-kernel mirror-doc aggregate guard is missing.")
    if missing_doc_anchors:
        risks.append("Runtime 08 documentation mirror anchors are missing.")
    if missing_cargo_gate_anchors:
        risks.append("Runtime 08 pending Cargo gate commands are missing from docs.")

    return {
        "source_files": source_files,
        "expected_source_file_count": EXPECTED_SOURCE_FILE_COUNT,
        "missing_source_files": missing_source_files,
        "test_files": test_files,
        "expected_test_file_count": EXPECTED_TEST_FILE_COUNT,
        "missing_test_files": missing_test_files,
        "archetype_anchor_count": len(ARCHETYPE_ANCHORS),
        "missing_archetype_anchors": missing_archetype_anchors,
        "storage_anchor_count": len(STORAGE_ANCHORS),
        "missing_storage_anchors": missing_storage_anchors,
        "component_storage_private_reexport_anchor_count": len(
            COMPONENT_STORAGE_PRIVATE_REEXPORT_ANCHORS
        ),
        "missing_component_storage_private_reexport_anchors": (
            missing_component_storage_private_reexport_anchors
        ),
        "unexpected_component_storage_private_reexports": (
            unexpected_component_storage_private_reexports
        ),
        "component_identity_anchor_count": len(COMPONENT_IDENTITY_ANCHORS),
        "missing_component_identity_anchors": missing_component_identity_anchors,
        "entity_lifecycle_anchor_count": len(ENTITY_LIFECYCLE_ANCHORS),
        "missing_entity_lifecycle_anchors": missing_entity_lifecycle_anchors,
        "observer_anchor_count": len(OBSERVER_ANCHORS),
        "missing_observer_anchors": missing_observer_anchors,
        "command_anchor_count": len(COMMAND_ANCHORS),
        "missing_command_anchors": missing_command_anchors,
        "event_message_anchor_count": len(EVENT_MESSAGE_ANCHORS),
        "missing_event_message_anchors": missing_event_message_anchors,
        "resource_identity_anchor_count": len(RESOURCE_IDENTITY_ANCHORS),
        "missing_resource_identity_anchors": missing_resource_identity_anchors,
        "change_tick_anchor_count": len(CHANGE_TICK_ANCHORS),
        "missing_change_tick_anchors": missing_change_tick_anchors,
        "test_anchor_count": len(RUNTIME_08_TEST_ANCHORS),
        "missing_test_anchors": missing_test_anchors,
        "behavior_test_anchor_count": len(RUNTIME_08_BEHAVIOR_TEST_ANCHORS),
        "missing_behavior_test_anchors": missing_behavior_test_anchors,
        "doc_anchor_count": len(RUNTIME_08_DOC_ANCHORS),
        "missing_doc_anchors": missing_doc_anchors,
        "cargo_gate_anchor_count": len(CARGO_GATE_ANCHORS),
        "missing_cargo_gate_anchors": missing_cargo_gate_anchors,
        "mirror_docs_guard": MIRROR_DOCS_GUARD,
        "mirror_docs_guard_present": any(
            MIRROR_DOCS_GUARD in source for source in test_sources
        ),
        "risks": risks,
    }
