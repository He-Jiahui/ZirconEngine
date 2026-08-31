from __future__ import annotations

from pathlib import Path

from .job_system_anchor_inventory import (
    JOB_SYSTEM_API_SNIPPETS,
    JOB_SYSTEM_BEHAVIOR_TEST_ANCHORS,
    JOB_SYSTEM_FORBIDDEN_GRAPHICS_OWNER_SNIPPETS,
    JOB_SYSTEM_FORBIDDEN_NAVIGATION_MANAGER_SNIPPETS,
    JOB_SYSTEM_FORBIDDEN_NAVIGATION_OWNER_SNIPPETS,
    JOB_SYSTEM_FORBIDDEN_PLATFORM_ADAPTER_OWNER_SNIPPETS,
    JOB_SYSTEM_FORBIDDEN_PLATFORM_DEFAULT_CALL,
    JOB_SYSTEM_FORBIDDEN_PLATFORM_OWNER_SNIPPETS,
    JOB_SYSTEM_FORBIDDEN_SCHEDULER_OWNER_SNIPPETS,
    JOB_SYSTEM_REQUIRED_NAVIGATION_MANAGER_SNIPPETS,
    JOB_SYSTEM_REQUIRED_NAVIGATION_MODULE_SNIPPETS,
    JOB_SYSTEM_REQUIRED_PLATFORM_ADAPTER_SNIPPETS,
    JOB_SYSTEM_REQUIRED_PLATFORM_ADAPTER_TEST_SNIPPETS,
    JOB_SYSTEM_REQUIRED_PLATFORM_APP_SNIPPETS,
    JOB_SYSTEM_REQUIRED_PLATFORM_DRIVER_SNIPPETS,
    JOB_SYSTEM_REQUIRED_PLATFORM_MODULE_SNIPPETS,
    JOB_SYSTEM_REQUIRED_PLATFORM_TEST_SUPPORT_SNIPPETS,
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
    job_handle_tests = tasks_dir / "job_handle" / "tests.rs"
    job_scheduler_tests = tasks_dir / "job_scheduler" / "tests.rs"
    job_scheduler = tasks_dir / "job_scheduler.rs"
    graphics_framework_construction = (
        root
        / "zircon_runtime"
        / "src"
        / "graphics"
        / "runtime"
        / "render_framework"
        / "wgpu_render_framework_construction"
        / "construct.rs"
    )
    navigation_runtime_dir = root / "zircon_plugins" / "navigation" / "runtime" / "src"
    navigation_manager = navigation_runtime_dir / "manager.rs"
    navigation_module = navigation_runtime_dir / "lib.rs"
    runtime_source_dir = root / "zircon_runtime" / "src"
    app_source_dir = root / "zircon_app" / "src"
    platform_dir = runtime_source_dir / "platform"
    platform_driver = platform_dir / "service_types" / "driver.rs"
    platform_adapter = platform_dir / "preferences" / "persistence" / "adapter.rs"
    platform_adapter_tests = platform_dir / "preferences" / "persistence" / "tests.rs"
    platform_module = platform_dir / "module.rs"
    platform_test_support = platform_dir / "test_support.rs"
    app_engine_entry = root / "zircon_app" / "src" / "entry" / "engine_entry.rs"
    pool_tests = tasks_dir / "pool" / "tests.rs"
    diagnostics_tests = tasks_dir / "diagnostics" / "tests.rs"
    diagnostic_observation_tests = tasks_dir / "diagnostic_observation" / "tests.rs"
    bounded_stream_io_tests = tasks_dir / "bounded_stream_io" / "tests.rs"
    retained_byte_budget_tests = tasks_dir / "retained_byte_budget" / "tests.rs"
    runtime_manifest = root / "zircon_runtime" / "Cargo.toml"
    task_graph_engine_tests = tasks_dir / "task_graph" / "engine_task_graph.rs"
    task_graph_scope_tests = tasks_dir / "task_graph" / "scope" / "tests.rs"
    dynamic_scene_spawn_tests = (
        root
        / "zircon_runtime"
        / "src"
        / "scene"
        / "dynamic_scene"
        / "spawn_task"
        / "loader.rs"
    )
    level_manager_project_io_tests = (
        root
        / "zircon_runtime"
        / "src"
        / "scene"
        / "module"
        / "level_manager_project_io.rs"
    )
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
    job_handle_tests_source = (
        _read_text(job_handle_tests) if job_handle_tests.exists() else ""
    )
    job_scheduler_tests_source = (
        _read_text(job_scheduler_tests) if job_scheduler_tests.exists() else ""
    )
    job_scheduler_source = _read_text(job_scheduler) if job_scheduler.exists() else ""
    graphics_framework_construction_source = (
        _read_text(graphics_framework_construction)
        if graphics_framework_construction.exists()
        else ""
    )
    navigation_manager_source = (
        _read_text(navigation_manager) if navigation_manager.exists() else ""
    )
    navigation_module_source = (
        _read_text(navigation_module) if navigation_module.exists() else ""
    )
    navigation_production_source = "\n".join(
        _read_text(path)
        for path in navigation_runtime_dir.rglob("*.rs")
        if path.name not in {"test_support.rs", "tests.rs"}
        and "tests" not in path.relative_to(navigation_runtime_dir).parts
    )
    platform_driver_source = _read_text(platform_driver) if platform_driver.exists() else ""
    platform_adapter_source = _read_text(platform_adapter) if platform_adapter.exists() else ""
    platform_adapter_tests_source = (
        _read_text(platform_adapter_tests) if platform_adapter_tests.exists() else ""
    )
    platform_module_source = _read_text(platform_module) if platform_module.exists() else ""
    platform_test_support_source = (
        _read_text(platform_test_support) if platform_test_support.exists() else ""
    )
    app_engine_entry_source = _read_text(app_engine_entry) if app_engine_entry.exists() else ""
    pool_tests_source = _read_text(pool_tests) if pool_tests.exists() else ""
    diagnostics_tests_source = (
        _read_text(diagnostics_tests) if diagnostics_tests.exists() else ""
    )
    diagnostic_observation_tests_source = (
        _read_text(diagnostic_observation_tests)
        if diagnostic_observation_tests.exists()
        else ""
    )
    bounded_stream_io_tests_source = (
        _read_text(bounded_stream_io_tests) if bounded_stream_io_tests.exists() else ""
    )
    retained_byte_budget_tests_source = (
        _read_text(retained_byte_budget_tests)
        if retained_byte_budget_tests.exists()
        else ""
    )
    runtime_editor_dependency_references = []
    for path in (runtime_manifest, *tasks_dir.rglob("*.rs")):
        if path.exists() and "zircon_editor" in _read_text(path):
            runtime_editor_dependency_references.append(_relative(root, path))
    task_graph_scope_tests_source = (
        _read_text(task_graph_scope_tests) if task_graph_scope_tests.exists() else ""
    )
    task_graph_engine_tests_source = (
        _read_text(task_graph_engine_tests) if task_graph_engine_tests.exists() else ""
    )
    dynamic_scene_spawn_tests_source = (
        _read_text(dynamic_scene_spawn_tests)
        if dynamic_scene_spawn_tests.exists()
        else ""
    )
    level_manager_project_io_tests_source = (
        _read_text(level_manager_project_io_tests)
        if level_manager_project_io_tests.exists()
        else ""
    )
    behavior_test_sources = (
        bounded_stream_io_tests_source,
        retained_byte_budget_tests_source,
        tasks_tests_source,
        job_handle_tests_source,
        job_scheduler_tests_source,
        pool_tests_source,
        diagnostics_tests_source,
        diagnostic_observation_tests_source,
        task_graph_engine_tests_source,
        task_graph_scope_tests_source,
        dynamic_scene_spawn_tests_source,
        level_manager_project_io_tests_source,
    )

    owner_modules: list[dict[str, object]] = []
    missing_modules: list[str] = []
    oversized_modules: list[dict[str, object]] = []
    actual_modules = sorted(path.name for path in tasks_dir.glob("*.rs")) if tasks_dir.exists() else []
    unexpected_modules = [
        module for module in actual_modules if module not in JOB_SYSTEM_MODULES
    ]

    for module in JOB_SYSTEM_MODULES:
        path = tasks_dir / module
        if not path.exists():
            missing_modules.append(
                f"zircon_runtime/src/core/runtime/tasks/{module}"
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
    forbidden_scheduler_owner_snippets = [
        snippet
        for snippet in JOB_SYSTEM_FORBIDDEN_SCHEDULER_OWNER_SNIPPETS
        if snippet in job_scheduler_source
    ]
    forbidden_graphics_owner_snippets = [
        snippet
        for snippet in JOB_SYSTEM_FORBIDDEN_GRAPHICS_OWNER_SNIPPETS
        if snippet in graphics_framework_construction_source
    ]
    forbidden_navigation_owner_snippets = [
        snippet
        for snippet in JOB_SYSTEM_FORBIDDEN_NAVIGATION_OWNER_SNIPPETS
        if snippet in navigation_production_source
    ]
    forbidden_navigation_owner_snippets.extend(
        snippet
        for snippet in JOB_SYSTEM_FORBIDDEN_NAVIGATION_MANAGER_SNIPPETS
        if snippet in navigation_manager_source
    )
    missing_navigation_owner_snippets = {
        "zircon_plugins/navigation/runtime/src/manager.rs": [
            snippet
            for snippet in JOB_SYSTEM_REQUIRED_NAVIGATION_MANAGER_SNIPPETS
            if snippet not in navigation_manager_source
        ],
        "zircon_plugins/navigation/runtime/src/lib.rs": [
            snippet
            for snippet in JOB_SYSTEM_REQUIRED_NAVIGATION_MODULE_SNIPPETS
            if snippet not in navigation_module_source
        ],
    }
    missing_navigation_owner_snippets = {
        path: missing
        for path, missing in missing_navigation_owner_snippets.items()
        if missing
    }
    forbidden_platform_owner_snippets = [
        snippet
        for snippet in JOB_SYSTEM_FORBIDDEN_PLATFORM_OWNER_SNIPPETS
        if snippet in platform_driver_source
    ]
    forbidden_platform_adapter_owner_snippets = [
        snippet
        for snippet in JOB_SYSTEM_FORBIDDEN_PLATFORM_ADAPTER_OWNER_SNIPPETS
        if snippet in platform_adapter_source
    ]
    platform_default_scan_dirs = (platform_dir, app_source_dir / "entry")
    platform_default_constructor_references = sorted(
        _relative(root, path)
        for source_dir in platform_default_scan_dirs
        for path in source_dir.rglob("*.rs")
        if JOB_SYSTEM_FORBIDDEN_PLATFORM_DEFAULT_CALL in _read_text(path)
    )
    missing_platform_owner_snippets = {
        "zircon_runtime/src/platform/service_types/driver.rs": [
            snippet
            for snippet in JOB_SYSTEM_REQUIRED_PLATFORM_DRIVER_SNIPPETS
            if snippet not in platform_driver_source
        ],
        "zircon_runtime/src/platform/preferences/persistence/adapter.rs": [
            snippet
            for snippet in JOB_SYSTEM_REQUIRED_PLATFORM_ADAPTER_SNIPPETS
            if snippet not in platform_adapter_source
        ],
        "zircon_runtime/src/platform/preferences/persistence/tests.rs": [
            snippet
            for snippet in JOB_SYSTEM_REQUIRED_PLATFORM_ADAPTER_TEST_SNIPPETS
            if snippet not in platform_adapter_tests_source
        ],
        "zircon_runtime/src/platform/module.rs": [
            snippet
            for snippet in JOB_SYSTEM_REQUIRED_PLATFORM_MODULE_SNIPPETS
            if snippet not in platform_module_source
        ],
        "zircon_runtime/src/platform/test_support.rs": [
            snippet
            for snippet in JOB_SYSTEM_REQUIRED_PLATFORM_TEST_SUPPORT_SNIPPETS
            if snippet not in platform_test_support_source
        ],
        "zircon_app/src/entry/engine_entry.rs": [
            snippet
            for snippet in JOB_SYSTEM_REQUIRED_PLATFORM_APP_SNIPPETS
            if snippet not in app_engine_entry_source
        ],
    }
    missing_platform_owner_snippets = {
        path: missing
        for path, missing in missing_platform_owner_snippets.items()
        if missing
    }
    missing_schedule_executor_snippets = [
        snippet
        for snippet in SCHEDULE_EXECUTOR_REQUIRED_SNIPPETS
        if snippet not in schedule_executor_source
    ]
    missing_behavior_test_anchors = [
        anchor
        for anchor in JOB_SYSTEM_BEHAVIOR_TEST_ANCHORS
        if not any(anchor in source for source in behavior_test_sources)
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
    if forbidden_scheduler_owner_snippets:
        risks.append(
            "JobScheduler must receive an explicit task owner and cannot expose implicit process or private-pool constructors."
        )
    if forbidden_graphics_owner_snippets:
        risks.append(
            "WgpuRenderFramework constructors must receive the Runtime-owned task pool and cannot create or expose a private worker owner."
        )
    if forbidden_navigation_owner_snippets or missing_navigation_owner_snippets:
        risks.append(
            "Navigation bake work must consume the Runtime-owned task pool through an explicit manager constructor and TasksModule dependency."
        )
    if (
        forbidden_platform_owner_snippets
        or forbidden_platform_adapter_owner_snippets
        or platform_default_constructor_references
        or missing_platform_owner_snippets
    ):
        risks.append(
            "Platform preference persistence must consume an explicitly injected Runtime task pool in both builtin and app-overridden driver factories."
        )
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
    if runtime_editor_dependency_references:
        risks.append(
            "Runtime JobSystem sources or manifest must not depend on zircon_editor."
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
        "forbidden_scheduler_owner_snippets": forbidden_scheduler_owner_snippets,
        "forbidden_graphics_owner_snippets": forbidden_graphics_owner_snippets,
        "forbidden_navigation_owner_snippets": forbidden_navigation_owner_snippets,
        "missing_navigation_owner_snippets": missing_navigation_owner_snippets,
        "forbidden_platform_owner_snippets": forbidden_platform_owner_snippets,
        "forbidden_platform_adapter_owner_snippets": forbidden_platform_adapter_owner_snippets,
        "platform_default_constructor_references": platform_default_constructor_references,
        "missing_platform_owner_snippets": missing_platform_owner_snippets,
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
        "runtime_editor_dependency_references": runtime_editor_dependency_references,
        "mirror_docs_guard": MIRROR_DOCS_GUARD,
        "mirror_docs_guard_present": MIRROR_DOCS_GUARD in job_system_guard_source,
        "risks": risks,
    }
