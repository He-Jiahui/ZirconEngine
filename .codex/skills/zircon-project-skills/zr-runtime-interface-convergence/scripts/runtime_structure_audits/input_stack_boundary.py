from __future__ import annotations

from pathlib import Path

from runtime_structure_audits.input_stack_anchor_inventory import (
    ACTION_EVALUATOR_ANCHORS,
    CARGO_GATE_ANCHORS,
    CURSOR_HOST_REQUEST_ANCHORS,
    FRAMEWORK_MOD_DECLARATIONS,
    GAMEPAD_ABI_ANCHORS,
    INPUT_MOD_DECLARATIONS,
    INPUT_TEST_DECLARATIONS,
    MIRROR_DOCS_GUARD,
    PUBLIC_SURFACE_ANCHORS,
    RUNTIME_12_BEHAVIOR_TEST_ANCHORS,
    RUNTIME_12_DOC_ANCHORS,
    RUNTIME_12_GUARDS,
    RUNTIME_12_TEST_ANCHORS,
    RUNTIME_MOD_DECLARATIONS,
)
from runtime_structure_audits.input_stack_source_inventory import (
    EXPECTED_INPUT_STACK_GUARD_FILE_COUNT,
    FRAMEWORK_INPUT_MODULES,
    INPUT_STACK_GUARD_FILES,
    INPUT_MANAGER_CHILD_TEST_MODULES,
    INPUT_PRODUCTION_MODULE_MAX_LINES,
    INPUT_RUNTIME_MODULES,
    INPUT_TEST_MODULE_MAX_LINES,
    INPUT_TEST_MODULES,
)


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _relative(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def _file_line_count(path: Path) -> int:
    return len(_read_text(path).splitlines())


def _module_entries(
    root: Path,
    modules: tuple[str, ...],
    max_lines: int,
) -> tuple[list[dict[str, object]], list[str], list[dict[str, object]]]:
    entries: list[dict[str, object]] = []
    missing: list[str] = []
    oversized: list[dict[str, object]] = []

    for module in modules:
        path = root / module
        if not path.exists():
            missing.append(module)
            continue

        entry = {"path": module, "lines": _file_line_count(path)}
        entries.append(entry)
        if entry["lines"] > max_lines:
            oversized.append(entry)

    return entries, missing, oversized


def _actual_modules(root: Path, folder: str, exclude_tests: bool = False) -> list[str]:
    source_root = root / folder
    if not source_root.is_dir():
        return []

    modules: list[str] = []
    for path in source_root.rglob("*.rs"):
        if exclude_tests and "/tests/" in _relative(root, path):
            continue
        modules.append(_relative(root, path))
    return sorted(modules)


def _missing_snippets(sources: tuple[str, ...], snippets: tuple[str, ...]) -> list[str]:
    return [
        snippet
        for snippet in snippets
        if not any(snippet in source for source in sources)
    ]


def input_stack_boundary_audit(root: Path) -> dict[str, object]:
    input_mod = root / "zircon_runtime/src/input/mod.rs"
    runtime_mod = root / "zircon_runtime/src/input/runtime/mod.rs"
    framework_mod = root / "zircon_runtime/src/core/framework/input/mod.rs"
    tests_mod = root / "zircon_runtime/src/input/tests/mod.rs"
    prelude = root / "zircon_runtime/src/prelude.rs"
    action_evaluator = root / "zircon_runtime/src/input/runtime/action_evaluator.rs"
    input_stack_guard_paths = tuple(root / path for path in INPUT_STACK_GUARD_FILES)
    runtime_12_plan = (
        root
        / "docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md"
    )
    runtime_index = root / "docs/plans/zircon_runtime/runtime/index.md"
    input_doc = root / "docs/zircon_runtime/input/input_state.md"
    app_gamepad_events = root / "zircon_app/src/entry/runtime_entry_app/gamepad/events.rs"
    app_gamepad_polling = root / "zircon_app/src/entry/runtime_entry_app/gamepad/polling.rs"
    dynamic_session = root / "zircon_runtime/src/dynamic_api/session.rs"
    dynamic_session_events = root / "zircon_runtime/src/dynamic_api/session/events.rs"
    input_event = root / "zircon_runtime/src/core/framework/input/input_event.rs"
    default_input_manager = root / "zircon_runtime/src/input/runtime/default_input_manager.rs"
    dynamic_session_host_requests = (
        root / "zircon_runtime/src/dynamic_api/session/host_requests.rs"
    )
    runtime_interface_host_requests = (
        root / "zircon_runtime_interface/src/runtime_api/host_requests.rs"
    )
    app_host_request_routing = (
        root / "zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs"
    )
    app_cursor_request = (
        root / "zircon_app/src/entry/runtime_entry_app/host_requests/cursor/request.rs"
    )
    platform_backend_tokens = root / "zircon_runtime/src/platform/tests/backend_tokens.rs"
    platform_diagnostics = root / "zircon_runtime/src/platform/tests/diagnostics.rs"

    input_mod_source = _read_text(input_mod) if input_mod.exists() else ""
    runtime_mod_source = _read_text(runtime_mod) if runtime_mod.exists() else ""
    framework_mod_source = _read_text(framework_mod) if framework_mod.exists() else ""
    tests_mod_source = _read_text(tests_mod) if tests_mod.exists() else ""
    prelude_source = _read_text(prelude) if prelude.exists() else ""
    action_evaluator_source = (
        _read_text(action_evaluator) if action_evaluator.exists() else ""
    )
    missing_guard_files = [
        path.relative_to(root).as_posix()
        for path in input_stack_guard_paths
        if not path.exists()
    ]
    input_stack_guard_source = "\n".join(
        _read_text(path) for path in input_stack_guard_paths if path.exists()
    )
    doc_sources = tuple(
        _read_text(path)
        for path in (runtime_12_plan, runtime_index, input_doc)
        if path.exists()
    )
    test_sources = tuple(
        _read_text(root / module)
        for module in (*INPUT_TEST_MODULES, *INPUT_MANAGER_CHILD_TEST_MODULES)
        if (root / module).exists()
    )
    gamepad_sources = tuple(
        _read_text(path)
        for path in (
            app_gamepad_events,
            app_gamepad_polling,
            dynamic_session,
            dynamic_session_events,
        )
        if path.exists()
    )
    cursor_host_request_sources = tuple(
        _read_text(path)
        for path in (
            framework_mod,
            input_event,
            default_input_manager,
            dynamic_session,
            dynamic_session_host_requests,
            runtime_interface_host_requests,
            app_host_request_routing,
            app_cursor_request,
            platform_backend_tokens,
            platform_diagnostics,
        )
        if path.exists()
    )

    runtime_modules, missing_runtime_modules, oversized_runtime_modules = _module_entries(
        root,
        INPUT_RUNTIME_MODULES,
        INPUT_PRODUCTION_MODULE_MAX_LINES,
    )
    framework_modules, missing_framework_modules, oversized_framework_modules = (
        _module_entries(
            root,
            FRAMEWORK_INPUT_MODULES,
            INPUT_PRODUCTION_MODULE_MAX_LINES,
        )
    )
    test_modules, missing_test_modules, oversized_test_modules = _module_entries(
        root,
        INPUT_TEST_MODULES,
        INPUT_TEST_MODULE_MAX_LINES,
    )

    actual_runtime_modules = _actual_modules(
        root,
        "zircon_runtime/src/input",
        exclude_tests=True,
    )
    unexpected_runtime_modules = [
        module for module in actual_runtime_modules if module not in INPUT_RUNTIME_MODULES
    ]
    actual_framework_modules = _actual_modules(
        root,
        "zircon_runtime/src/core/framework/input",
    )
    unexpected_framework_modules = [
        module for module in actual_framework_modules if module not in FRAMEWORK_INPUT_MODULES
    ]

    missing_input_mod_declarations = _missing_snippets(
        (input_mod_source,),
        INPUT_MOD_DECLARATIONS,
    )
    missing_runtime_mod_declarations = _missing_snippets(
        (runtime_mod_source,),
        RUNTIME_MOD_DECLARATIONS,
    )
    missing_framework_mod_declarations = _missing_snippets(
        (framework_mod_source,),
        FRAMEWORK_MOD_DECLARATIONS,
    )
    missing_test_mod_declarations = _missing_snippets(
        (tests_mod_source,),
        INPUT_TEST_DECLARATIONS,
    )
    missing_public_surface = _missing_snippets(
        (input_mod_source, framework_mod_source, prelude_source),
        PUBLIC_SURFACE_ANCHORS,
    )
    missing_action_evaluator_anchors = _missing_snippets(
        (action_evaluator_source,),
        ACTION_EVALUATOR_ANCHORS,
    )
    missing_gamepad_abi_anchors = _missing_snippets(
        gamepad_sources,
        GAMEPAD_ABI_ANCHORS,
    )
    missing_cursor_host_request_anchors = _missing_snippets(
        cursor_host_request_sources,
        CURSOR_HOST_REQUEST_ANCHORS,
    )
    missing_runtime_12_guards = _missing_snippets(
        (input_stack_guard_source,),
        RUNTIME_12_GUARDS,
    )
    missing_doc_anchors = _missing_snippets(doc_sources, RUNTIME_12_DOC_ANCHORS)
    missing_test_anchors = _missing_snippets(test_sources, RUNTIME_12_TEST_ANCHORS)
    missing_behavior_test_anchors = _missing_snippets(
        test_sources, RUNTIME_12_BEHAVIOR_TEST_ANCHORS
    )
    missing_cargo_gate_anchors = _missing_snippets(doc_sources, CARGO_GATE_ANCHORS)

    oversized_modules = (
        oversized_runtime_modules
        + oversized_framework_modules
        + oversized_test_modules
    )

    risks: list[str] = []
    if missing_runtime_modules:
        risks.append("Runtime 12 input runtime owner modules are missing.")
    if missing_framework_modules:
        risks.append("Runtime 12 framework input contract modules are missing.")
    if missing_test_modules:
        risks.append("Runtime 12 input test owner modules are missing.")
    if unexpected_runtime_modules:
        risks.append("Runtime 12 input runtime module set changed without audit sync.")
    if unexpected_framework_modules:
        risks.append("Runtime 12 framework input contract module set changed without audit sync.")
    if missing_input_mod_declarations:
        risks.append("zircon_runtime/src/input/mod.rs is missing expected declarations or exports.")
    if missing_runtime_mod_declarations:
        risks.append("input/runtime/mod.rs is missing expected owner module declarations.")
    if missing_framework_mod_declarations:
        risks.append("core/framework/input/mod.rs is missing expected contract module declarations.")
    if missing_test_mod_declarations:
        risks.append("input/tests/mod.rs is missing expected owner test declarations.")
    if missing_public_surface:
        risks.append("Runtime 12 input public surface anchors are missing from exports.")
    if missing_action_evaluator_anchors:
        risks.append("Runtime 12 action evaluator lost required UI-filtered evaluation anchors.")
    if missing_gamepad_abi_anchors:
        risks.append("Runtime 12 gamepad app ABI to dynamic-session path anchors are missing.")
    if missing_cursor_host_request_anchors:
        risks.append("Runtime 12 cursor host-request ABI/app/platform path anchors are missing.")
    if missing_runtime_12_guards:
        risks.append("Runtime 12 Rust guard anchors are missing.")
    if missing_guard_files:
        risks.append("Runtime 12 input-stack guard/test owner files are missing.")
    if MIRROR_DOCS_GUARD not in input_stack_guard_source:
        risks.append("Runtime 12 input-stack mirror-doc aggregate guard is missing.")
    if missing_doc_anchors:
        risks.append("Runtime 12 plan or mirror docs are missing required status anchors.")
    if missing_test_anchors:
        risks.append("Runtime 12 named input/action/gamepad test anchors are missing.")
    if missing_behavior_test_anchors:
        risks.append("Runtime 12 behavior test anchors are missing.")
    if missing_cargo_gate_anchors:
        risks.append("Runtime 12 pending Cargo gate anchors are missing from docs.")
    if oversized_modules:
        risks.append("Runtime 12 input owner module exceeds its line budget.")

    return {
        "runtime_modules": runtime_modules,
        "expected_runtime_module_count": len(INPUT_RUNTIME_MODULES),
        "missing_runtime_modules": missing_runtime_modules,
        "unexpected_runtime_modules": unexpected_runtime_modules,
        "framework_modules": framework_modules,
        "expected_framework_module_count": len(FRAMEWORK_INPUT_MODULES),
        "missing_framework_modules": missing_framework_modules,
        "unexpected_framework_modules": unexpected_framework_modules,
        "test_modules": test_modules,
        "expected_test_module_count": len(INPUT_TEST_MODULES),
        "missing_test_modules": missing_test_modules,
        "missing_input_mod_declarations": missing_input_mod_declarations,
        "missing_runtime_mod_declarations": missing_runtime_mod_declarations,
        "missing_framework_mod_declarations": missing_framework_mod_declarations,
        "missing_test_mod_declarations": missing_test_mod_declarations,
        "missing_public_surface": missing_public_surface,
        "public_surface_anchor_count": len(PUBLIC_SURFACE_ANCHORS),
        "missing_action_evaluator_anchors": missing_action_evaluator_anchors,
        "missing_gamepad_abi_anchors": missing_gamepad_abi_anchors,
        "missing_cursor_host_request_anchors": missing_cursor_host_request_anchors,
        "cursor_host_request_anchor_count": len(CURSOR_HOST_REQUEST_ANCHORS),
        "missing_runtime_12_guards": missing_runtime_12_guards,
        "guard_anchor_count": len(RUNTIME_12_GUARDS),
        "expected_guard_file_count": EXPECTED_INPUT_STACK_GUARD_FILE_COUNT,
        "missing_guard_files": missing_guard_files,
        "missing_doc_anchors": missing_doc_anchors,
        "missing_test_anchors": missing_test_anchors,
        "behavior_test_anchor_count": len(RUNTIME_12_BEHAVIOR_TEST_ANCHORS),
        "missing_behavior_test_anchors": missing_behavior_test_anchors,
        "missing_cargo_gate_anchors": missing_cargo_gate_anchors,
        "max_production_module_lines": INPUT_PRODUCTION_MODULE_MAX_LINES,
        "max_test_module_lines": INPUT_TEST_MODULE_MAX_LINES,
        "oversized_modules": oversized_modules,
        "mirror_docs_guard": MIRROR_DOCS_GUARD,
        "mirror_docs_guard_present": MIRROR_DOCS_GUARD in input_stack_guard_source,
        "risks": risks,
    }
