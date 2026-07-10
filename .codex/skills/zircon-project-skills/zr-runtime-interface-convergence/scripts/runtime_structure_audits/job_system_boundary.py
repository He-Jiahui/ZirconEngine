from __future__ import annotations

from pathlib import Path

from .job_system_anchor_inventory import (
    JOB_SYSTEM_API_SNIPPETS,
    JOB_SYSTEM_BEHAVIOR_TEST_ANCHORS,
    JOB_SYSTEM_REQUIRED_DECLARATIONS,
    JOB_SYSTEM_REQUIRED_PUBLIC_SURFACE,
    MIRROR_DOCS_GUARD,
    SCHEDULE_EXECUTOR_REQUIRED_SNIPPETS,
)
from .job_system_source_inventory import (
    EXPECTED_JOB_SYSTEM_GUARD_FILE_COUNT,
    JOB_SYSTEM_GUARD_FILES,
    JOB_SYSTEM_MODULE_MAX_LINES,
    JOB_SYSTEM_MODULES,
    RAYON_CLASSIFICATIONS,
    collect_direct_rayon_references,
)


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _relative(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def _file_line_count(path: Path) -> int:
    return len(_read_text(path).splitlines())


def _missing_snippets(path: Path, snippets: tuple[str, ...]) -> list[str]:
    source = _read_text(path) if path.exists() else ""
    return [snippet for snippet in snippets if snippet not in source]


def job_system_boundary_audit(root: Path) -> dict[str, object]:
    tasks_dir = root / "zircon_runtime" / "src" / "core" / "runtime" / "tasks"
    mod_file = tasks_dir / "mod.rs"
    mod_source = _read_text(mod_file) if mod_file.exists() else ""
    schedule_executor = (
        root / "zircon_runtime" / "src" / "scene" / "ecs" / "schedule_parallel_executor.rs"
    )
    job_system_guard_paths = tuple(root / path for path in JOB_SYSTEM_GUARD_FILES)
    tasks_tests = root / "zircon_runtime" / "src" / "tests" / "tasks.rs"
    schedule_executor_source = (
        _read_text(schedule_executor) if schedule_executor.exists() else ""
    )
    missing_guard_files = [
        path.relative_to(root).as_posix()
        for path in job_system_guard_paths
        if not path.exists()
    ]
    job_system_guard_source = "\n".join(
        _read_text(path) for path in job_system_guard_paths if path.exists()
    )
    tasks_tests_source = _read_text(tasks_tests) if tasks_tests.exists() else ""

    owner_modules: list[dict[str, object]] = []
    missing_modules: list[str] = []
    oversized_modules: list[dict[str, object]] = []
    actual_modules = sorted(path.stem for path in tasks_dir.glob("*.rs")) if tasks_dir.exists() else []
    unexpected_modules = [
        module for module in actual_modules if module not in JOB_SYSTEM_MODULES
    ]

    for module in JOB_SYSTEM_MODULES:
        path = tasks_dir / f"{module}.rs"
        if not path.exists():
            missing_modules.append(
                f"zircon_runtime/src/core/runtime/tasks/{module}.rs"
            )
            continue

        module_entry = {
            "path": _relative(root, path),
            "lines": _file_line_count(path),
        }
        owner_modules.append(module_entry)
        if module_entry["lines"] > JOB_SYSTEM_MODULE_MAX_LINES:
            oversized_modules.append(module_entry)

    missing_mod_declarations = [
        declaration
        for declaration in JOB_SYSTEM_REQUIRED_DECLARATIONS
        if declaration not in mod_source
    ]
    missing_public_surface = [
        name
        for name, snippet in JOB_SYSTEM_REQUIRED_PUBLIC_SURFACE.items()
        if snippet not in mod_source
    ]
    missing_api_snippets = {
        file_name: missing
        for file_name, snippets in JOB_SYSTEM_API_SNIPPETS.items()
        if (missing := _missing_snippets(tasks_dir / file_name, snippets))
    }
    missing_schedule_executor_snippets = [
        snippet
        for snippet in SCHEDULE_EXECUTOR_REQUIRED_SNIPPETS
        if snippet not in schedule_executor_source
    ]
    missing_behavior_test_anchors = [
        anchor
        for anchor in JOB_SYSTEM_BEHAVIOR_TEST_ANCHORS
        if anchor not in tasks_tests_source
    ]

    direct_rayon_references = collect_direct_rayon_references(root)
    unclassified_direct_rayon = [
        reference
        for reference in direct_rayon_references
        if reference["classification"] is None
    ]
    direct_rayon_paths = sorted(
        {reference["path"] for reference in direct_rayon_references}
    )
    expected_rayon_paths = sorted(RAYON_CLASSIFICATIONS)
    missing_expected_rayon_paths = [
        path for path in expected_rayon_paths if path not in direct_rayon_paths
    ]
    unexpected_rayon_paths = [
        path for path in direct_rayon_paths if path not in RAYON_CLASSIFICATIONS
    ]
    schedule_executor_direct_rayon = [
        reference
        for reference in direct_rayon_references
        if reference["path"]
        == "zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs"
    ]

    risks: list[str] = []
    if not tasks_dir.is_dir():
        risks.append("zircon_runtime/src/core/runtime/tasks/ is missing.")
    if missing_modules:
        risks.append("JobSystem owner modules are missing from the task owner folder.")
    if unexpected_modules:
        risks.append("JobSystem task folder has unexpected modules; update the boundary intentionally.")
    if missing_mod_declarations:
        risks.append("core/runtime/tasks/mod.rs is missing one or more owner module declarations.")
    if missing_public_surface:
        risks.append("core/runtime/tasks/mod.rs is missing required JobSystem public surface exports.")
    if missing_api_snippets:
        risks.append("JobSystem owner modules are missing one or more API or diagnostics anchors.")
    if not schedule_executor.exists():
        risks.append("zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs is missing.")
    if missing_schedule_executor_snippets:
        risks.append("ScheduleParallelExecutor is missing dependency-chain or scheduler-join anchors.")
    if not tasks_tests.exists():
        risks.append("zircon_runtime/src/tests/tasks.rs is missing.")
    if missing_behavior_test_anchors:
        risks.append("JobSystem behavior tests are missing one or more M1/M3 anchors.")
    if missing_guard_files:
        risks.append("Runtime 11 JobSystem guard/test owner files are missing.")
    if unclassified_direct_rayon:
        risks.append("Production direct-Rayon usage exists outside the Runtime 11 whitelist.")
    if missing_expected_rayon_paths:
        risks.append("Expected Runtime 11 direct-Rayon owner paths are absent; update the boundary intentionally.")
    if unexpected_rayon_paths:
        risks.append("Direct-Rayon paths differ from the Runtime 11 whitelist.")
    if schedule_executor_direct_rayon:
        risks.append("ScheduleParallelExecutor should not call Rayon directly after Runtime 11 M2.2.")
    if oversized_modules:
        risks.append(
            f"one or more JobSystem owner modules exceed {JOB_SYSTEM_MODULE_MAX_LINES} lines; split by execution owner before adding more behavior."
        )
    if MIRROR_DOCS_GUARD not in job_system_guard_source:
        risks.append("Runtime 11 JobSystem mirror-doc aggregate guard is missing.")

    return {
        "tasks_dir_exists": tasks_dir.is_dir(),
        "owner_modules": owner_modules,
        "expected_module_count": len(JOB_SYSTEM_MODULES),
        "missing_modules": missing_modules,
        "unexpected_modules": unexpected_modules,
        "missing_mod_declarations": missing_mod_declarations,
        "missing_public_surface": missing_public_surface,
        "missing_api_snippets": missing_api_snippets,
        "schedule_parallel_executor_exists": schedule_executor.exists(),
        "missing_schedule_executor_snippets": missing_schedule_executor_snippets,
        "behavior_test_anchor_count": len(JOB_SYSTEM_BEHAVIOR_TEST_ANCHORS),
        "missing_behavior_test_anchors": missing_behavior_test_anchors,
        "expected_guard_file_count": EXPECTED_JOB_SYSTEM_GUARD_FILE_COUNT,
        "missing_guard_files": missing_guard_files,
        "schedule_parallel_executor_uses_schedule_after": (
            ".schedule_after(" in schedule_executor_source
        ),
        "schedule_parallel_executor_direct_rayon": schedule_executor_direct_rayon,
        "direct_rayon_references": direct_rayon_references,
        "direct_rayon_paths": direct_rayon_paths,
        "expected_direct_rayon_paths": expected_rayon_paths,
        "missing_expected_rayon_paths": missing_expected_rayon_paths,
        "unexpected_rayon_paths": unexpected_rayon_paths,
        "unclassified_direct_rayon": unclassified_direct_rayon,
        "diagnostic_anchor_count": len(JOB_SYSTEM_API_SNIPPETS["diagnostics.rs"]),
        "max_module_lines": JOB_SYSTEM_MODULE_MAX_LINES,
        "oversized_modules": oversized_modules,
        "mirror_docs_guard": MIRROR_DOCS_GUARD,
        "mirror_docs_guard_present": MIRROR_DOCS_GUARD in job_system_guard_source,
        "risks": risks,
    }
