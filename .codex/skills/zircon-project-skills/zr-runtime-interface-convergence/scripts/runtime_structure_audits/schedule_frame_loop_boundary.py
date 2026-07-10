from __future__ import annotations

import re
from pathlib import Path

from .schedule_frame_loop_anchor_inventory import (
    CARGO_GATE_ANCHORS,
    FIXED_PLAN_ANCHORS,
    FRAME_SCHEDULE_DOC_ANCHORS,
    MIRROR_DOCS_GUARD,
    PARALLEL_EXECUTOR_ANCHORS,
    RUNTIME_03_BEHAVIOR_TEST_ANCHORS,
    RUNTIME_03_DOC_ANCHORS,
    RUNTIME_03_TEST_ANCHORS,
    SCHEDULE_ORDER_ANCHORS,
    SCHEDULE_RUNNER_ANCHORS,
    SESSION_TICK_ANCHORS,
    STAGE_VARIANT_ANCHORS,
    SYSTEM_STAGE_ANCHORS,
    TIME_HANDOFF_ANCHORS,
    UI_EXTRACT_ANCHORS,
)
from .schedule_frame_loop_source_inventory import (
    EXPECTED_DYNAMIC_SESSION_TICK_TIME_CALLS,
    EXPECTED_FIXED_LOOP_STAGE_COUNT,
    EXPECTED_STAGE_COUNT,
    RUNTIME_03_GUARD_FILES,
    RUNTIME_03_SOURCE_FILES,
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


def _count_declared_stage_variants(system_stage_source: str) -> int:
    enum_start = system_stage_source.find("pub enum SystemStage")
    impl_start = system_stage_source.find("impl SystemStage")
    if enum_start == -1 or impl_start == -1:
        return 0

    enum_body = system_stage_source[enum_start:impl_start]
    return sum(1 for line in enum_body.splitlines() if line.strip().endswith(","))


def _const_usize(source: str, name: str) -> int | None:
    match = re.search(rf"pub const {re.escape(name)}: usize = (\d+);", source)
    if not match:
        return None
    return int(match.group(1))


def _fixed_loop_stage_count(system_stage_source: str) -> int:
    match = re.search(r"pub const FIXED_LOOP: \[Self; (\d+)\]", system_stage_source)
    if not match:
        return 0
    return int(match.group(1))


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


def _current_ui_extract_body(session_source: str) -> str:
    start = session_source.find("fn current_ui_extract(&self)")
    if start == -1:
        return ""

    next_method = session_source[start + 1 :].find("\n    fn ")
    if next_method == -1:
        return session_source[start:]
    return session_source[start : start + 1 + next_method]


def schedule_frame_loop_boundary_audit(root: Path) -> dict[str, object]:
    system_stage = root / "zircon_runtime/src/scene/ecs/system_stage.rs"
    session = root / "zircon_runtime/src/dynamic_api/session.rs"
    session_profile = root / "zircon_runtime/src/dynamic_api/session/profile.rs"
    session_extract = root / "zircon_runtime/src/dynamic_api/session/extract.rs"
    runtime_loop = root / "zircon_runtime/src/dynamic_api/runtime_loop.rs"
    level_system = root / "zircon_runtime/src/scene/level_system.rs"
    world_driver = root / "zircon_runtime/src/scene/module/world_driver.rs"
    fixed_step_plan = root / "zircon_runtime/src/core/framework/time/fixed_step_plan.rs"
    schedule_stage_plan = root / "zircon_runtime/src/scene/ecs/schedule_stage_plan.rs"
    scene_system_descriptor = root / "zircon_runtime/src/scene/ecs/scene_system_descriptor.rs"
    scene_system_registry = root / "zircon_runtime/src/scene/ecs/scene_system_registry.rs"
    schedule_runner = root / "zircon_runtime/src/scene/ecs/schedule_runner.rs"
    schedule_parallel_executor = (
        root / "zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs"
    )
    runtime_03_plan = (
        root
        / "docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md"
    )
    runtime_index = root / "docs/plans/zircon_runtime/runtime/index.md"
    frame_schedule_doc = root / "docs/zircon_runtime/core/frame_schedule.md"
    schedule_parallel_doc = (
        root / "docs/zircon_runtime/scene/ecs/schedule_parallel_executor.md"
    )
    review = root / "docs/engine-architecture/runtime-architecture-review-m0.md"
    convergence = root / "docs/engine-architecture/runtime-interface-convergence.md"

    system_stage_source = _read_text(system_stage) if system_stage.exists() else ""
    session_source = _read_text(session) if session.exists() else ""
    session_profile_source = (
        _read_text(session_profile) if session_profile.exists() else ""
    )
    session_extract_source = (
        _read_text(session_extract) if session_extract.exists() else ""
    )
    runtime_loop_source = _read_text(runtime_loop) if runtime_loop.exists() else ""
    level_system_source = _read_text(level_system) if level_system.exists() else ""
    world_driver_source = _read_text(world_driver) if world_driver.exists() else ""
    fixed_step_plan_source = (
        _read_text(fixed_step_plan) if fixed_step_plan.exists() else ""
    )
    schedule_stage_plan_source = (
        _read_text(schedule_stage_plan) if schedule_stage_plan.exists() else ""
    )
    scene_system_descriptor_source = (
        _read_text(scene_system_descriptor)
        if scene_system_descriptor.exists()
        else ""
    )
    scene_system_registry_source = (
        _read_text(scene_system_registry) if scene_system_registry.exists() else ""
    )
    schedule_runner_source = (
        _read_text(schedule_runner) if schedule_runner.exists() else ""
    )
    schedule_parallel_executor_source = (
        _read_text(schedule_parallel_executor)
        if schedule_parallel_executor.exists()
        else ""
    )

    source_files, missing_source_files = _file_entries(root, RUNTIME_03_SOURCE_FILES)
    guard_files, missing_guard_files = _file_entries(root, RUNTIME_03_GUARD_FILES)

    doc_sources = tuple(
        _read_text(path)
        for path in (
            runtime_03_plan,
            runtime_index,
            frame_schedule_doc,
            schedule_parallel_doc,
            review,
            convergence,
        )
        if path.exists()
    )
    guard_sources = tuple(
        _read_text(root / file_name)
        for file_name in RUNTIME_03_GUARD_FILES
        if (root / file_name).exists()
    )
    stage_order_sources = (
        schedule_stage_plan_source,
        scene_system_descriptor_source,
        scene_system_registry_source,
    )
    tick_handoff_sources = (
        session_source,
        level_system_source,
        world_driver_source,
    )
    ui_extract_sources = (
        session_source,
        session_extract_source,
        runtime_loop_source,
        _read_text(root / "zircon_runtime/src/dynamic_api/session/hud.rs")
        if (root / "zircon_runtime/src/dynamic_api/session/hud.rs").exists()
        else "",
        _read_text(root / "zircon_runtime/src/dynamic_api/session/menu.rs")
        if (root / "zircon_runtime/src/dynamic_api/session/menu.rs").exists()
        else "",
    )

    stage_count = _const_usize(system_stage_source, "COUNT")
    declared_stage_variant_count = _count_declared_stage_variants(system_stage_source)
    fixed_loop_stage_count = _fixed_loop_stage_count(system_stage_source)
    dynamic_session_tick_time_call_count = session_source.count(".tick_time(")
    ui_extract_body = _current_ui_extract_body(session_extract_source)
    ui_extract_render_stage_references = (
        _line_references(root, session_extract, "SystemStage::RenderExtract")
        if "SystemStage::RenderExtract" in ui_extract_body
        else []
    )
    world_driver_second_advance_references = _line_references(
        root,
        world_driver,
        "advance_time_by(",
    )
    session_raw_delta_tick_references = _line_references(
        root,
        session,
        ".tick(&self.runtime.handle(), advance.real_delta()",
    )

    missing_stage_variants = _missing_snippets(
        (system_stage_source,),
        STAGE_VARIANT_ANCHORS,
    )
    missing_system_stage_anchors = _missing_snippets(
        (system_stage_source,),
        SYSTEM_STAGE_ANCHORS,
    )
    missing_session_tick_anchors = _missing_snippets(
        (session_source, session_profile_source),
        SESSION_TICK_ANCHORS,
    )
    missing_time_handoff_anchors = _missing_snippets(
        tick_handoff_sources,
        TIME_HANDOFF_ANCHORS,
    )
    missing_fixed_plan_anchors = _missing_snippets(
        (fixed_step_plan_source,),
        FIXED_PLAN_ANCHORS,
    )
    missing_ui_extract_anchors = _missing_snippets(
        ui_extract_sources,
        UI_EXTRACT_ANCHORS,
    )
    missing_schedule_order_anchors = _missing_snippets(
        stage_order_sources,
        SCHEDULE_ORDER_ANCHORS,
    )
    missing_schedule_runner_anchors = _missing_snippets(
        (schedule_runner_source,),
        SCHEDULE_RUNNER_ANCHORS,
    )
    missing_parallel_executor_anchors = _missing_snippets(
        (schedule_parallel_executor_source,),
        PARALLEL_EXECUTOR_ANCHORS,
    )
    missing_test_anchors = _missing_snippets(guard_sources, RUNTIME_03_TEST_ANCHORS)
    missing_behavior_test_anchors = _missing_snippets(
        guard_sources,
        RUNTIME_03_BEHAVIOR_TEST_ANCHORS,
    )
    missing_doc_anchors = _missing_snippets(doc_sources, RUNTIME_03_DOC_ANCHORS)
    missing_frame_schedule_doc_anchors = _missing_snippets(
        (_read_text(frame_schedule_doc) if frame_schedule_doc.exists() else "",),
        FRAME_SCHEDULE_DOC_ANCHORS,
    )
    missing_cargo_gate_anchors = _missing_snippets(doc_sources, CARGO_GATE_ANCHORS)
    mirror_docs_guard_present = any(MIRROR_DOCS_GUARD in source for source in guard_sources)

    risks: list[str] = []
    if missing_source_files:
        risks.append("Runtime 03 source files for schedule/frame-loop ownership are missing.")
    if missing_guard_files:
        risks.append("Runtime 03 guard files are missing.")
    if stage_count != EXPECTED_STAGE_COUNT:
        risks.append("Runtime 03 SystemStage COUNT changed without plan/audit sync.")
    if declared_stage_variant_count != EXPECTED_STAGE_COUNT:
        risks.append("Runtime 03 SystemStage variant count changed without plan/audit sync.")
    if fixed_loop_stage_count != EXPECTED_FIXED_LOOP_STAGE_COUNT:
        risks.append("Runtime 03 fixed-loop stage count changed without plan/audit sync.")
    if dynamic_session_tick_time_call_count != EXPECTED_DYNAMIC_SESSION_TICK_TIME_CALLS:
        risks.append("Runtime 03 dynamic session tick_time call count changed.")
    if missing_stage_variants:
        risks.append("Runtime 03 SystemStage variant anchors are missing.")
    if missing_system_stage_anchors:
        risks.append("Runtime 03 SystemStage authority anchors are missing.")
    if missing_session_tick_anchors:
        risks.append("Runtime 03 dynamic session tick-frame anchors are missing.")
    if missing_time_handoff_anchors:
        risks.append("Runtime 03 RuntimeTimeAdvance handoff anchors are missing.")
    if missing_fixed_plan_anchors:
        risks.append("Runtime 03 FixedStepPlan interpolation anchors are missing.")
    if missing_ui_extract_anchors:
        risks.append("Runtime 03 documented UI extract side-path anchors are missing.")
    if ui_extract_render_stage_references:
        risks.append("Runtime 03 UI extract side path now references RenderExtract stage.")
    if missing_schedule_order_anchors:
        risks.append("Runtime 03 explicit stage-ordering anchors are missing.")
    if missing_schedule_runner_anchors:
        risks.append("Runtime 03 schedule runner stage/deferred/hook anchors are missing.")
    if missing_parallel_executor_anchors:
        risks.append("Runtime 03 parallel executor observability anchors are missing.")
    if missing_test_anchors:
        risks.append("Runtime 03 guard/test anchors are missing.")
    if missing_behavior_test_anchors:
        risks.append("Runtime 03 behavior test anchors are missing.")
    if not mirror_docs_guard_present:
        risks.append("Runtime 03 schedule/frame-loop mirror-doc aggregate guard is missing.")
    if missing_doc_anchors:
        risks.append("Runtime 03 plan or mirror docs are missing required status anchors.")
    if missing_frame_schedule_doc_anchors:
        risks.append("Runtime 03 frame schedule module doc is missing audit mirror anchors.")
    if missing_cargo_gate_anchors:
        risks.append("Runtime 03 pending Cargo gate anchors are missing from docs.")
    if world_driver_second_advance_references:
        risks.append("Runtime 03 WorldDriver reintroduced a second time advance path.")
    if session_raw_delta_tick_references:
        risks.append("Runtime 03 dynamic session reduced RuntimeTimeAdvance to raw delta.")

    return {
        "source_files": source_files,
        "expected_source_file_count": len(RUNTIME_03_SOURCE_FILES),
        "missing_source_files": missing_source_files,
        "guard_files": guard_files,
        "expected_guard_file_count": len(RUNTIME_03_GUARD_FILES),
        "missing_guard_files": missing_guard_files,
        "stage_count": stage_count,
        "declared_stage_variant_count": declared_stage_variant_count,
        "expected_stage_count": EXPECTED_STAGE_COUNT,
        "fixed_loop_stage_count": fixed_loop_stage_count,
        "expected_fixed_loop_stage_count": EXPECTED_FIXED_LOOP_STAGE_COUNT,
        "dynamic_session_tick_time_call_count": dynamic_session_tick_time_call_count,
        "expected_dynamic_session_tick_time_call_count": EXPECTED_DYNAMIC_SESSION_TICK_TIME_CALLS,
        "missing_stage_variants": missing_stage_variants,
        "missing_system_stage_anchors": missing_system_stage_anchors,
        "missing_session_tick_anchors": missing_session_tick_anchors,
        "missing_time_handoff_anchors": missing_time_handoff_anchors,
        "missing_fixed_plan_anchors": missing_fixed_plan_anchors,
        "missing_ui_extract_anchors": missing_ui_extract_anchors,
        "ui_extract_render_stage_references": ui_extract_render_stage_references,
        "missing_schedule_order_anchors": missing_schedule_order_anchors,
        "missing_schedule_runner_anchors": missing_schedule_runner_anchors,
        "missing_parallel_executor_anchors": missing_parallel_executor_anchors,
        "missing_test_anchors": missing_test_anchors,
        "test_anchor_count": len(RUNTIME_03_TEST_ANCHORS),
        "behavior_test_anchor_count": len(RUNTIME_03_BEHAVIOR_TEST_ANCHORS),
        "missing_behavior_test_anchors": missing_behavior_test_anchors,
        "mirror_docs_guard": MIRROR_DOCS_GUARD,
        "mirror_docs_guard_present": mirror_docs_guard_present,
        "missing_doc_anchors": missing_doc_anchors,
        "doc_anchor_count": len(RUNTIME_03_DOC_ANCHORS),
        "frame_schedule_doc_anchor_count": len(FRAME_SCHEDULE_DOC_ANCHORS),
        "missing_frame_schedule_doc_anchors": missing_frame_schedule_doc_anchors,
        "missing_cargo_gate_anchors": missing_cargo_gate_anchors,
        "world_driver_second_advance_references": world_driver_second_advance_references,
        "session_raw_delta_tick_references": session_raw_delta_tick_references,
        "risks": risks,
    }
